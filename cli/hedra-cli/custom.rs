//! Custom command handlers.
//!
//! This file is yours to edit — add it to `.fernignore` so
//! `fern generate` will never overwrite your changes.
//!
//! The generated `main.rs` calls `custom::register(app)` at
//! startup, composing your commands into the CLI at compile time.
//!
//! Each handler receives an `AppContext`. Use `super::sdk::client(ctx)`
//! to get a fully-wired SDK client that inherits the CLI's auth,
//! retries, TLS, and global headers. Use `super::sdk::block_on(future)`
//! to run async SDK calls from synchronous handler context.
//! Types are available via `hedra_cli_sdk::api::*`.

use std::sync::Arc;
use std::time::Duration;

use fern_cli_sdk::app::CliApp;
use fern_cli_sdk::auth::{
    active_store, DynAuthProvider, LoginContext, LoginFlow, OAuth2KeyringProvider, PkceLoginFlow,
};
use fern_cli_sdk::error::CliError;
use serde::{Deserialize, Serialize};

/// Register custom commands and login flows on the CLI app builder.
///
/// Called from `main.rs` during startup.
pub fn register(app: CliApp) -> CliApp {
    app.login_flow(DcrPkceLoginFlow)
}

// ---------------------------------------------------------------------------
// DcrPkceLoginFlow — OAuth login without a pre-provisioned client id
// ---------------------------------------------------------------------------
//
// PROTOTYPE (login-flow mechanics only). `hedra-cli auth login` obtains a
// token from Hedra's AuthKit with no baked-in client id:
//
//   RFC 8414 discovery → RFC 7591 Dynamic Client Registration → the
//   runtime's PkceLoginFlow (authorization-code + PKCE, loopback callback).
//
// The minted registration `{client_id, issuer, token_url}` is persisted in
// the keyring at account `BearerToken.dcr` — deliberately distinct from
// `BearerToken`, where PkceLoginFlow keeps the token bundle — and reused on
// later logins. AuthKit can evict a registration server-side; a persisted
// client is therefore preflighted against the authorize endpoint (a dead one
// 302s to `error=application_not_found`) and re-registered once when dead.
//
// Scope caution: tokens minted this way are AuthKit identity JWTs. The /v3
// REST surface does not accept them (API keys only — the MCP gateway is what
// consumes AuthKit tokens), so the `KeyAuth` env-var path remains the way to
// call the API. This flow exists to prove DCR+PKCE end to end: the flow runs,
// the token lands in the keyring, `auth status` shows it.

/// The v3 spec's http-bearer security scheme. Endpoints listing it route
/// through this flow's provider; the generated `main.rs` binds only
/// `KeyAuth`, so registering this scheme conflicts with nothing.
const SCHEME: &str = "BearerToken";
/// Keyring account for the DCR registration. Must stay distinct from
/// `SCHEME`: PkceLoginFlow owns that account for the token bundle.
const DCR_ACCOUNT: &str = "BearerToken.dcr";
const ISSUER_ENV: &str = "HEDRA_OAUTH_ISSUER";
const DEFAULT_ISSUER: &str = "https://auth.hedra.com";
/// DCR registers this exact URI; the PKCE listener must bind the same port,
/// so wildcard/ephemeral ports are off the table.
const REDIRECT_PORT: u16 = 8484;
const REDIRECT_URI: &str = "http://127.0.0.1:8484/callback";
const SCOPES: [&str; 3] = ["openid", "email", "offline_access"];
const CLIENT_NAME: &str = "hedra-cli";

/// RFC 8414 authorization-server metadata — the three fields this flow needs.
#[derive(Debug, Deserialize)]
struct AsMetadata {
    authorization_endpoint: String,
    token_endpoint: String,
    registration_endpoint: Option<String>,
}

/// RFC 7591 registration response. AuthKit returns no
/// `registration_access_token` (no RFC 7592 management), so the id is all
/// there is to keep.
#[derive(Debug, Deserialize)]
struct RegistrationResponse {
    client_id: String,
}

/// What survives between logins, as JSON in the keyring at `DCR_ACCOUNT`.
/// `token_url` is duplicated out of the metadata so `build_auth_provider`
/// can construct the refresh-capable provider without a network round trip.
#[derive(Debug, Serialize, Deserialize)]
struct PersistedClient {
    client_id: String,
    issuer: String,
    token_url: String,
}

#[derive(Debug)]
struct DcrPkceLoginFlow;

impl LoginFlow for DcrPkceLoginFlow {
    fn flow_type(&self) -> &'static str {
        "dcr-pkce"
    }

    fn scheme_name(&self) -> &str {
        SCHEME
    }

    fn run(&self, ctx: &LoginContext) -> Result<(), CliError> {
        // Same bridge PkceLoginFlow::run uses: dispatch runs inside
        // CliApp::run's multi-thread runtime, so block in place here and
        // hand back a sync context before delegating (the delegate does its
        // own block_in_place).
        let (client, meta) = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(ensure_registration(&ctx.cli_name))
        })?;
        PkceLoginFlow::new(SCHEME)
            .client_id(client.client_id.as_str())
            .authorization_url(meta.authorization_endpoint.as_str())
            .token_url(meta.token_endpoint.as_str())
            .scopes(SCOPES)
            .redirect_port(REDIRECT_PORT)
            .run(ctx)
    }

    fn build_auth_provider(&self, cli_name: &str) -> Option<DynAuthProvider> {
        // Registration-time, synchronous, every process start. No persisted
        // registration (or an unreadable one) → None, and the KeyAuth env
        // path works as before. The one process that ran the very first
        // login misses out harmlessly: its access token is fresh, and the
        // next process start reads the id this login persisted.
        let raw = active_store().get(cli_name, DCR_ACCOUNT).ok()??;
        let client: PersistedClient = serde_json::from_str(&raw).ok()?;
        Some(Arc::new(OAuth2KeyringProvider::new(
            SCHEME,
            cli_name,
            &client.token_url,
            &client.client_id,
        )))
    }
}

/// Resolve the issuer, discover its endpoints, and return a usable client
/// registration — persisted if still live, freshly minted otherwise.
async fn ensure_registration(cli_name: &str) -> Result<(PersistedClient, AsMetadata), CliError> {
    let issuer = issuer();
    let http = http_client()?;
    let meta = discover(&http, &issuer).await?;

    let store = active_store();
    // An entry persisted against a different issuer (env override changed)
    // is ignored, not deleted — flipping the env back finds it again.
    let persisted = store
        .get(cli_name, DCR_ACCOUNT)?
        .and_then(|raw| serde_json::from_str::<PersistedClient>(&raw).ok())
        .filter(|c| c.issuer == issuer);

    if let Some(client) = persisted {
        if client_is_live(&http, &meta, &client.client_id).await {
            eprintln!(
                "Using OAuth client `{}` registered with {issuer}.",
                client.client_id
            );
            return Ok((client, meta));
        }
        eprintln!(
            "OAuth client `{}` is no longer registered with {issuer}; re-registering…",
            client.client_id
        );
    }

    let client_id = register_client(&http, &meta).await?;
    let client = PersistedClient {
        client_id,
        issuer: issuer.clone(),
        token_url: meta.token_endpoint.clone(),
    };
    let value = serde_json::to_string(&client)
        .map_err(|e| CliError::Auth(format!("could not serialize client registration: {e}")))?;
    store.set(cli_name, DCR_ACCOUNT, &value)?;
    eprintln!(
        "✓ Registered OAuth client `{}` with {issuer} (stored in {}).",
        client.client_id,
        store.backend_label()
    );
    Ok((client, meta))
}

fn issuer() -> String {
    std::env::var(ISSUER_ENV)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_ISSUER.to_string())
        .trim_end_matches('/')
        .to_string()
}

/// `oauth_common::token_http_client` is `pub(crate)` — not reachable from
/// this bin crate — so build a minimal client. Redirects stay disabled: the
/// authorize preflight below reads meaning out of the 302, and neither
/// discovery nor registration should be bounced anywhere.
fn http_client() -> Result<reqwest::Client, CliError> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|e| CliError::Auth(format!("could not build HTTP client: {e}")))
}

/// RFC 8414 discovery: `<issuer>/.well-known/oauth-authorization-server`.
async fn discover(http: &reqwest::Client, issuer: &str) -> Result<AsMetadata, CliError> {
    let url = format!("{issuer}/.well-known/oauth-authorization-server");
    let resp = http
        .get(&url)
        .send()
        .await
        .map_err(|e| CliError::Auth(format!("OAuth discovery request failed ({url}): {e}")))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(CliError::Auth(format!(
            "OAuth discovery failed ({url}): HTTP {status}"
        )));
    }
    resp.json::<AsMetadata>().await.map_err(|e| {
        CliError::Auth(format!(
            "OAuth discovery response from {url} is not usable metadata: {e}"
        ))
    })
}

/// RFC 7591 Dynamic Client Registration: mint a public client for the exact
/// redirect URI the PKCE listener will serve. `token_endpoint_auth_method:
/// none` because a CLI cannot keep a secret; PKCE carries the proof instead.
async fn register_client(http: &reqwest::Client, meta: &AsMetadata) -> Result<String, CliError> {
    let endpoint = meta.registration_endpoint.as_deref().ok_or_else(|| {
        CliError::Auth(
            "authorization server metadata has no registration_endpoint; \
             cannot register a client dynamically"
                .to_string(),
        )
    })?;
    let body = serde_json::json!({
        "client_name": CLIENT_NAME,
        "redirect_uris": [REDIRECT_URI],
        "grant_types": ["authorization_code", "refresh_token"],
        "response_types": ["code"],
        "token_endpoint_auth_method": "none",
    });
    let resp = http
        .post(endpoint)
        .json(&body)
        .send()
        .await
        .map_err(|e| CliError::Auth(format!("dynamic client registration failed: {e}")))?;
    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| CliError::Auth(format!("dynamic client registration response: {e}")))?;
    if !status.is_success() {
        return Err(CliError::Auth(format!(
            "dynamic client registration failed (HTTP {status}): {}",
            truncate(&text)
        )));
    }
    serde_json::from_str::<RegistrationResponse>(&text)
        .map(|r| r.client_id)
        .map_err(|e| CliError::Auth(format!("registration response is not usable JSON: {e}")))
}

/// AuthKit answers authorize for an unknown/evicted client with
/// `302 → /oauth2/error?error=application_not_found` — but that redirect
/// happens in the user's browser, invisible to this process. Probe with
/// redirects disabled first, so a dead persisted client is re-registered
/// before a browser tab dead-ends on an error page.
async fn client_is_live(http: &reqwest::Client, meta: &AsMetadata, client_id: &str) -> bool {
    let resp = http
        .get(&meta.authorization_endpoint)
        .query(&[
            ("response_type", "code"),
            ("client_id", client_id),
            ("redirect_uri", REDIRECT_URI),
        ])
        .send()
        .await;
    match resp {
        Ok(resp) => !resp
            .headers()
            .get(reqwest::header::LOCATION)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .contains("application_not_found"),
        // A transport failure proves nothing about the client; let the real
        // authorize attempt surface whatever is wrong.
        Err(_) => true,
    }
}

/// Bound copy of an error body for messages — AS error pages can be huge.
fn truncate(s: &str) -> String {
    const MAX: usize = 300;
    if s.chars().count() <= MAX {
        s.trim().to_string()
    } else {
        let head: String = s.chars().take(MAX).collect();
        format!("{}…", head.trim())
    }
}
