//! The auth flow: an env-late-bound PKCE login, plus the token bootstrap
//! that exchanges the resulting OAuth JWT for a durable API key on the v3
//! login plane.
//!
//! Renew is tried before mint: a mint response is the only time a secret
//! exists in plaintext, so keeping an already-held key alive is preferred.
//! The workspace is named explicitly at mint time; the login itself carries
//! no workspace selector, because the identity provider's organization
//! cannot name an org-less ("born-free") workspace.
//!
//! Hand-written and .fernignore-protected — the generator never emits this
//! file; the ignore entry is what stops regeneration from deleting it.
//! Declared from `custom.rs` via `#[path]` so the generated `main.rs`
//! stays untouched.

use std::sync::{Arc, OnceLock};

use fern_cli_sdk::auth::{
    active_store, AuthProvider, DynAuthProvider, EndpointAuthMetadata, LoginContext, LoginFlow,
    PkceLoginFlow, TokenBundle,
};
use fern_cli_sdk::error::CliError;
use serde_json::Value;

use super::workspaces;

const SCHEME: &str = "OAuth";

/// Keyring slot the minted API key lands in. Must match the scheme name of
/// the `BearerAuth` registered in `main.rs` — `inject_keyring_sources`
/// appends a keyring source at `(cli_name, scheme)` to that chain, which is
/// how the key flows into every data-plane request.
pub(crate) const KEY_SCHEME: &str = "KeyAuth";

#[derive(Clone, Copy, Debug, PartialEq)]
enum HedraEnv {
    Prod,
    Staging,
}

fn hedra_env() -> HedraEnv {
    match std::env::var("HEDRA_ENV") {
        Ok(v) if v.eq_ignore_ascii_case("staging") => HedraEnv::Staging,
        _ => HedraEnv::Prod,
    }
}

/// The compiled resource base for the current `HEDRA_ENV`. These are the
/// only hostnames that ship in the binary — we own them permanently. The
/// authorization server is NOT compiled in; it is discovered from the
/// resource base at runtime, so a vendor domain change is a
/// server-side config edit and released binaries keep working.
pub(crate) fn resource_base_url() -> &'static str {
    match hedra_env() {
        HedraEnv::Staging => "https://api.staging.hedra.com",
        HedraEnv::Prod => "https://api.hedra.com",
    }
}

// ---------------------------------------------------------------------------
// Authorization-server discovery: RFC 9728 → RFC 8414.
//
//   1. GET {resource_base}/.well-known/oauth-protected-resource
//      → validate its `resource` covers ours → authorization_servers[0]
//   2. GET {issuer}/.well-known/oauth-authorization-server
//      → authorization_endpoint + token_endpoint, taken verbatim
//
// Failures hard-fail naming the exact URL and status — deliberately no
// compiled fallback, which would reintroduce the staleness this removes.
// ---------------------------------------------------------------------------

/// Legacy keyring slot that used to cache the discovered endpoints.
///
/// The cache now lives in a plain file — see [`discovery_cache_path`]. These
/// endpoints come from two unauthenticated `.well-known` documents fetched
/// over plain HTTPS; they are public by construction and were never a
/// secret. Keeping them in the OS credential store bought nothing and cost a
/// whole keychain item, which on macOS means its own authorization prompt on
/// every login-plane command.
///
/// The constant survives so [`drop_stale_discovery_item`] can clean up after
/// releases that did write it.
const DISCOVERY_SCHEME: &str = "OAuthDiscovery";

/// Where the endpoint cache lives, alongside the credential store's own
/// `auth-keyring.json`:
///
/// * macOS — `~/Library/Application Support/hedra-cli/auth-endpoints.json`
/// * Linux — `$XDG_CONFIG_HOME/hedra-cli/auth-endpoints.json`, else
///   `~/.config/hedra-cli/auth-endpoints.json`
/// * Windows — `%APPDATA%\hedra-cli\auth-endpoints.json`
///
/// This duplicates the SDK's own `oauth_common::config_dir`, which is
/// `pub(crate)` and so unreachable from here. Kept byte-identical in
/// behaviour on purpose: the two files are siblings, and a CLI whose cache
/// landed somewhere other than its credentials would be its own bug report.
///
/// `None` when no home directory can be determined. The caller treats that
/// as "no cache" and re-discovers, which is correct — unlike the credential
/// store there is nothing here worth a `/tmp` fallback.
fn discovery_cache_path(cli_name: &str) -> Option<std::path::PathBuf> {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .filter(|p| !p.as_os_str().is_empty())?;

    #[cfg(target_os = "macos")]
    let root = home.join("Library").join("Application Support");
    #[cfg(target_os = "windows")]
    let root = std::env::var_os("APPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| home.join("AppData").join("Roaming"));
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let root = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));

    Some(root.join(cli_name).join("auth-endpoints.json"))
}

fn read_cached_endpoints(cli_name: &str) -> Option<AuthEndpoints> {
    let raw = std::fs::read_to_string(discovery_cache_path(cli_name)?).ok()?;
    serde_json::from_str(&raw).ok()
}

/// Best-effort write. A cache that cannot be written costs one extra pair of
/// `.well-known` fetches next run; it must never fail the command that just
/// discovered them.
///
/// Written via temp-file-then-rename so a concurrent reader sees either the
/// old document or the new one, never a half-written file. The temp name
/// carries the pid because two `hedra-cli` processes can discover at once.
fn write_cached_endpoints(cli_name: &str, endpoints: &AuthEndpoints) {
    let Some(path) = discovery_cache_path(cli_name) else {
        return;
    };
    let Ok(json) = serde_json::to_string(endpoints) else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(parent).is_err() {
        return;
    }
    let tmp = parent.join(format!(".auth-endpoints.{}.tmp", std::process::id()));
    if std::fs::write(&tmp, json).is_err() {
        let _ = std::fs::remove_file(&tmp);
        return;
    }
    if std::fs::rename(&tmp, &path).is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
}

/// Remove the endpoint cache from the keyring, where releases before the
/// move stored it.
///
/// Left in place it would be a public document sitting in the OS credential
/// store forever, costing an authorization prompt that buys nothing —
/// nothing reads it any more. Cleared whenever discovery runs, which is
/// every login, so an upgraded install sheds it on first use.
///
/// Silent: on a fresh install there is nothing to delete and the backend
/// says so without prompting.
fn drop_stale_discovery_item(cli_name: &str) {
    let _ = active_store().delete(cli_name, DISCOVERY_SCHEME);
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
struct AuthEndpoints {
    /// The resource base these endpoints were discovered for. Doubles as
    /// the RFC 8707 `resource` indicator and as the cache-invalidation key
    /// (a `HEDRA_ENV` flip changes it, forcing re-discovery).
    resource: String,
    authorization_endpoint: String,
    token_endpoint: String,
}

/// Process-wide endpoint resolution, memoized so discovery runs at most
/// once per process. `use_cache: false` (login) skips the on-disk cache so
/// each login re-validates the live chain; `use_cache: true` (the refresh
/// path) prefers the cached copy.
fn auth_endpoints(cli_name: &str, use_cache: bool) -> Result<AuthEndpoints, CliError> {
    static ENDPOINTS: OnceLock<Result<AuthEndpoints, String>> = OnceLock::new();
    ENDPOINTS
        .get_or_init(|| resolve_auth_endpoints(cli_name, resource_base_url(), use_cache))
        .clone()
        .map_err(CliError::Auth)
}

/// The un-memoized chain (separate so tests can drive it against mock
/// servers and a mock keyring).
fn resolve_auth_endpoints(
    cli_name: &str,
    resource_base_url: &str,
    use_cache: bool,
) -> Result<AuthEndpoints, String> {
    if let Some(overridden) = endpoint_override(resource_base_url)? {
        return Ok(overridden);
    }
    if use_cache {
        if let Some(cached) = read_cached_endpoints(cli_name) {
            if cached.resource == resource_base_url {
                return Ok(cached);
            }
        }
    }
    let discovered = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(discover_endpoints(resource_base_url))
    })?;
    write_cached_endpoints(cli_name, &discovered);
    // Discovery has just run, so anything an older release left in the
    // keyring is both stale and unread. This is the natural moment to shed
    // it, and it is the only path that reaches every install.
    drop_stale_discovery_item(cli_name);
    Ok(discovered)
}

/// Operator escape hatch: explicit endpoints, skipping discovery entirely.
///
/// Deliberately a **pair of endpoints** rather than a base URL with
/// reconstructed `/oauth2/...` paths. Reconstruction would re-hardcode the
/// vendor URL shape this whole chain exists to remove, so it would break in
/// precisely the scenario the hatch is for — a vendor URL change — and it
/// would be useless against the likelier failure, a discovery document that
/// is served but wrong. Taking both endpoints verbatim survives either.
///
/// Both or neither. A half-configured override mixes one overridden leg with
/// one discovered leg, which is never what was meant, and silently sending an
/// authorization code to an unintended token endpoint is the one mistake here
/// that hands over a credential — so it errors instead.
///
/// Not advertised in `--help` or the README: it exists for operators
/// recovering from a broken discovery document, and for pointing the flow at
/// a local or mock authorization server. Both legs must be https (loopback
/// exempt) — the token leg carries the PKCE exchange.
fn endpoint_override(resource_base_url: &str) -> Result<Option<AuthEndpoints>, String> {
    let read = |key: &str| {
        std::env::var(key)
            .ok()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty())
    };
    match (
        read("HEDRA_AUTH_AUTHORIZE_URL"),
        read("HEDRA_AUTH_TOKEN_URL"),
    ) {
        (None, None) => Ok(None),
        (Some(authorization_endpoint), Some(token_endpoint)) => {
            require_https(&authorization_endpoint)?;
            require_https(&token_endpoint)?;
            Ok(Some(AuthEndpoints {
                resource: resource_base_url.to_string(),
                authorization_endpoint,
                token_endpoint,
            }))
        }
        _ => Err(
            "HEDRA_AUTH_AUTHORIZE_URL and HEDRA_AUTH_TOKEN_URL must be set together \
                  (one alone would mix an overridden endpoint with a discovered one)"
                .to_string(),
        ),
    }
}

async fn discover_endpoints(resource_base_url: &str) -> Result<AuthEndpoints, String> {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("could not build HTTP client: {e}"))?;

    // Hop 1 — RFC 9728: the resource names its authorization server(s).
    let url = format!("{resource_base_url}/.well-known/oauth-protected-resource");
    let doc = fetch_json(&http, &url).await?;
    let published = doc
        .get("resource")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{url}: document has no `resource` field"))?;
    if !resource_covers(published, resource_base_url) {
        return Err(format!(
            "{url}: refusing document for resource `{published}` — it is neither \
             `{resource_base_url}` nor an ancestor of it"
        ));
    }
    let issuer = doc
        .get("authorization_servers")
        .and_then(|v| v.get(0))
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{url}: document lists no authorization_servers"))?
        .trim_end_matches('/')
        .to_string();
    require_https(&issuer).map_err(|e| format!("{url}: {e}"))?;

    // Hop 2 — RFC 8414: the issuer names its endpoints. Taken verbatim —
    // reconstructing `{issuer}/oauth2/…` would re-hardcode the vendor's
    // path shape, the very thing this chain exists to avoid.
    let meta_url = format!("{issuer}/.well-known/oauth-authorization-server");
    let meta = fetch_json(&http, &meta_url).await?;
    let endpoint = |key: &str| -> Result<String, String> {
        meta.get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("{meta_url}: metadata has no `{key}`"))
    };
    Ok(AuthEndpoints {
        resource: resource_base_url.to_string(),
        authorization_endpoint: endpoint("authorization_endpoint")?,
        token_endpoint: endpoint("token_endpoint")?,
    })
}

async fn fetch_json(http: &reqwest::Client, url: &str) -> Result<Value, String> {
    let resp = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("discovery failed: GET {url}: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("discovery failed: GET {url} returned {status}"));
    }
    let body = resp.text().await.unwrap_or_default();
    serde_json::from_str(&body).map_err(|e| format!("{url}: response is not JSON: {e}"))
}

/// RFC 9728 client rule, as the MCP TypeScript SDK applies it: trust the
/// document only when its `resource` IS the resource we were configured
/// with, or an ancestor of it (path-boundary prefix).
fn resource_covers(published: &str, requested: &str) -> bool {
    let published = published.trim_end_matches('/');
    let requested = requested.trim_end_matches('/');
    if published.is_empty() {
        return false;
    }
    requested == published
        || (requested.starts_with(published)
            && requested.as_bytes().get(published.len()) == Some(&b'/'))
}

/// The issuer must be https; loopback is exempt so local mock servers (and
/// the wiremock tests) can exercise the chain.
fn require_https(issuer: &str) -> Result<(), String> {
    if issuer.starts_with("https://") {
        return Ok(());
    }
    let loopback = issuer.strip_prefix("http://").is_some_and(|rest| {
        let host = rest.split(['/', ':']).next().unwrap_or("");
        matches!(host, "127.0.0.1" | "localhost")
    });
    if loopback {
        Ok(())
    } else {
        Err(format!("authorization server `{issuer}` is not https"))
    }
}

/// Derive the data-plane base URL from `HEDRA_ENV` so one knob switches the
/// whole CLI (login plane AND generated commands). Called from `register()`,
/// i.e. before clap parses — `resolve_base_url_override` reads
/// `HEDRA_CLI_BASE_URL` at parse time and the `--base-url` flag still beats
/// it, so precedence is untouched. The value replaces the spec's server URL
/// wholesale (executor `build_url`), so it must carry the `/v3` prefix the
/// spec's `https://api.hedra.com/v3` server carries.
///
/// `.env` values are honored too: `register()` loads dotenv before calling
/// this (the runtime only loads it later, inside `run()`), so both a
/// `.env`-set `HEDRA_ENV` and a `.env`-set `HEDRA_CLI_BASE_URL` are visible
/// here. Without that early load the login plane (which reads `HEDRA_ENV`
/// at dispatch time, post-dotenv) and the data plane would split-brain on a
/// `.env`-only `HEDRA_ENV=staging`.
pub(crate) fn derive_base_url_from_hedra_env() {
    if std::env::var_os("HEDRA_CLI_BASE_URL").is_some() {
        return; // explicit override — never clobber
    }
    // Prod derives nothing: the spec's server URL already is prod, and
    // leaving the variable unset keeps `--spec`/help output untouched.
    if hedra_env() == HedraEnv::Staging {
        std::env::set_var("HEDRA_CLI_BASE_URL", format!("{}/v3", resource_base_url()));
    }
}

/// Graft the discovered endpoints onto the env-independent base flow.
/// Callers must only invoke this at dispatch/request time — `register()`
/// runs before `run_with_args` loads `.env`, so an eager call would freeze
/// the wrong env (and discovery is I/O besides).
/// The login carries NO workspace selector — deliberately. The identity
/// provider's organization cannot name an org-less ("born-free") workspace, so
/// hinting the hosted login at an org selects the wrong thing or nothing;
/// the workspace is named at mint time instead, by id.
fn concretize(base: &PkceLoginFlow, endpoints: &AuthEndpoints) -> PkceLoginFlow {
    let resource = endpoints.resource.as_str();
    base.clone()
        .client_id(format!("{resource}/.well-known/hedra-cli.json"))
        .authorization_url(endpoints.authorization_endpoint.clone())
        .token_url(endpoints.token_endpoint.clone())
        // RFC 8707 resource indicator on all three legs; refresh_params ride
        // into OAuth2KeyringProvider via build_auth_provider.
        .authorization_params([("resource", resource)])
        .token_params([("resource", resource)])
        .refresh_params([("resource", resource)])
}

/// Mint a **fresh** login-plane JWT from the stored session.
///
/// The login-plane endpoints apply a much shorter freshness window than
/// ordinary bearer validation: a token is refused once its `iat` is more
/// than a few minutes old, with "The login token is too old to use here."
/// The ordinary provider only refreshes once a token is *expired* by its
/// own `expires_at`, which is far longer, so a perfectly valid access token
/// sails past the client and is refused by the server. Every login-plane
/// call therefore exchanges the refresh token for a brand-new access token
/// rather than reusing the stored one.
///
/// The rotated refresh token is written back: the identity provider
/// rotates it on every
/// exchange, so dropping the new one would break the *next* call.
pub(crate) fn fresh_login_jwt(cli_name: &str) -> Result<String, CliError> {
    let endpoints =
        resolve_auth_endpoints(cli_name, resource_base_url(), true).map_err(CliError::Auth)?;
    let raw = active_store().get(cli_name, SCHEME)?.ok_or_else(|| {
        CliError::Auth(format!(
            "Not logged in. Run `{cli_name} auth login` to authenticate."
        ))
    })?;
    let bundle: TokenBundle = serde_json::from_str(&raw)
        .map_err(|e| CliError::Auth(format!("stored OAuth bundle is not valid JSON: {e}")))?;
    let refresh = bundle.refresh_token.clone().ok_or_else(|| {
        CliError::Auth(format!(
            "the stored session carries no refresh token — run `{cli_name} auth login` again"
        ))
    })?;
    let refreshed = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(refresh_token_grant(&endpoints, &refresh))
    })?;
    // Best-effort persist: the call at hand already has its token, so a
    // keyring hiccup must not fail it — only the next call pays.
    if let Ok(json) = refreshed.to_keyring_value() {
        let _ = active_store().set(cli_name, SCHEME, &json);
    }
    Ok(refreshed.access_token)
}

async fn refresh_token_grant(
    endpoints: &AuthEndpoints,
    refresh_token: &str,
) -> Result<TokenBundle, CliError> {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CliError::Auth(format!("could not build HTTP client: {e}")))?;
    let client_id = format!("{}/.well-known/hedra-cli.json", endpoints.resource);
    let resp = http
        .post(&endpoints.token_endpoint)
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", client_id.as_str()),
            // Same RFC 8707 indicator the login and refresh legs carry.
            ("resource", endpoints.resource.as_str()),
        ])
        .send()
        .await
        .map_err(|e| {
            CliError::Auth(format!(
                "token refresh failed: POST {}: {e}",
                endpoints.token_endpoint
            ))
        })?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(CliError::Auth(format!(
            "session refresh refused ({status}) — run `hedra-cli auth login` again: {}",
            body.chars().take(200).collect::<String>()
        )));
    }
    let parsed: Value = serde_json::from_str(&body)
        .map_err(|e| CliError::Auth(format!("token endpoint returned non-JSON: {e}")))?;
    let access = parsed
        .get("access_token")
        .and_then(Value::as_str)
        .ok_or_else(|| CliError::Auth("token response carries no access_token".to_string()))?;
    Ok(TokenBundle::from_token_response(
        access,
        parsed
            .get("refresh_token")
            .and_then(Value::as_str)
            .or(Some(refresh_token)),
        parsed.get("expires_in").and_then(Value::as_u64),
    ))
}

/// Env-late-binding wrapper around a partial `PkceLoginFlow` holding the
/// env-independent config (scheme, scopes). Identity methods delegate to
/// the base; `run` and the auth provider concretize it with `HEDRA_ENV`'s
/// endpoints when they execute, not when `register()` builds the app.
#[derive(Debug)]
pub(crate) struct EnvPkceLoginFlow {
    base: PkceLoginFlow,
}

impl EnvPkceLoginFlow {
    pub(crate) fn new() -> Self {
        Self {
            base: PkceLoginFlow::new(SCHEME).scopes(["openid", "email", "offline_access"]),
        }
    }
}

impl LoginFlow for EnvPkceLoginFlow {
    fn flow_type(&self) -> &'static str {
        self.base.flow_type()
    }
    fn scheme_name(&self) -> &str {
        self.base.scheme_name()
    }
    fn run(&self, ctx: &LoginContext) -> Result<(), CliError> {
        eprintln!(
            "Login environment: {:?} (set HEDRA_ENV=staging to switch)",
            hedra_env()
        );
        // Fresh discovery on every login (the on-disk cache is skipped):
        // login is
        // the natural point to re-validate the chain and rewrite the cache.
        let endpoints = auth_endpoints(&ctx.cli_name, false)?;
        concretize(&self.base, &endpoints).run(ctx)?;
        dump_token_claims(&ctx.cli_name);
        bootstrap_api_key(&ctx.cli_name)
    }
    fn token_paste_url(&self) -> Option<&str> {
        // Fully qualified: the inherent builder *setter* of the same name
        // would otherwise shadow the trait getter.
        LoginFlow::token_paste_url(&self.base)
    }
    fn build_auth_provider(&self, cli_name: &str) -> Option<DynAuthProvider> {
        // The one method that must NOT delegate to the base: it is called
        // at registration time (pre-.env, pre-parse) and the base has no
        // token_url/client_id yet — return a lazy shell only.
        Some(Arc::new(EnvOAuthProvider {
            cli_name: cli_name.to_string(),
            base: self.base.clone(),
            inner: OnceLock::new(),
        }))
    }
}

/// Request-time provider that defers env resolution to first use, so
/// Bearer + refresh hit the same environment the login flow targeted.
#[derive(Debug)]
struct EnvOAuthProvider {
    cli_name: String,
    base: PkceLoginFlow,
    inner: OnceLock<Result<DynAuthProvider, String>>,
}

impl EnvOAuthProvider {
    fn resolved(&self) -> Result<&DynAuthProvider, CliError> {
        self.inner
            .get_or_init(|| {
                // Cache-first endpoints: the refresh path must not re-run
                // discovery on every invocation.
                let endpoints = auth_endpoints(&self.cli_name, true).map_err(|e| e.to_string())?;
                Ok(concretize(&self.base, &endpoints)
                    .build_auth_provider(&self.cli_name)
                    .expect("PkceLoginFlow always supplies a provider"))
            })
            .as_ref()
            .map_err(|e| CliError::Auth(e.clone()))
    }
}

impl AuthProvider for EnvOAuthProvider {
    fn name(&self) -> &str {
        self.base.scheme_name()
    }
    fn has_credentials(&self) -> bool {
        // Keyring presence only — `auth status` must not trigger a
        // discovery fetch just to answer "am I logged in".
        matches!(active_store().get(&self.cli_name, SCHEME), Ok(Some(_)))
    }
    fn credential_hints(&self) -> Vec<String> {
        match self.inner.get() {
            Some(Ok(provider)) => provider.credential_hints(),
            _ => vec![format!("run `{} auth login`", self.cli_name)],
        }
    }
    fn apply(
        &self,
        request: reqwest::RequestBuilder,
        endpoint: &EndpointAuthMetadata,
    ) -> Result<reqwest::RequestBuilder, CliError> {
        self.resolved()?.apply(request, endpoint)
    }
}

/// Print selected claims from the freshly minted access token. Diagnostic
/// only — decodes the JWT payload without verifying the signature.
fn dump_token_claims(cli_name: &str) {
    let bundle: TokenBundle = match active_store().get(cli_name, SCHEME) {
        Ok(Some(stored)) => match serde_json::from_str(&stored) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("(stored token bundle is not valid JSON: {e})");
                return;
            }
        },
        _ => {
            eprintln!("(no stored token bundle to inspect)");
            return;
        }
    };
    match jwt_claims(&bundle.access_token) {
        Ok(claims) => {
            eprintln!("Access-token claims:");
            for key in [
                "aud",
                "iss",
                "sub",
                "sid",
                "org_id",
                "organization_id",
                // The client-identity claims — their absence once forced
                // an out-of-band probe to establish which client was used.
                "client_id",
                "azp",
            ] {
                let shown = match claims.get(key) {
                    Some(Value::String(s)) => s.clone(),
                    Some(other) => other.to_string(),
                    None => "(absent)".to_string(),
                };
                eprintln!("  {key:<16} {shown}");
            }
        }
        Err(e) => eprintln!("(could not decode access token as a JWT: {e})"),
    }
}

fn jwt_claims(token: &str) -> Result<serde_json::Map<String, Value>, String> {
    use base64::Engine as _;
    let mut parts = token.split('.');
    let (Some(_header), Some(payload), Some(_sig), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err("not a three-segment JWT".to_string());
    };
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|e| format!("payload is not base64url: {e}"))?;
    match serde_json::from_slice(&bytes) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(_) => Err("payload is not a JSON object".to_string()),
        Err(e) => Err(format!("payload is not JSON: {e}")),
    }
}

// ---------------------------------------------------------------------------
// Token bootstrap (login plane): OAuth JWT → durable API key.
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
struct BootstrapMintResponse {
    key_id: String,
    /// The full `<key_id>:<secret>` pair — shown exactly once, at mint.
    credential: String,
    #[serde(default)]
    workspace_id: Option<String>,
    #[serde(default)]
    workspace_name: Option<String>,
    #[serde(default)]
    organization_id: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct BootstrapRenewResponse {
    key_id: String,
    expires_at: String,
    #[serde(default)]
    workspace_id: Option<String>,
}

fn bootstrap_api_key(cli_name: &str) -> Result<(), CliError> {
    let api_base = resource_base_url();
    let jwt = match active_store().get(cli_name, SCHEME) {
        Ok(Some(stored)) => serde_json::from_str::<TokenBundle>(&stored)
            .map(|b| b.access_token)
            .map_err(|e| CliError::Auth(format!("stored OAuth bundle is not valid JSON: {e}")))?,
        _ => {
            return Err(CliError::Auth(
                "no OAuth token in the keyring — login did not complete".to_string(),
            ))
        }
    };
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(bootstrap_inner(cli_name, api_base, &jwt))
    })
}

async fn bootstrap_inner(cli_name: &str, api_base: &str, jwt: &str) -> Result<(), CliError> {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CliError::Auth(format!("could not build HTTP client: {e}")))?;

    // 1. The picker listing, authenticated with the login JWT.
    let listing = workspaces::fetch_workspaces(&http, api_base, jwt).await?;

    // 2. Renew the held key if there is one; mint on any refusal. One JWT
    //    covers every leg below, so the fallback mint needs no second
    //    login — do not reorder these on the assumption that it does.
    let held = active_store().get(cli_name, KEY_SCHEME).ok().flatten();
    let (key_id, expires_at, workspace_id, minted) = match held {
        Some(credential) => match try_renew(&http, api_base, &credential, jwt).await? {
            Some(renewed) => {
                record_workspace_key(
                    cli_name,
                    renewed.workspace_id.as_deref(),
                    &renewed.key_id,
                    &credential,
                    None, // renewals carry no name — the held one is kept
                    Some(&renewed.expires_at),
                    true,
                );
                (
                    renewed.key_id,
                    Some(renewed.expires_at),
                    renewed.workspace_id,
                    false,
                )
            }
            None => mint_and_store(&http, api_base, jwt, cli_name).await?,
        },
        None => mint_and_store(&http, api_base, jwt, cli_name).await?,
    };

    // 3. Render: the mint/renew response's workspace is the authoritative
    //    selection (the JWT's org is the sole selector server-side).
    let map = workspaces::WorkspaceKeyMap::load(cli_name);
    eprint!(
        "{}",
        workspaces::render_listing_table(&listing, workspace_id.as_deref(), &map.keys)
    );
    let expiry = expires_at
        .as_deref()
        .map(describe_expiry)
        .unwrap_or_else(|| "no expiry reported".to_string());
    eprintln!(
        "API key {key_id} {} — {expiry}.",
        if minted {
            "minted and stored in the keyring"
        } else {
            "renewed (existing keyring credential still valid)"
        }
    );
    if std::env::var("HEDRA_API_KEY").is_ok() {
        eprintln!(
            "⚠ HEDRA_API_KEY is set and shadows the keyring — unset it to use the bootstrapped key."
        );
    }
    Ok(())
}

/// `Ok(Some(_))` = renewed; `Ok(None)` = refused (expired / revoked / age
/// cap / identity mismatch / not a bootstrap-renewable key) → caller mints.
/// `Err` = the environment has no login plane at all, which a mint would
/// only repeat.
async fn try_renew(
    http: &reqwest::Client,
    api_base: &str,
    credential: &str,
    jwt: &str,
) -> Result<Option<BootstrapRenewResponse>, CliError> {
    let resp = http
        .post(format!("{api_base}/v3/keys/bootstrap/renew"))
        .bearer_auth(credential)
        .json(&serde_json::json!({ "authkit_token": jwt }))
        .send()
        .await
        .map_err(|e| CliError::Auth(format!("POST /v3/keys/bootstrap/renew failed: {e}")))?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::NOT_FOUND {
        return Err(CliError::Auth(format!(
            "key renewal refused — {}",
            login_plane_error(status, &body)
        )));
    }
    if !status.is_success() {
        eprintln!(
            "(held key not renewable: {} — minting a fresh one)",
            login_plane_error(status, &body)
        );
        return Ok(None);
    }
    serde_json::from_str(&body)
        .map(Some)
        .map_err(|e| CliError::Auth(format!("unexpected renew response: {e}")))
}

/// POST the mint and parse the response. The builder must already carry a
/// login credential (the login-fresh JWT, or the stored OAuth session via
/// [`fresh_login_jwt`]). `workspace_id` names the target workspace when the
/// caller has one; omitted, the server picks — today's behavior.
async fn post_bootstrap_mint(
    req: reqwest::RequestBuilder,
    workspace_id: Option<&str>,
) -> Result<BootstrapMintResponse, CliError> {
    let mut body = serde_json::json!({ "name": device_name() });
    if let Some(ws) = workspace_id {
        body["workspace_id"] = Value::String(ws.to_string());
    }
    let resp = req
        .json(&body)
        .send()
        .await
        .map_err(|e| CliError::Auth(format!("POST /v3/keys/bootstrap failed: {e}")))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(CliError::Auth(format!(
            "key mint refused — {}",
            login_plane_error(status, &text)
        )));
    }
    serde_json::from_str(&text)
        .map_err(|e| CliError::Auth(format!("unexpected mint response: {e}")))
}

async fn mint_and_store(
    http: &reqwest::Client,
    api_base: &str,
    jwt: &str,
    cli_name: &str,
) -> Result<(String, Option<String>, Option<String>, bool), CliError> {
    let req = http
        .post(format!("{api_base}/v3/keys/bootstrap"))
        .bearer_auth(jwt);
    let minted = post_bootstrap_mint(req, None).await?;
    record_workspace_key(
        cli_name,
        minted.workspace_id.as_deref(),
        &minted.key_id,
        &minted.credential,
        minted.workspace_name.as_deref(),
        minted.expires_at.as_deref(),
        true,
    );
    if let (Some(name), Some(org)) = (&minted.workspace_name, &minted.organization_id) {
        eprintln!("(key bound to workspace \"{name}\" via organization {org})");
    }
    Ok((minted.key_id, minted.expires_at, minted.workspace_id, true))
}

/// Is there an OAuth session to work from at all? Keyring presence only —
/// `workspaces select` uses this to decide whether it must open a browser,
/// without paying for discovery or a refresh to find out.
pub(crate) fn has_oauth_session(cli_name: &str) -> bool {
    matches!(active_store().get(cli_name, SCHEME), Ok(Some(t)) if !t.is_empty())
}

/// What `workspaces select` needs back from a targeted mint.
#[derive(Debug)]
pub(crate) struct MintedKey {
    pub(crate) key_id: String,
    pub(crate) workspace_name: Option<String>,
    pub(crate) expires_at: Option<String>,
}

/// Mint a key bound to `workspace_id` and make it the active credential
/// Runs off the stored OAuth session — refreshed through the
/// same provider the data plane uses — so switching workspaces costs no
/// browser round-trip; the login JWT's own org is irrelevant here.
pub(crate) fn mint_for_workspace(
    cli_name: &str,
    workspace_id: &str,
) -> Result<MintedKey, CliError> {
    mint_for_workspace_at(cli_name, resource_base_url(), workspace_id)
}

/// The api-base-parameterized body (tests drive it against a mock server),
/// mirroring `bootstrap_inner`'s shape.
fn mint_for_workspace_at(
    cli_name: &str,
    api_base: &str,
    workspace_id: &str,
) -> Result<MintedKey, CliError> {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CliError::Auth(format!("could not build HTTP client: {e}")))?;
    // A mint is a login-plane call: it needs a token minted seconds ago,
    // not merely an unexpired one (see `fresh_login_jwt`).
    let jwt = fresh_login_jwt(cli_name)?;
    let req = http
        .post(format!("{api_base}/v3/keys/bootstrap"))
        .bearer_auth(jwt);
    let minted = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(post_bootstrap_mint(req, Some(workspace_id)))
    })?;

    // Compatibility guard, not paranoia: the API ignores unknown request
    // fields rather than rejecting them, so a deployment that predates
    // workspace-targeted mint silently DROPS `workspace_id` and binds the
    // key to the login's own workspace. Trusting the request would file a
    // key under a workspace it does not belong to — and bill the wrong
    // target. The response is the only trustworthy source.
    if minted.workspace_id.as_deref() != Some(workspace_id) {
        let landed = minted.workspace_id.as_deref();
        // The credential is real; record it where it actually belongs so it
        // is not orphaned — but leave the active credential alone, because
        // the workspace the user asked for was not reached.
        record_workspace_key(
            cli_name,
            landed,
            &minted.key_id,
            &minted.credential,
            minted.workspace_name.as_deref(),
            minted.expires_at.as_deref(),
            false,
        );
        return Err(CliError::Auth(format!(
            "the mint bound its key to workspace {} instead of the requested {workspace_id} — \
             this environment does not support selecting a workspace at mint time yet. \
             The key was kept for {}; the active workspace is unchanged.",
            landed.unwrap_or("<none>"),
            landed.unwrap_or("that workspace")
        )));
    }

    record_workspace_key(
        cli_name,
        Some(workspace_id),
        &minted.key_id,
        &minted.credential,
        minted.workspace_name.as_deref(),
        minted.expires_at.as_deref(),
        true,
    );
    Ok(MintedKey {
        key_id: minted.key_id,
        workspace_name: minted.workspace_name,
        expires_at: minted.expires_at,
    })
}

/// Best-effort update of the per-workspace key map (`workspaces select`'s
/// data source). The credential is already safe in `KeyAuth`, so a map
/// write failure must not fail the login — but it must not be silent
/// either, or a later `select` mysteriously lacks the key.
#[allow(clippy::too_many_arguments)]
fn record_workspace_key(
    cli_name: &str,
    workspace_id: Option<&str>,
    key_id: &str,
    credential: &str,
    workspace_name: Option<&str>,
    expires_at: Option<&str>,
    activate: bool,
) {
    if let Err(e) = workspaces::record_key(
        cli_name,
        workspace_id,
        key_id,
        credential,
        workspace_name,
        expires_at,
        activate,
    ) {
        eprintln!("(could not record the workspace key map: {e})");
        return;
    }
    if activate {
        drop_stale_key_mirror(cli_name);
    }
}

/// Remove the legacy standalone `KeyAuth` keyring item, if one is still
/// there.
///
/// Releases before the projection landed stored the active credential twice:
/// once inside the workspace key map, and once as its own item at
/// `(cli_name, KeyAuth)` — which is the address the SDK's injected keyring
/// source reads. The map is now the only writer, so an item left over from
/// an older release would sit there frozen at whatever key was active on the
/// day of the upgrade.
///
/// [`super::active_key`] prefers the map precisely so that stale item cannot
/// win, but leaving it in place would keep a live credential in the keychain
/// that nothing maintains — and would keep costing an authorization prompt.
/// Clearing it at the moments the active credential changes migrates the
/// install on first use.
///
/// Best-effort and silent: on a fresh install there is nothing to delete and
/// the backend says so without prompting, and a failure here must never sink
/// a login that has otherwise succeeded.
pub(crate) fn drop_stale_key_mirror(cli_name: &str) {
    let _ = active_store().delete(cli_name, KEY_SCHEME);
}

/// Server error body → one readable line; a 404 on this plane almost always
/// means the feature gate, not a wrong URL.
pub(crate) fn login_plane_error(status: reqwest::StatusCode, body: &str) -> String {
    let detail = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| {
            v.pointer("/error/message")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_else(|| body.chars().take(300).collect());
    let hint = if status == reqwest::StatusCode::NOT_FOUND {
        " (the login plane may not be enabled on this environment)"
    } else {
        ""
    };
    format!("{status}: {detail}{hint}")
}

/// Display-only label for the minted key — never used by the server to
/// match or replace anything.
fn device_name() -> String {
    let host = std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown-host".to_string());
    format!("hedra-cli @ {host}")
}

/// "expires 2026-08-19T00:00:00Z (in 23h 58m)" — the ISO instant is kept
/// verbatim (it carries its offset) and the relative form disambiguates.
fn describe_expiry(iso: &str) -> String {
    let Some(expiry_epoch) = iso8601_to_epoch(iso) else {
        return format!("expires {iso}");
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let delta = expiry_epoch - now;
    let human = |secs: i64| -> String {
        let (h, m) = (secs / 3600, (secs % 3600) / 60);
        if h > 48 {
            format!("{}d {}h", h / 24, h % 24)
        } else if h > 0 {
            format!("{h}h {m}m")
        } else {
            format!("{m}m")
        }
    };
    if delta >= 0 {
        format!("expires {iso} (in {})", human(delta))
    } else {
        format!("expired {iso} ({} ago)", human(-delta))
    }
}

/// Minimal ISO-8601 → Unix epoch seconds: `YYYY-MM-DDTHH:MM:SS[.frac]`
/// followed by `Z` or `±HH:MM`. Enough for the server's UTC instants;
/// anything else returns `None` and the caller shows the raw string.
fn iso8601_to_epoch(s: &str) -> Option<i64> {
    let s = s.trim();
    let (datetime, offset_secs) = if let Some(rest) = s.strip_suffix('Z') {
        (rest, 0i64)
    } else if s.len() > 6 {
        let (dt, off) = s.split_at(s.len() - 6);
        let sign = match off.as_bytes()[0] {
            b'+' => 1i64,
            b'-' => -1i64,
            _ => return None,
        };
        let (oh, om) = off[1..].split_once(':')?;
        (
            dt,
            sign * (oh.parse::<i64>().ok()? * 3600 + om.parse::<i64>().ok()? * 60),
        )
    } else {
        return None;
    };
    let (date, time) = datetime.split_once('T')?;
    let mut dp = date.split('-');
    let (y, mo, d) = (
        dp.next()?.parse::<i64>().ok()?,
        dp.next()?.parse::<u32>().ok()?,
        dp.next()?.parse::<i64>().ok()?,
    );
    if dp.next().is_some() || !(1..=12).contains(&mo) {
        return None;
    }
    let time = time.split_once('.').map(|(t, _)| t).unwrap_or(time);
    let mut tp = time.split(':');
    let (h, mi, sec) = (
        tp.next()?.parse::<i64>().ok()?,
        tp.next()?.parse::<i64>().ok()?,
        tp.next().unwrap_or("0").parse::<i64>().ok()?,
    );
    // Days-from-civil (Howard Hinnant): valid for all Gregorian dates.
    let mo = mo as i64;
    let y_adj = if mo <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = y_adj - era * 400;
    let doy = (153 * (if mo > 2 { mo - 3 } else { mo + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;
    Some(days * 86400 + h * 3600 + mi * 60 + sec - offset_secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    #[test]
    fn jwt_claims_decodes_payload() {
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(r#"{"aud":"https://api.hedra.com/","sub":"user_123","org_id":"org_9"}"#);
        let token = format!("e30.{payload}.sig");
        let claims = jwt_claims(&token).unwrap();
        assert_eq!(claims["aud"], "https://api.hedra.com/");
        assert_eq!(claims["sub"], "user_123");
        assert_eq!(claims["org_id"], "org_9");
        assert!(claims.get("sid").is_none());
    }

    #[test]
    fn jwt_claims_rejects_opaque_token() {
        assert!(jwt_claims("opaque-token").is_err());
        assert!(jwt_claims("a.b.c.d").is_err());
    }

    #[test]
    fn iso8601_to_epoch_reference_values() {
        assert_eq!(iso8601_to_epoch("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(iso8601_to_epoch("2026-08-19T00:00:00Z"), Some(1787097600));
        assert_eq!(
            iso8601_to_epoch("2000-02-29T12:30:45+00:00"),
            Some(951827445)
        );
        // Offset shifts back to the same UTC instant.
        assert_eq!(
            iso8601_to_epoch("2026-08-19T02:00:00+02:00"),
            Some(1787097600)
        );
        // Fractional seconds are truncated, not rejected.
        assert_eq!(
            iso8601_to_epoch("2026-08-19T00:00:00.123456Z"),
            Some(1787097600)
        );
        assert_eq!(iso8601_to_epoch("not a date"), None);
    }

    #[test]
    #[serial_test::serial]
    fn base_url_derivation_rules() {
        let restore_env = std::env::var("HEDRA_ENV").ok();
        let restore_base = std::env::var("HEDRA_CLI_BASE_URL").ok();

        // staging + no explicit override → derived, with the /v3 prefix.
        std::env::set_var("HEDRA_ENV", "staging");
        std::env::remove_var("HEDRA_CLI_BASE_URL");
        derive_base_url_from_hedra_env();
        assert_eq!(
            std::env::var("HEDRA_CLI_BASE_URL").as_deref(),
            Ok("https://api.staging.hedra.com/v3")
        );

        // explicit override → never clobbered.
        std::env::set_var("HEDRA_CLI_BASE_URL", "http://localhost:1/v3");
        derive_base_url_from_hedra_env();
        assert_eq!(
            std::env::var("HEDRA_CLI_BASE_URL").as_deref(),
            Ok("http://localhost:1/v3")
        );

        // prod (or unset) derives nothing — the spec default already is prod.
        std::env::remove_var("HEDRA_ENV");
        std::env::remove_var("HEDRA_CLI_BASE_URL");
        derive_base_url_from_hedra_env();
        assert!(std::env::var_os("HEDRA_CLI_BASE_URL").is_none());

        match restore_env {
            Some(v) => std::env::set_var("HEDRA_ENV", v),
            None => std::env::remove_var("HEDRA_ENV"),
        }
        match restore_base {
            Some(v) => std::env::set_var("HEDRA_CLI_BASE_URL", v),
            None => std::env::remove_var("HEDRA_CLI_BASE_URL"),
        }
    }

    // ── discovery tests ──────────────────────────────────────────────

    #[test]
    fn resource_covers_requires_equality_or_path_ancestor() {
        assert!(resource_covers(
            "https://api.hedra.com",
            "https://api.hedra.com"
        ));
        assert!(resource_covers(
            "https://api.hedra.com/",
            "https://api.hedra.com"
        )); // trailing slash
        assert!(resource_covers(
            "https://api.hedra.com",
            "https://api.hedra.com/v3"
        )); // ancestor
        assert!(!resource_covers(
            "https://api.hedra.co",
            "https://api.hedra.com"
        )); // not a path boundary
        assert!(!resource_covers(
            "https://evil.example",
            "https://api.hedra.com"
        ));
        assert!(!resource_covers(
            "https://api.hedra.com/v3",
            "https://api.hedra.com"
        )); // descendant ≠ ancestor
        assert!(!resource_covers("", "https://api.hedra.com"));
    }

    async fn mock_discovery_docs(server: &MockServer, published_resource: &str, issuer: &str) {
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-protected-resource"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "resource": published_resource,
                "resource_name": "Hedra",
                "authorization_servers": [issuer],
                "bearer_methods_supported": ["header"],
            })))
            .mount(server)
            .await;
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-authorization-server"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": issuer,
                // Deliberately NOT the /oauth2/… shape: proves endpoints are
                // taken verbatim from the metadata, not reconstructed.
                "authorization_endpoint": format!("{issuer}/custom/authorize"),
                "token_endpoint": format!("{issuer}/custom/token"),
            })))
            .mount(server)
            .await;
    }

    #[tokio::test]
    async fn discovery_two_hop_takes_endpoints_verbatim() {
        let server = MockServer::start().await;
        let uri = server.uri();
        mock_discovery_docs(&server, &uri, &uri).await;

        let ep = discover_endpoints(&uri).await.unwrap();
        assert_eq!(ep.resource, uri);
        assert_eq!(ep.authorization_endpoint, format!("{uri}/custom/authorize"));
        assert_eq!(ep.token_endpoint, format!("{uri}/custom/token"));
    }

    #[tokio::test]
    async fn discovery_rejects_foreign_resource_document() {
        let server = MockServer::start().await;
        mock_discovery_docs(&server, "https://evil.example", &server.uri()).await;

        let err = discover_endpoints(&server.uri()).await.unwrap_err();
        assert!(err.contains("neither"), "unexpected error: {err}");
        assert!(err.contains("evil.example"), "unexpected error: {err}");
    }

    #[tokio::test]
    async fn discovery_rejects_non_https_issuer() {
        let server = MockServer::start().await;
        mock_discovery_docs(&server, &server.uri(), "http://evil.example").await;

        let err = discover_endpoints(&server.uri()).await.unwrap_err();
        assert!(err.contains("not https"), "unexpected error: {err}");
    }

    #[tokio::test]
    async fn discovery_failure_names_url_and_status() {
        let server = MockServer::start().await; // no mocks mounted → 404
        let err = discover_endpoints(&server.uri()).await.unwrap_err();
        assert!(
            err.contains("/.well-known/oauth-protected-resource"),
            "unexpected error: {err}"
        );
        assert!(err.contains("404"), "unexpected error: {err}");
    }

    /// Both halves cleared — most tests need discovery to actually run.
    fn clear_endpoint_override() {
        std::env::remove_var("HEDRA_AUTH_AUTHORIZE_URL");
        std::env::remove_var("HEDRA_AUTH_TOKEN_URL");
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn endpoint_override_skips_discovery_and_is_taken_verbatim() {
        clear_endpoint_override();
        // Paths deliberately unlike the vendor's `/oauth2/...` shape: the
        // override must not reconstruct anything.
        std::env::set_var(
            "HEDRA_AUTH_AUTHORIZE_URL",
            "https://auth.example/custom/authorize",
        );
        std::env::set_var(
            "HEDRA_AUTH_TOKEN_URL",
            "  https://auth.example/custom/token  ",
        );

        // The dead resource base proves no discovery request is attempted.
        let ep = resolve_auth_endpoints("test-cli", "http://127.0.0.1:9", false).unwrap();
        assert_eq!(
            ep.authorization_endpoint,
            "https://auth.example/custom/authorize"
        );
        assert_eq!(ep.token_endpoint, "https://auth.example/custom/token");
        assert_eq!(ep.resource, "http://127.0.0.1:9");

        clear_endpoint_override();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn endpoint_override_refuses_half_configuration() {
        clear_endpoint_override();
        std::env::set_var("HEDRA_AUTH_AUTHORIZE_URL", "https://auth.example/authorize");

        let err = resolve_auth_endpoints("test-cli", "http://127.0.0.1:9", false).unwrap_err();
        assert!(err.contains("must be set together"), "unexpected: {err}");

        // The other half alone is refused the same way.
        clear_endpoint_override();
        std::env::set_var("HEDRA_AUTH_TOKEN_URL", "https://auth.example/token");
        let err = resolve_auth_endpoints("test-cli", "http://127.0.0.1:9", false).unwrap_err();
        assert!(err.contains("must be set together"), "unexpected: {err}");

        clear_endpoint_override();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn endpoint_override_requires_https_off_loopback() {
        clear_endpoint_override();
        // The token leg carries the PKCE exchange — plaintext is refused.
        std::env::set_var("HEDRA_AUTH_AUTHORIZE_URL", "https://auth.example/authorize");
        std::env::set_var("HEDRA_AUTH_TOKEN_URL", "http://auth.example/token");
        let err = resolve_auth_endpoints("test-cli", "http://127.0.0.1:9", false).unwrap_err();
        assert!(err.contains("not https"), "unexpected: {err}");

        // Loopback stays usable so a local/mock server still works.
        std::env::set_var(
            "HEDRA_AUTH_AUTHORIZE_URL",
            "http://127.0.0.1:4444/authorize",
        );
        std::env::set_var("HEDRA_AUTH_TOKEN_URL", "http://localhost:4444/token");
        let ep = resolve_auth_endpoints("test-cli", "http://127.0.0.1:9", false).unwrap();
        assert_eq!(ep.token_endpoint, "http://localhost:4444/token");

        clear_endpoint_override();
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn refresh_path_uses_cached_endpoints_and_login_ignores_them() {
        clear_endpoint_override();
        let _store = fresh_keyring();
        let cached = AuthEndpoints {
            resource: "http://127.0.0.1:9".to_string(),
            authorization_endpoint: "http://cached/authorize".to_string(),
            token_endpoint: "http://cached/token".to_string(),
        };
        let _home = seed_discovery_cache(&cached);

        // use_cache=true (refresh path): served from the file, no network
        // (the dead resource base would error otherwise).
        let ep = resolve_auth_endpoints("test-cli", "http://127.0.0.1:9", true).unwrap();
        assert_eq!(ep, cached);

        // use_cache=false (login path): the cache is skipped, so the dead
        // resource base surfaces as a discovery failure.
        let err = resolve_auth_endpoints("test-cli", "http://127.0.0.1:9", false).unwrap_err();
        assert!(err.contains("discovery failed"), "unexpected error: {err}");

        // A cache for a different resource (env flip) is not trusted.
        let err = resolve_auth_endpoints("test-cli", "http://127.0.0.1:10", true).unwrap_err();
        assert!(err.contains("discovery failed"), "unexpected error: {err}");
    }

    // ── wire-level bootstrap tests (mock server + mock keyring) ─────────

    use fern_cli_sdk::auth::{KeyringStore, MockKeyringStore};
    use wiremock::matchers::{body_partial_json, body_string_contains, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    const JWT: &str = "test-jwt";

    async fn mock_login_plane() -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/workspaces"))
            .and(header("authorization", format!("Bearer {JWT}").as_str()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [
                    {"workspace_id": "w1", "workspace_name": "Personal", "role": "owner",
                     "workos_organization_id": null},
                    {"workspace_id": "w2", "workspace_name": "Acme", "role": "admin",
                     "workos_organization_id": "org_1"},
                ],
                "next_cursor": null,
            })))
            .mount(&server)
            .await;
        server
    }

    /// Installs the same `KeyAuth` projection production uses, so anything
    /// asserting on the active credential exercises the derivation from the
    /// workspace map rather than a raw item that nothing writes any more.
    /// The mock is returned so a test can still seed and inspect the slots
    /// underneath it.
    fn fresh_keyring() -> std::sync::Arc<MockKeyringStore> {
        super::super::active_key::projected_mock()
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn bootstrap_mints_and_stores_when_no_key_is_held() {
        let server = mock_login_plane().await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .and(header("authorization", format!("Bearer {JWT}").as_str()))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_1", "credential": "key_1:s3cret", "kind": "personal",
                "workspace_id": "w2", "workspace_name": "Acme", "organization_id": "org_1",
                "expires_at": "2026-08-19T00:00:00Z",
            })))
            .expect(1)
            .mount(&server)
            .await;
        let _store = fresh_keyring();

        bootstrap_inner("test-cli", &server.uri(), JWT)
            .await
            .unwrap();

        assert_eq!(
            active_store()
                .get("test-cli", KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_1:s3cret")
        );
        // The mint is recorded in the per-workspace key map, marked active.
        let map = workspaces::WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w2"));
        assert_eq!(map.keys["w2"].credential, "key_1:s3cret");
        assert_eq!(map.keys["w2"].workspace_name.as_deref(), Some("Acme"));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn bootstrap_renews_a_held_key_without_replacing_it() {
        let server = mock_login_plane().await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap/renew"))
            .and(header("authorization", "Bearer key_0:held"))
            .and(body_partial_json(serde_json::json!({"authkit_token": JWT})))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "key_id": "key_0", "expires_at": "2026-08-19T00:00:00Z", "workspace_id": "w1",
            })))
            .expect(1)
            .mount(&server)
            .await;
        let store = fresh_keyring();
        store.set("test-cli", KEY_SCHEME, "key_0:held").unwrap();

        bootstrap_inner("test-cli", &server.uri(), JWT)
            .await
            .unwrap();

        assert_eq!(
            active_store()
                .get("test-cli", KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_0:held"),
            "renewal must never touch the stored credential"
        );
        // The renewal lands in the key map too — credential is the held one.
        let map = workspaces::WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w1"));
        assert_eq!(map.keys["w1"].credential, "key_0:held");
        assert_eq!(
            map.keys["w1"].expires_at.as_deref(),
            Some("2026-08-19T00:00:00Z")
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn bootstrap_falls_back_to_mint_when_renewal_is_refused() {
        let server = mock_login_plane().await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap/renew"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "error": {"code": "PERMISSION_DENIED",
                          "message": "key is beyond its absolute age cap"},
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_2", "credential": "key_2:fresh", "kind": "personal",
                "workspace_id": "w1", "workspace_name": "Personal", "organization_id": null,
                "expires_at": "2026-08-19T00:00:00Z",
            })))
            .expect(1)
            .mount(&server)
            .await;
        let store = fresh_keyring();
        store.set("test-cli", KEY_SCHEME, "key_old:dead").unwrap();

        bootstrap_inner("test-cli", &server.uri(), JWT)
            .await
            .unwrap();

        assert_eq!(
            active_store()
                .get("test-cli", KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_2:fresh"),
            "refused renewal must be replaced by the freshly minted credential"
        );
    }

    // ── targeted mint (`workspaces select`) ────────────────────────────

    /// A live (unexpired) OAuth session plus a discovery cache pointing at
    /// `server` — enough for `oauth_apply` to authenticate without any
    /// network round-trip of its own.
    ///
    /// Returns the [`TempHome`] the endpoint cache was written into; the
    /// caller must hold it for the length of the test, or the directory is
    /// removed and `HOME` restored the moment it drops.
    #[must_use]
    fn seed_live_session(store: &MockKeyringStore, server_uri: &str) -> TempHome {
        let home = seed_discovery_cache(&AuthEndpoints {
            resource: resource_base_url().to_string(),
            authorization_endpoint: format!("{server_uri}/authorize"),
            token_endpoint: format!("{server_uri}/token"),
        });
        store
            .set(
                "test-cli",
                SCHEME,
                // expires in 2100 — the provider must not attempt a refresh
                r#"{"access_token":"live-token","refresh_token":"r1","expires_at":4102444800}"#,
            )
            .unwrap();
        home
    }

    /// Redirects the on-disk endpoint cache into a fresh temp directory and
    /// seeds it with `endpoints`.
    ///
    /// The cache is deliberately a plain file, so isolating it means moving
    /// `HOME` — process-global state, which is safe here only because every
    /// test that touches it is `#[serial]`. [`TempHome`] restores the
    /// previous value on drop.
    #[must_use]
    fn seed_discovery_cache(endpoints: &AuthEndpoints) -> TempHome {
        let home = TempHome::new();
        write_cached_endpoints("test-cli", endpoints);
        assert_eq!(
            read_cached_endpoints("test-cli").as_ref(),
            Some(endpoints),
            "the seed must be readable back, or the test is asserting on nothing"
        );
        home
    }

    /// Points `HOME` (and `XDG_CONFIG_HOME`, which the Linux branch prefers)
    /// at a temp directory, restoring both on drop. Holding one keeps a
    /// test off the developer's real config directory.
    struct TempHome {
        _dir: tempfile::TempDir,
        previous_home: Option<std::ffi::OsString>,
        previous_xdg: Option<std::ffi::OsString>,
    }

    impl TempHome {
        fn new() -> Self {
            let dir = tempfile::tempdir().expect("temp dir");
            let previous_home = std::env::var_os("HOME");
            let previous_xdg = std::env::var_os("XDG_CONFIG_HOME");
            std::env::set_var("HOME", dir.path());
            std::env::set_var("XDG_CONFIG_HOME", dir.path().join(".config"));
            Self {
                _dir: dir,
                previous_home,
                previous_xdg,
            }
        }
    }

    impl Drop for TempHome {
        fn drop(&mut self) {
            match &self.previous_home {
                Some(v) => std::env::set_var("HOME", v),
                None => std::env::remove_var("HOME"),
            }
            match &self.previous_xdg {
                Some(v) => std::env::set_var("XDG_CONFIG_HOME", v),
                None => std::env::remove_var("XDG_CONFIG_HOME"),
            }
        }
    }

    /// Every login-plane call mints a fresh JWT first, so a mint test needs
    /// the token leg mocked too.
    async fn mock_token_refresh(server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fresh-jwt", "refresh_token": "rotated",
                "token_type": "Bearer", "expires_in": 3600,
            })))
            .mount(server)
            .await;
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn targeted_mint_names_the_workspace_and_activates_it() {
        clear_endpoint_override();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .and(header("authorization", "Bearer fresh-jwt"))
            // The whole point: the target rides in the body, not in the login.
            .and(body_partial_json(serde_json::json!({"workspace_id": "w2"})))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_w2", "credential": "key_w2:s3cret", "kind": "personal",
                "workspace_id": "w2", "workspace_name": "Born Free", "organization_id": null,
                "expires_at": "2026-08-21T00:00:00Z",
            })))
            .expect(1)
            .mount(&server)
            .await;
        let store = fresh_keyring();
        let _home = seed_live_session(&store, &server.uri());
        mock_token_refresh(&server).await;

        let minted = mint_for_workspace_at("test-cli", &server.uri(), "w2").unwrap();

        assert_eq!(minted.key_id, "key_w2");
        assert_eq!(minted.workspace_name.as_deref(), Some("Born Free"));
        assert_eq!(
            active_store()
                .get("test-cli", KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_w2:s3cret"),
        );
        let map = workspaces::WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w2"));
        assert_eq!(map.keys["w2"].credential, "key_w2:s3cret");
    }

    /// The compatibility guard: a server that predates workspace-targeted
    /// minting ignores the
    /// unknown `workspace_id` field and mints for the JWT's own workspace.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn targeted_mint_refuses_a_key_that_landed_elsewhere() {
        clear_endpoint_override();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_w1", "credential": "key_w1:s3cret", "kind": "personal",
                // asked for w2, server bound w1 — the silent-drop shape
                "workspace_id": "w1", "workspace_name": "Personal", "organization_id": null,
                "expires_at": "2026-08-21T00:00:00Z",
            })))
            .mount(&server)
            .await;
        let store = fresh_keyring();
        let _home = seed_live_session(&store, &server.uri());
        mock_token_refresh(&server).await;
        store.set("test-cli", KEY_SCHEME, "key_held:stay").unwrap();

        let err = mint_for_workspace_at("test-cli", &server.uri(), "w2").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("w1") && msg.contains("w2"),
            "unexpected: {msg}"
        );
        assert!(
            msg.contains("does not support selecting a workspace at mint time"),
            "unexpected: {msg}"
        );

        // The active credential must NOT move to a key for another workspace.
        assert_eq!(
            active_store()
                .get("test-cli", KEY_SCHEME)
                .unwrap()
                .as_deref(),
            Some("key_held:stay"),
        );
        // …but the minted key is filed where it actually belongs, not orphaned.
        let map = workspaces::WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.keys["w1"].credential, "key_w1:s3cret");
        assert_eq!(map.active_workspace_id, None, "active marker must not move");
        assert!(!map.keys.contains_key("w2"));
    }

    #[test]
    #[serial_test::serial]
    fn has_oauth_session_reports_keyring_presence_only() {
        let store = fresh_keyring();
        assert!(!has_oauth_session("test-cli"));
        store.set("test-cli", SCHEME, "{}").unwrap();
        assert!(has_oauth_session("test-cli"));
    }

    // ── fresh_login_jwt (the login-plane auth path) ─────────────────────

    /// The regression that matters: the stored token is NOT expired, so the
    /// ordinary provider would reuse it — and the login plane would refuse it
    /// as "too old", since its `iat` gate is far shorter than a token's TTL.
    /// `fresh_login_jwt` must exchange the refresh token regardless.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn fresh_login_jwt_refreshes_even_an_unexpired_token() {
        clear_endpoint_override();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(body_string_contains("grant_type=refresh_token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "brand-new", "refresh_token": "rotated",
                "token_type": "Bearer", "expires_in": 3600,
            })))
            .expect(1)
            .mount(&server)
            .await;
        let store = fresh_keyring();
        let _home = seed_live_session(&store, &server.uri());

        let jwt = fresh_login_jwt("test-cli").unwrap();

        assert_eq!(
            jwt, "brand-new",
            "must not reuse the unexpired stored token"
        );
        // The rotated refresh token has to be persisted, or the NEXT call breaks.
        let stored: TokenBundle =
            serde_json::from_str(&store.get("test-cli", SCHEME).unwrap().unwrap()).unwrap();
        assert_eq!(stored.access_token, "brand-new");
        assert_eq!(stored.refresh_token.as_deref(), Some("rotated"));
    }

    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn fresh_login_jwt_without_a_session_says_log_in() {
        clear_endpoint_override();
        let _store = fresh_keyring();
        let _home = seed_discovery_cache(&AuthEndpoints {
            resource: resource_base_url().to_string(),
            authorization_endpoint: "https://example.invalid/a".to_string(),
            token_endpoint: "https://example.invalid/t".to_string(),
        });

        let err = fresh_login_jwt("test-cli").unwrap_err().to_string();
        assert!(err.contains("auth login"), "unexpected: {err}");
    }

    // ── the endpoint cache is a plain file, not a keyring item ──────────

    // It is two unauthenticated `.well-known` documents; keeping it in the
    // credential store cost a whole keychain item for nothing.
    #[test]
    #[serial_test::serial]
    fn discovery_cache_is_written_beside_the_credential_store() {
        let home = TempHome::new();
        let endpoints = AuthEndpoints {
            resource: "https://api.example.com".to_string(),
            authorization_endpoint: "https://auth.example.com/authorize".to_string(),
            token_endpoint: "https://auth.example.com/token".to_string(),
        };

        write_cached_endpoints("hedra-cli", &endpoints);

        let path = discovery_cache_path("hedra-cli").expect("HOME is set");
        assert!(path.exists(), "expected a cache file at {}", path.display());
        assert!(
            path.ends_with("hedra-cli/auth-endpoints.json"),
            "must sit in the CLI's own config dir, beside auth-keyring.json: {}",
            path.display()
        );
        assert_eq!(
            read_cached_endpoints("hedra-cli").as_ref(),
            Some(&endpoints)
        );
        drop(home);
    }

    // A `HEDRA_ENV` flip changes the resource base, and a cache discovered
    // for the other environment must not be served for this one.
    // Multi-thread flavor: `resolve_auth_endpoints` reaches discovery via
    // `block_in_place`, which panics on a current-thread runtime.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn a_cache_for_another_resource_base_is_not_served() {
        clear_endpoint_override();
        let _home = seed_discovery_cache(&AuthEndpoints {
            resource: "https://api.staging.hedra.com".to_string(),
            authorization_endpoint: "https://staging/authorize".to_string(),
            token_endpoint: "https://staging/token".to_string(),
        });

        // A dead resource base: reaching the network at all is the failure.
        let err = resolve_auth_endpoints("hedra-cli", "http://127.0.0.1:9", true)
            .expect_err("must reject the mismatched cache and try to discover");
        assert!(
            !err.is_empty(),
            "the mismatch must fall through to discovery, not silently serve staging"
        );
    }

    #[test]
    #[serial_test::serial]
    fn a_corrupt_cache_file_reads_as_absent() {
        let _home = TempHome::new();
        let path = discovery_cache_path("hedra-cli").expect("HOME is set");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "{ not json").unwrap();

        assert!(
            read_cached_endpoints("hedra-cli").is_none(),
            "a corrupt cache must degrade to re-discovery, not error"
        );
    }
}
