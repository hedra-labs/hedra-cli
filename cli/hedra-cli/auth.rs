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
//! That last point is why a login-time mint has a second leg. Left to the
//! server, an untargeted mint resolves the workspace from the JWT's
//! organization and refuses outright when that organization has none —
//! stranding an account whose only workspaces are org-less, even though
//! `workspaces select` would mint for one of them without a second login.
//! So a refusal of that specific shape retries against the listing this
//! flow already holds; see [`mint_for_a_listed_workspace`].
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

/// The resource base this invocation should actually use — the origin that
/// discovery, the RFC 8707 resource indicator, and every `/v3/...`
/// login-plane URL hang off.
///
/// [`resource_base_url`] is the compiled default for `HEDRA_ENV`. It is not
/// the whole answer, because `--base-url` / `HEDRA_CLI_BASE_URL` retarget
/// the data plane and the login plane has to follow. Without this, pointing
/// the CLI at a local stack sent generated commands there while workspace
/// listing and — worse — key *minting* silently went on talking to
/// production: a developer testing locally would create real production
/// credentials without ever being told.
///
/// The override names the data-plane base *including* the `/v3` prefix (it
/// replaces the spec's `https://api.hedra.com/v3` server wholesale), while
/// everything here wants the origin, so the suffix comes off.
pub(crate) fn resource_base() -> Result<String, CliError> {
    match std::env::var("HEDRA_CLI_BASE_URL") {
        Ok(raw) if !raw.trim().is_empty() => resource_base_from_override(raw.trim()),
        _ => Ok(resource_base_url().to_string()),
    }
}

/// Strip the `/v3` the data-plane override carries, so the login plane can
/// rebuild its own `/v3/...` paths from the same origin.
///
/// An override that does not carry it is rejected rather than guessed at.
/// The two planes would otherwise disagree about where `/v3` lives — the
/// data plane appending nothing, the login plane appending `/v3` — and the
/// failure would surface as a 404 from whichever deployment happened to
/// answer, long after the point where the mistake was made. Refusing before
/// any request is the only way the message can still name the cause.
pub(crate) fn resource_base_from_override(raw: &str) -> Result<String, CliError> {
    let trimmed = raw.trim_end_matches('/');
    trimmed
        .strip_suffix("/v3")
        .map(str::to_string)
        .ok_or_else(|| {
            CliError::Validation(format!(
                "base URL `{raw}` does not end in `/v3`. The CLI's base URL names the \
                 data-plane root, which carries that prefix (as the default \
                 `https://api.hedra.com/v3` does); the login plane derives its own \
                 endpoints from the same origin and cannot do so without it. Pass \
                 `{trimmed}/v3` instead."
            ))
        })
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
    let base = resource_base()?;
    ENDPOINTS
        .get_or_init(|| resolve_auth_endpoints(cli_name, &base, use_cache))
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
            require_https("authorization endpoint", &authorization_endpoint)?;
            require_https("token endpoint", &token_endpoint)?;
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
    require_https("authorization server", &issuer).map_err(|e| format!("{url}: {e}"))?;

    // Hop 2 — RFC 8414: the issuer names its endpoints. Taken verbatim —
    // reconstructing `{issuer}/oauth2/…` would re-hardcode the vendor's
    // path shape, the very thing this chain exists to avoid.
    let meta_url = metadata_url(&issuer)?;
    let meta = fetch_json(&http, &meta_url).await?;

    // RFC 8414 § 3.3: the metadata's own `issuer` MUST match the issuer used
    // to fetch it. Without the check, a document served at one issuer can
    // name another, and the CLI would carry an authorization code to
    // whatever token endpoint that second party published.
    let declared = meta
        .get("issuer")
        .and_then(Value::as_str)
        .ok_or_else(|| format!("{meta_url}: metadata has no `issuer`"))?;
    if declared.trim_end_matches('/') != issuer {
        return Err(format!(
            "{meta_url}: metadata declares issuer `{declared}` but was served from \
             `{issuer}` — refusing a document that does not describe the \
             authorization server it came from"
        ));
    }

    let endpoint = |key: &str| -> Result<String, String> {
        let value = meta
            .get(key)
            .and_then(Value::as_str)
            .map(str::to_string)
            .ok_or_else(|| format!("{meta_url}: metadata has no `{key}`"))?;
        // The token leg carries the PKCE code exchange and the authorization
        // leg carries the code itself, so neither may be plaintext however
        // the document asks for it.
        require_https(key, &value).map_err(|e| format!("{meta_url}: {e}"))?;
        Ok(value)
    };
    Ok(AuthEndpoints {
        resource: resource_base_url.to_string(),
        authorization_endpoint: endpoint("authorization_endpoint")?,
        token_endpoint: endpoint("token_endpoint")?,
    })
}

/// The RFC 8414 § 3.1 metadata URL for `issuer`.
///
/// The well-known segment goes *between the host and the issuer's path*, not
/// on the end of the whole issuer:
///
/// ```text
///   https://host          → https://host/.well-known/oauth-authorization-server
///   https://host/tenant1  → https://host/.well-known/oauth-authorization-server/tenant1
/// ```
///
/// Appending — the old behaviour — happens to work only because today's
/// issuer is origin-only. The moment an authorization server is served under
/// a path, which a per-tenant issuer usually is, the appended form 404s and
/// the CLI cannot log in at all until a new binary ships. Building it
/// correctly costs nothing and takes a release off that recovery path.
fn metadata_url(issuer: &str) -> Result<String, String> {
    let url = reqwest::Url::parse(issuer)
        .map_err(|e| format!("authorization server `{issuer}` is not a valid URL: {e}"))?;
    if url.query().is_some() || url.fragment().is_some() {
        return Err(format!(
            "authorization server `{issuer}` carries a query or fragment, which \
             RFC 8414 forbids in an issuer identifier"
        ));
    }
    let issuer_path = url.path().trim_end_matches('/');
    let mut meta = url.clone();
    meta.set_path(&format!(
        "/.well-known/oauth-authorization-server{issuer_path}"
    ));
    Ok(meta.to_string())
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
fn require_https(what: &str, url: &str) -> Result<(), String> {
    if url.starts_with("https://") {
        return Ok(());
    }
    let loopback = url.strip_prefix("http://").is_some_and(|rest| {
        let host = rest.split(['/', ':']).next().unwrap_or("");
        matches!(host, "127.0.0.1" | "localhost")
    });
    if loopback {
        Ok(())
    } else {
        Err(format!("{what} `{url}` is not https"))
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
/// "There is no login session", phrased for what the user actually holds.
///
/// A flat "Not logged in" is false for anyone who pasted an API key: they are
/// authenticated — `auth status` shows the key active and every generated
/// command works — and it sends them hunting for a problem they do not have.
/// What they lack is a *user* identity, which only the login plane carries.
fn no_login_session(cli_name: &str) -> String {
    let has_key = std::env::var_os("HEDRA_API_KEY").is_some()
        || matches!(active_store().get(cli_name, KEY_SCHEME), Ok(Some(_)));
    if has_key {
        format!(
            "An API key identifies a workspace, not a person, so workspace commands \
             need a browser login. Run `{cli_name} auth login` — your key still works \
             for everything else."
        )
    } else {
        format!("Not logged in. Run `{cli_name} auth login` to authenticate.")
    }
}

/// The rotated refresh token is written back: the identity provider
/// rotates it on every
/// exchange, so dropping the new one would break the *next* call.
pub(crate) fn fresh_login_jwt(cli_name: &str) -> Result<String, CliError> {
    let endpoints =
        resolve_auth_endpoints(cli_name, &resource_base()?, true).map_err(CliError::Auth)?;
    let raw = active_store()
        .get(cli_name, SCHEME)?
        .ok_or_else(|| CliError::Auth(no_login_session(cli_name)))?;
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

    // The persist is NOT best-effort. The identity provider rotates the
    // refresh token on every exchange, so the moment this grant succeeds the
    // token still in the keyring is dead. Dropping the replacement — the old
    // behaviour, on the reasoning that "the call at hand already has its
    // token, only the next call pays" — silently destroys the session: the
    // next login-plane call presents an invalidated token and is refused,
    // and the user is sent back through a browser login for no visible
    // reason.
    //
    // Failing here instead costs the current command, which the user can
    // retry, rather than the session.
    let json = refreshed.to_keyring_value().map_err(|e| {
        CliError::Auth(format!(
            "could not serialize the refreshed session: {e}. The previous refresh \
             token has already been consumed — run `{cli_name} auth login` again."
        ))
    })?;
    active_store().set(cli_name, SCHEME, &json).map_err(|e| {
        CliError::Auth(format!(
            "the refreshed session could not be saved: {e}. The previous refresh \
             token has already been consumed, so the stored session is now stale — \
             fix the credential store and run `{cli_name} auth login` again."
        ))
    })?;
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
    let api_base = resource_base()?;
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
        tokio::runtime::Handle::current().block_on(bootstrap_inner(cli_name, &api_base, &jwt))
    })
}

async fn bootstrap_inner(cli_name: &str, api_base: &str, jwt: &str) -> Result<(), CliError> {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CliError::Auth(format!("could not build HTTP client: {e}")))?;

    // 1. The picker listing, authenticated with the login JWT. It renders
    //    the summary at the end, and it is also what a mint refused for
    //    lack of an organization workspace falls back to — which is why it
    //    is fetched up front rather than after the key is settled.
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
                )?;
                (
                    renewed.key_id,
                    Some(renewed.expires_at),
                    renewed.workspace_id,
                    false,
                )
            }
            None => mint_and_store(&http, api_base, jwt, cli_name, &listing).await?,
        },
        None => mint_and_store(&http, api_base, jwt, cli_name, &listing).await?,
    };

    // 3. Render: the mint/renew response's workspace is the authoritative
    //    selection (the JWT's org is the sole selector server-side). The key
    //    line comes first — it is what the login was for — and the workspace
    //    picture follows it.
    let map = workspaces::WorkspaceKeyMap::load(cli_name);
    let expiry = expires_at
        .as_deref()
        .map(describe_expiry)
        .unwrap_or_else(|| "no expiry reported".to_string());
    eprintln!(
        "API key {key_id} {} — {expiry}.",
        if minted { "minted" } else { "renewed" }
    );
    eprint!(
        "{}",
        workspaces::render_login_summary(&listing, workspace_id.as_deref(), &map.keys)
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

/// The distinguishing fragment of the login plane's org-has-no-workspace
/// refusal — the one mint refusal a workspace-targeted retry can clear.
///
/// Matching prose is not the first choice, but the v3 envelope offers
/// nothing finer: every 403 on this route carries `PERMISSION_DENIED`
/// (`ApiAccessForbiddenError`), so the membership, key-cap and
/// no-linked-user refusals are indistinguishable from this one by code.
///
/// The failure mode is deliberately one-sided. A reworded server message
/// stops the retry engaging and the user gets exactly today's error — a
/// safe degradation. Only a *false* match would be harmful, by retrying a
/// refusal that a different workspace must not paper over (a key cap, most
/// of all, where a retry would bill a workspace the login never named), and
/// no other refusal on this route can plausibly contain this phrase.
const ORG_HAS_NO_WORKSPACE: &str = "has no Hedra workspace";

/// A mint the server answered and refused, with the status still attached.
///
/// [`post_bootstrap_mint`] flattens this into a `CliError` for callers that
/// only report it; the login bootstrap needs the status and message intact
/// to tell the one recoverable refusal from every other.
struct MintRefusal {
    status: reqwest::StatusCode,
    /// Already rendered by [`login_plane_error`], so becoming the error
    /// message re-derives nothing.
    line: String,
}

impl MintRefusal {
    fn is_org_without_workspace(&self) -> bool {
        self.status == reqwest::StatusCode::FORBIDDEN && self.line.contains(ORG_HAS_NO_WORKSPACE)
    }

    fn into_error(self) -> CliError {
        CliError::Auth(format!("key mint refused — {}", self.line))
    }
}

/// Either the server refused, or something else went wrong entirely
/// (transport, or a success body that would not parse). Only the first is
/// worth inspecting; the second is reported as-is.
enum MintError {
    Refused(MintRefusal),
    Other(CliError),
}

impl From<MintError> for CliError {
    fn from(e: MintError) -> Self {
        match e {
            MintError::Refused(refusal) => refusal.into_error(),
            MintError::Other(err) => err,
        }
    }
}

/// POST the mint and parse the response, keeping a refusal inspectable.
/// The builder must already carry a login credential (the login-fresh JWT,
/// or the stored OAuth session via [`fresh_login_jwt`]). `workspace_id`
/// names the target workspace when the caller has one; omitted, the server
/// resolves one from the JWT's organization.
async fn try_bootstrap_mint(
    req: reqwest::RequestBuilder,
    workspace_id: Option<&str>,
) -> Result<BootstrapMintResponse, MintError> {
    let mut body = serde_json::json!({ "name": device_name() });
    if let Some(ws) = workspace_id {
        body["workspace_id"] = Value::String(ws.to_string());
    }
    let resp = req.json(&body).send().await.map_err(|e| {
        MintError::Other(CliError::Auth(format!(
            "POST /v3/keys/bootstrap failed: {e}"
        )))
    })?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(MintError::Refused(MintRefusal {
            status,
            line: login_plane_error(status, &text),
        }));
    }
    serde_json::from_str(&text)
        .map_err(|e| MintError::Other(CliError::Auth(format!("unexpected mint response: {e}"))))
}

/// [`try_bootstrap_mint`] for callers that only report a refusal.
async fn post_bootstrap_mint(
    req: reqwest::RequestBuilder,
    workspace_id: Option<&str>,
) -> Result<BootstrapMintResponse, CliError> {
    try_bootstrap_mint(req, workspace_id)
        .await
        .map_err(CliError::from)
}

/// The login's organization named no workspace — mint for one the account
/// can actually reach, taken from the listing fetched moments ago.
///
/// Two server guarantees make this a retry rather than a second login.
/// The mint accepts an explicit `workspace_id` that bypasses the org claim
/// entirely (ENG-10403), and a refusal deliberately does NOT consume the
/// single-use login JWT — the plane spends it "only once every refusal path
/// is behind us" — so the very same token carries the second attempt, with
/// no browser round-trip and no refresh-token rotation.
///
/// Without this, a login whose identity-provider organization has no linked
/// workspace dead-ends on `auth login` while `workspaces select` would mint
/// for the same account happily: the CLI would be refusing to do at login
/// what it offers as its next command.
///
/// `refusal` is threaded through so that a case this cannot resolve reports
/// what the *server* said, not a paraphrase of it.
async fn mint_for_a_listed_workspace(
    http: &reqwest::Client,
    api_base: &str,
    jwt: &str,
    cli_name: &str,
    listing: &[workspaces::WorkspaceSummary],
    refusal: MintRefusal,
) -> Result<BootstrapMintResponse, CliError> {
    let (workspace_id, workspace_name) = match workspaces::fallback_mint_target(cli_name, listing) {
        workspaces::FallbackTarget::One {
            workspace_id,
            workspace_name,
        } => (workspace_id, workspace_name),
        // Nothing to fall back to: the refusal is the whole truth, so
        // it is what the user sees.
        workspaces::FallbackTarget::Nothing => return Err(refusal.into_error()),
        workspaces::FallbackTarget::Ambiguous(candidates) => {
            return Err(CliError::Auth(format!(
                "key mint refused — {}\n\
                     This account can reach {} workspaces and none is selected, so the \
                     CLI will not pick one for you — the key would bind this device's \
                     billing to a workspace you never named:\n  {}\n\
                     Run `hedra-cli workspaces select --workspace-id <id>` to mint for \
                     one of them; no second login is needed.",
                refusal.line,
                candidates.len(),
                candidates.join("\n  "),
            )));
        }
    };
    eprintln!(
        "(this login's organization names no workspace — minting for \
         \"{workspace_name}\" ({workspace_id}) instead)"
    );
    post_bootstrap_mint(
        http.post(format!("{api_base}/v3/keys/bootstrap"))
            .bearer_auth(jwt),
        Some(&workspace_id),
    )
    .await
}

async fn mint_and_store(
    http: &reqwest::Client,
    api_base: &str,
    jwt: &str,
    cli_name: &str,
    listing: &[workspaces::WorkspaceSummary],
) -> Result<(String, Option<String>, Option<String>, bool), CliError> {
    let req = http
        .post(format!("{api_base}/v3/keys/bootstrap"))
        .bearer_auth(jwt);
    // The untargeted mint stays the primary path: when the login's
    // organization does name a workspace, that IS the user's selection,
    // expressed through the identity provider, and naming one locally would
    // override it. Only the refusal that leaves the login with nothing at
    // all falls through to the listing.
    let minted = match try_bootstrap_mint(req, None).await {
        Ok(minted) => minted,
        Err(MintError::Refused(refusal)) if refusal.is_org_without_workspace() => {
            mint_for_a_listed_workspace(http, api_base, jwt, cli_name, listing, refusal).await?
        }
        Err(other) => return Err(other.into()),
    };
    record_workspace_key(
        cli_name,
        minted.workspace_id.as_deref(),
        &minted.key_id,
        &minted.credential,
        minted.workspace_name.as_deref(),
        minted.expires_at.as_deref(),
        true,
    )?;
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

/// Mint a key bound to `workspace_id` and make it the active credential.
/// Runs off the stored OAuth session, so switching workspaces costs no
/// browser round-trip; the login JWT's own org is irrelevant here.
///
/// `api_base` is the caller's resolved resource base, not the compiled
/// default: minting is the one login-plane call that *creates* state, so a
/// `--base-url` pointing at a local stack must not quietly mint a real
/// production key.
///
/// `jwt` must be login-plane fresh. There is deliberately no wrapper that
/// mints one internally: the only caller already holds a fresh token for
/// its listing call, and a convenience overload would make it too easy to
/// reintroduce the second rotation this signature exists to prevent.
pub(crate) fn mint_for_workspace_at(
    cli_name: &str,
    api_base: &str,
    jwt: &str,
    workspace_id: &str,
) -> Result<MintedKey, CliError> {
    let http = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| CliError::Auth(format!("could not build HTTP client: {e}")))?;
    // `jwt` must be login-plane fresh — minted seconds ago, not merely
    // unexpired (see `fresh_login_jwt`). Taking it as a parameter rather
    // than minting one here lets a caller that already refreshed reuse it:
    // each refresh rotates the token, so two in one command is one rotation
    // too many.
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
        // This arm returns an error either way, so the persistence result is
        // not propagated with `?` — that would replace the explanation of
        // *why* the mint was refused with a storage message, and the
        // workspace mismatch is the more useful half. Instead both outcomes
        // are reported: whether the key was salvaged, or is now orphaned.
        let fate = match record_workspace_key(
            cli_name,
            landed,
            &minted.key_id,
            &minted.credential,
            minted.workspace_name.as_deref(),
            minted.expires_at.as_deref(),
            false,
        ) {
            Ok(()) => format!(
                "The key was kept for {}; the active workspace is unchanged.",
                landed.unwrap_or("that workspace")
            ),
            Err(e) => format!(
                "It could not be saved locally either, so it is now orphaned — \
                 revoke key {} in the dashboard. ({e})",
                minted.key_id
            ),
        };
        return Err(CliError::Auth(format!(
            "the mint bound its key to workspace {} instead of the requested {workspace_id} — \
             this environment does not support selecting a workspace at mint time yet. {fate}",
            landed.unwrap_or("<none>"),
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
    )?;
    Ok(MintedKey {
        key_id: minted.key_id,
        workspace_name: minted.workspace_name,
        expires_at: minted.expires_at,
    })
}

/// Persist a minted or renewed key into the per-workspace key map.
///
/// This used to swallow the write error on the grounds that "the credential
/// is already safe in `KeyAuth`". That stopped being true when the `KeyAuth`
/// slot became a projection of this very map: the map is now the *only*
/// copy. Swallowing the failure meant the CLI could create a key
/// server-side, drop the sole copy of its secret, and print "minted and
/// stored in the keyring" — leaving a live credential on the account that
/// nobody can see, use, or revoke through this CLI.
///
/// So the error propagates. A caller that cannot persist a minted secret
/// has not succeeded and must not say it has.
#[allow(clippy::too_many_arguments)]
fn record_workspace_key(
    cli_name: &str,
    workspace_id: Option<&str>,
    key_id: &str,
    credential: &str,
    workspace_name: Option<&str>,
    expires_at: Option<&str>,
    activate: bool,
) -> Result<(), CliError> {
    workspaces::record_key(
        cli_name,
        workspace_id,
        key_id,
        credential,
        workspace_name,
        expires_at,
        activate,
    )
    .map_err(|e| {
        CliError::Auth(format!(
            "API key {key_id} was created but could not be saved locally: {e}. \
             The key exists on your account and this CLI no longer holds its \
             secret — revoke it in the dashboard, resolve the credential-store \
             problem, and run the command again."
        ))
    })
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

/// Format a Unix instant in the device's locale and timezone.
///
/// `%x %X %Z` is the locale's own date and time plus the zone —
/// `setlocale(LC_TIME, "")` adopts whatever `LC_ALL` / `LC_TIME` / `LANG`
/// say, and `localtime_r` applies `TZ`. The zone is appended explicitly
/// because most locales' date and time formats omit it, and an expiry
/// without one is ambiguous.
///
/// Unix only. `std` has no locale-aware formatter, and `chrono` is not
/// available to reach for: it is declared `optional` and enabled by no
/// feature purely to pin the lock closure for the generated crates (see the
/// note in Cargo.toml), and Cargo.toml is generator-owned, so enabling it
/// here would not survive regeneration. Other platforms return `None` and
/// the caller falls back to the server's ISO instant.
#[cfg(unix)]
fn format_local(epoch: i64) -> Option<String> {
    use std::sync::Once;
    static LOCALE: Once = Once::new();
    // SAFETY: `setlocale` mutates process-global locale state. Run once,
    // before any `strftime` below, from the login print path.
    LOCALE.call_once(|| unsafe {
        libc::setlocale(libc::LC_TIME, c"".as_ptr());
    });

    let t = epoch as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    // SAFETY: `t` and `tm` are valid, distinct, aligned locals; `localtime_r`
    // fills `tm` and returns null on failure (e.g. `epoch` out of range for
    // a 32-bit `time_t`).
    if unsafe { libc::localtime_r(&t, &mut tm) }.is_null() {
        return None;
    }

    let mut buf = [0u8; 128];
    // SAFETY: `buf` is writable for the length passed, the format is
    // NUL-terminated, and `tm` was just filled by `localtime_r`.
    let written = unsafe {
        libc::strftime(
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            c"%x %X %Z".as_ptr(),
            &tm,
        )
    };
    if written == 0 {
        return None;
    }
    Some(String::from_utf8_lossy(&buf[..written]).into_owned())
}

#[cfg(not(unix))]
fn format_local(_epoch: i64) -> Option<String> {
    None
}

/// "expires 08/21/2026 17:58:00 PDT (in 23h 58m)" — the absolute instant in
/// the device's locale and timezone, with the relative form to disambiguate.
/// Falls back to the server's ISO string when the instant will not parse or
/// the platform has no locale formatter.
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
    let when = format_local(expiry_epoch).unwrap_or_else(|| iso.to_string());
    if delta >= 0 {
        format!("expires {when} (in {})", human(delta))
    } else {
        format!("expired {when} ({} ago)", human(-delta))
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
pub(crate) mod tests {
    use super::*;

    #[test]
    fn describe_expiry_is_locale_absolute_plus_relative() {
        // Both instants sit inside a 32-bit `time_t`, so the localized branch
        // is reachable on every target rather than silently falling back.
        let future = describe_expiry("2030-01-01T00:00:00Z");
        assert!(future.starts_with("expires "), "{future}");
        assert!(future.contains("(in "), "{future}");

        let past = describe_expiry("2000-01-01T00:00:00Z");
        assert!(past.starts_with("expired "), "{past}");
        assert!(past.ends_with(" ago)"), "{past}");

        // Locale output is whatever the machine says, so assert on the one
        // thing that must be true: the raw ISO instant was replaced.
        #[cfg(unix)]
        assert!(
            !future.contains("2030-01-01T00:00:00Z"),
            "not localized: {future}"
        );

        // An instant that will not parse falls back to the server's string.
        assert_eq!(describe_expiry("whenever"), "expires whenever");
    }

    #[cfg(unix)]
    #[test]
    fn format_local_renders_a_known_instant() {
        // 2026-08-19T00:00:00Z, the epoch pinned by the test below. Every
        // locale renders the year in some form, and no locale renders an
        // empty string, but the exact layout is the machine's business.
        let out = format_local(1787097600).expect("localtime_r + strftime available");
        assert!(!out.trim().is_empty(), "empty rendering");
        assert!(out.contains("26"), "year missing from {out}");
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
    fn override_to_resource_base_strips_the_v3_prefix() {
        assert_eq!(
            resource_base_from_override("http://localhost:8000/v3").unwrap(),
            "http://localhost:8000"
        );
        // A trailing slash is not a different deployment.
        assert_eq!(
            resource_base_from_override("https://api.staging.hedra.com/v3/").unwrap(),
            "https://api.staging.hedra.com"
        );
    }

    #[test]
    fn an_override_without_v3_is_refused_before_any_request() {
        let err = resource_base_from_override("http://localhost:8000").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("/v3"), "unexpected: {msg}");
        // The message has to carry the fix, since the failure would
        // otherwise surface as a 404 far from its cause.
        assert!(
            msg.contains("http://localhost:8000/v3"),
            "the error must name the corrected URL: {msg}"
        );
    }

    #[test]
    #[serial_test::serial]
    fn the_login_plane_follows_the_data_plane_override() {
        let restore = std::env::var("HEDRA_CLI_BASE_URL").ok();

        std::env::set_var("HEDRA_CLI_BASE_URL", "http://localhost:8000/v3");
        assert_eq!(
            resource_base().unwrap(),
            "http://localhost:8000",
            "a custom data-plane base must retarget the login plane too, or a \
             local test run mints production keys"
        );

        std::env::remove_var("HEDRA_CLI_BASE_URL");
        assert_eq!(resource_base().unwrap(), resource_base_url());

        match restore {
            Some(v) => std::env::set_var("HEDRA_CLI_BASE_URL", v),
            None => std::env::remove_var("HEDRA_CLI_BASE_URL"),
        }
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

    // RFC 8414 § 3.1: the well-known segment goes between host and path.
    #[test]
    fn metadata_url_inserts_the_well_known_segment_before_the_issuer_path() {
        assert_eq!(
            metadata_url("https://auth.example.com").unwrap(),
            "https://auth.example.com/.well-known/oauth-authorization-server"
        );
        assert_eq!(
            metadata_url("https://auth.example.com/tenant1").unwrap(),
            "https://auth.example.com/.well-known/oauth-authorization-server/tenant1"
        );
        // A trailing slash is not an extra path segment.
        assert_eq!(
            metadata_url("https://auth.example.com/tenant1/").unwrap(),
            "https://auth.example.com/.well-known/oauth-authorization-server/tenant1"
        );
    }

    #[test]
    fn metadata_url_refuses_an_issuer_with_a_query_or_fragment() {
        assert!(metadata_url("https://auth.example.com/t?x=1").is_err());
        assert!(metadata_url("https://auth.example.com/t#frag").is_err());
    }

    /// The path-based issuer end to end — the shape a per-tenant
    /// authorization server takes, and the one the appended form 404s on.
    #[tokio::test]
    async fn discovery_follows_an_issuer_that_has_a_path() {
        let server = MockServer::start().await;
        let uri = server.uri();
        let issuer = format!("{uri}/tenant1");
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-protected-resource"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "resource": uri,
                "authorization_servers": [issuer],
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-authorization-server/tenant1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": issuer,
                "authorization_endpoint": format!("{issuer}/authorize"),
                "token_endpoint": format!("{issuer}/token"),
            })))
            .mount(&server)
            .await;

        let ep = discover_endpoints(&uri).await.unwrap();
        assert_eq!(ep.token_endpoint, format!("{issuer}/token"));
    }

    /// A document that names an issuer other than the one that served it
    /// must be refused: honouring it would send an authorization code to a
    /// token endpoint published by a third party.
    #[tokio::test]
    async fn discovery_refuses_metadata_whose_issuer_does_not_match() {
        let server = MockServer::start().await;
        let uri = server.uri();
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-protected-resource"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "resource": uri,
                "authorization_servers": [uri],
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-authorization-server"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": "https://somewhere-else.example.com",
                "authorization_endpoint": "https://somewhere-else.example.com/authorize",
                "token_endpoint": "https://somewhere-else.example.com/token",
            })))
            .mount(&server)
            .await;

        let err = discover_endpoints(&uri).await.unwrap_err();
        assert!(
            err.contains("somewhere-else.example.com") && err.contains("issuer"),
            "unexpected: {err}"
        );
    }

    /// The token leg carries the PKCE code exchange, so a plaintext endpoint
    /// is refused however the document asks for it.
    #[tokio::test]
    async fn discovery_refuses_an_insecure_returned_endpoint() {
        let server = MockServer::start().await;
        let uri = server.uri();
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-protected-resource"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "resource": uri,
                "authorization_servers": [uri],
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/.well-known/oauth-authorization-server"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "issuer": uri,
                "authorization_endpoint": format!("{uri}/authorize"),
                // Not loopback, and not https.
                "token_endpoint": "http://tokens.example.com/token",
            })))
            .mount(&server)
            .await;

        let err = discover_endpoints(&uri).await.unwrap_err();
        assert!(
            err.contains("token_endpoint") && err.contains("not https"),
            "unexpected: {err}"
        );
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
    pub(crate) fn clear_endpoint_override() {
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

    /// A store whose writes to one slot always fail — a locked keychain, a
    /// full or read-only config directory. Reads and every other slot behave
    /// normally, so a test can set up state and then fail only the write
    /// under examination.
    ///
    /// Local to this crate rather than added to `MockKeyringStore`: that
    /// type lives in generator-owned `src/`, and a test fixture is not worth
    /// a second Fern Replay patch to re-apply after every regeneration.
    #[derive(Debug)]
    struct FailingWrites {
        inner: std::sync::Arc<MockKeyringStore>,
        slot: &'static str,
    }

    impl KeyringStore for FailingWrites {
        fn get(&self, service: &str, account: &str) -> Result<Option<String>, CliError> {
            self.inner.get(service, account)
        }
        fn set(&self, service: &str, account: &str, value: &str) -> Result<(), CliError> {
            if account == self.slot {
                return Err(CliError::Auth(format!(
                    "credential store is locked ({slot})",
                    slot = self.slot
                )));
            }
            self.inner.set(service, account, value)
        }
        fn delete(&self, service: &str, account: &str) -> Result<(), CliError> {
            self.inner.delete(service, account)
        }
        fn backend_label(&self) -> String {
            "failing mock".to_string()
        }
    }

    /// The projection, over a store that cannot persist the workspace map.
    fn keyring_that_cannot_save_keys() -> std::sync::Arc<MockKeyringStore> {
        let mock = std::sync::Arc::new(MockKeyringStore::new());
        fern_cli_sdk::auth::set_active_store(super::super::active_key::project(
            std::sync::Arc::new(FailingWrites {
                inner: mock.clone(),
                slot: workspaces::WORKSPACE_KEYS_SCHEME,
            }),
        ));
        mock
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

    // ── login-time fallback: the org names no workspace ─────────────────

    /// A login plane whose listing is exactly `data`.
    async fn mock_login_plane_listing(data: serde_json::Value) -> MockServer {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/workspaces"))
            .and(header("authorization", format!("Bearer {JWT}").as_str()))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": data, "next_cursor": null})),
            )
            .mount(&server)
            .await;
        server
    }

    /// The login plane's refusal, verbatim: the CLI's recovery hinges on
    /// recognizing *this* message, so a test that paraphrased it would pass
    /// while the product failed.
    fn org_without_workspace() -> ResponseTemplate {
        ResponseTemplate::new(403).set_body_json(serde_json::json!({
            "error": {
                "code": "PERMISSION_DENIED",
                "message": "This login's organization has no Hedra workspace. List your \
                            workspaces with GET /v3/workspaces and log in again for one of them.",
            },
        }))
    }

    /// Two mints differ only by whether the body names a workspace, and
    /// "carries the field at all" is not something `body_partial_json` can
    /// say — so the two mocks on this path are told apart by a closure.
    fn names_a_workspace(req: &wiremock::Request) -> bool {
        req.body_json::<Value>()
            .ok()
            .is_some_and(|b| b.get("workspace_id").is_some())
    }

    fn names_no_workspace(req: &wiremock::Request) -> bool {
        !names_a_workspace(req)
    }

    /// The reported bug: the login's identity-provider organization names no
    /// Hedra workspace, so the untargeted mint is refused — while the
    /// listing fetched moments earlier shows a workspace the account owns.
    /// The login must mint for it instead of dead-ending, because
    /// `workspaces select` would have minted for that very workspace.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn bootstrap_mints_for_the_only_listed_workspace_when_the_org_names_none() {
        let server = mock_login_plane_listing(serde_json::json!([
            {"workspace_id": "w9", "workspace_name": "My Workspace", "role": "owner",
             "workos_organization_id": null},
        ]))
        .await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .and(names_no_workspace)
            .respond_with(org_without_workspace())
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .and(names_a_workspace)
            // The same login JWT carries the retry: a refusal does not
            // consume it, so no second browser round-trip is needed.
            .and(header("authorization", format!("Bearer {JWT}").as_str()))
            .and(body_partial_json(serde_json::json!({"workspace_id": "w9"})))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_w9", "credential": "key_w9:s3cret", "kind": "personal",
                "workspace_id": "w9", "workspace_name": "My Workspace",
                "organization_id": null, "expires_at": "2026-08-19T00:00:00Z",
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
            Some("key_w9:s3cret"),
            "the fallback mint's key must become the active credential"
        );
        let map = workspaces::WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w9"));
        assert_eq!(map.keys["w9"].credential, "key_w9:s3cret");
    }

    /// With several reachable workspaces, an earlier `workspaces select`
    /// is the tie-break — a login must not silently move the active key to
    /// a different workspace than the one the user chose.
    ///
    /// This is the reported failure end to end: a held key that no longer
    /// renews, then an untargeted mint the login's organization cannot
    /// satisfy.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn bootstrap_fallback_honors_the_previously_selected_workspace() {
        let server = mock_login_plane().await; // lists w1 and w2
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap/renew"))
            .respond_with(ResponseTemplate::new(401).set_body_json(serde_json::json!({
                "error": {"code": "UNAUTHORIZED", "message": "Invalid API key."},
            })))
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .and(names_no_workspace)
            .respond_with(org_without_workspace())
            .expect(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .and(body_partial_json(serde_json::json!({"workspace_id": "w2"})))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_w2", "credential": "key_w2:fresh", "kind": "personal",
                "workspace_id": "w2", "workspace_name": "Acme",
                "organization_id": null, "expires_at": "2026-08-19T00:00:00Z",
            })))
            .expect(1)
            .mount(&server)
            .await;
        let _store = fresh_keyring();
        workspaces::record_key(
            "test-cli",
            Some("w2"),
            "key_old",
            "key_old:stale",
            Some("Acme"),
            None,
            true,
        )
        .unwrap();

        bootstrap_inner("test-cli", &server.uri(), JWT)
            .await
            .unwrap();

        let map = workspaces::WorkspaceKeyMap::load("test-cli");
        assert_eq!(map.active_workspace_id.as_deref(), Some("w2"));
        assert_eq!(map.keys["w2"].credential, "key_w2:fresh");
    }

    /// Several reachable workspaces and nothing selected: the CLI must not
    /// guess. A key binds what it bills, so the refusal is reported with the
    /// candidates and the command that resolves it.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn bootstrap_fallback_refuses_to_guess_between_several_workspaces() {
        let server = mock_login_plane().await; // lists w1 and w2
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .respond_with(org_without_workspace())
            // Exactly one attempt: with no defensible target there is
            // nothing to retry.
            .expect(1)
            .mount(&server)
            .await;
        let _store = fresh_keyring();

        let msg = bootstrap_inner("test-cli", &server.uri(), JWT)
            .await
            .unwrap_err()
            .to_string();

        assert!(
            msg.contains("w1") && msg.contains("w2"),
            "the candidates must be named: {msg}"
        );
        // Spelled the way the command actually parses. `select` takes a
        // named flag, so a bare-positional hint would be a command the user
        // copies and watches fail.
        assert!(
            msg.contains("workspaces select --workspace-id"),
            "and the runnable command that resolves it: {msg}"
        );
        assert!(
            msg.contains("has no Hedra workspace"),
            "without losing what the server actually said: {msg}"
        );
    }

    /// The retry is scoped to the one refusal a different workspace can
    /// clear. A key-cap refusal must NOT be retried elsewhere: the login's
    /// organization DID name a workspace, and minting into another would
    /// bill a target the user never chose.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn bootstrap_does_not_retry_a_refusal_another_workspace_cannot_fix() {
        let server = mock_login_plane_listing(serde_json::json!([
            {"workspace_id": "w9", "workspace_name": "My Workspace", "role": "owner",
             "workos_organization_id": null},
        ]))
        .await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .respond_with(ResponseTemplate::new(403).set_body_json(serde_json::json!({
                "error": {"code": "PERMISSION_DENIED",
                          "message": "this workspace is at its API key limit"},
            })))
            .expect(1)
            .mount(&server)
            .await;
        let _store = fresh_keyring();

        let msg = bootstrap_inner("test-cli", &server.uri(), JWT)
            .await
            .unwrap_err()
            .to_string();

        assert!(msg.contains("API key limit"), "unexpected: {msg}");
        assert!(
            workspaces::WorkspaceKeyMap::load("test-cli")
                .keys
                .is_empty(),
            "no key may be minted anywhere on an unrelated refusal"
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
    pub(crate) fn seed_live_session(store: &MockKeyringStore, server_uri: &str) -> TempHome {
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
    pub(crate) struct TempHome {
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
    pub(crate) async fn mock_token_refresh(server: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "fresh-jwt", "refresh_token": "rotated",
                "token_type": "Bearer", "expires_in": 3600,
            })))
            .mount(server)
            .await;
    }

    /// A mint whose only local copy cannot be saved must fail, loudly.
    ///
    /// It used to print "minted and stored in the keyring" and exit zero,
    /// having created a key server-side and dropped the sole copy of its
    /// secret — a live credential on the account that nobody could see, use
    /// or revoke through this CLI.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn a_mint_that_cannot_be_saved_fails_instead_of_reporting_success() {
        clear_endpoint_override();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/keys/bootstrap"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "key_id": "key_w2", "credential": "key_w2:s3cret", "kind": "personal",
                "workspace_id": "w2", "workspace_name": "Born Free", "organization_id": null,
                "expires_at": "2026-08-21T00:00:00Z",
            })))
            .mount(&server)
            .await;
        let store = keyring_that_cannot_save_keys();
        let _home = seed_live_session(&store, &server.uri());
        mock_token_refresh(&server).await;

        let err = mint_for_workspace_at(
            "test-cli",
            &server.uri(),
            &fresh_login_jwt("test-cli").unwrap(),
            "w2",
        )
        .unwrap_err();
        let msg = err.to_string();

        assert!(
            msg.contains("key_w2"),
            "the error must name the key that now exists on the account: {msg}"
        );
        assert!(
            msg.contains("revoke"),
            "and must tell the user how to clean it up: {msg}"
        );
        assert!(
            active_store()
                .get("test-cli", KEY_SCHEME)
                .unwrap()
                .is_none(),
            "nothing was persisted, so nothing must resolve"
        );
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

        let minted = mint_for_workspace_at(
            "test-cli",
            &server.uri(),
            &fresh_login_jwt("test-cli").unwrap(),
            "w2",
        )
        .unwrap();

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

        let err = mint_for_workspace_at(
            "test-cli",
            &server.uri(),
            &fresh_login_jwt("test-cli").unwrap(),
            "w2",
        )
        .unwrap_err();
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

    /// If the rotated token cannot be stored, the command must fail rather
    /// than return a working JWT.
    ///
    /// The exchange has already consumed the previous refresh token
    /// server-side, so a silently-dropped replacement leaves the stored
    /// session dead: the next login-plane call presents an invalidated token
    /// and is refused, sending the user through a browser login for no
    /// visible reason. Failing here costs one retryable command instead.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn a_rotation_that_cannot_be_stored_fails_the_call() {
        clear_endpoint_override();
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "brand-new", "refresh_token": "rotated",
                "token_type": "Bearer", "expires_in": 3600,
            })))
            .mount(&server)
            .await;

        // Seed through a plain store, then swap in one that cannot write the
        // OAuth slot — the seeding itself has to succeed.
        let mock = std::sync::Arc::new(MockKeyringStore::new());
        fern_cli_sdk::auth::set_active_store(super::super::active_key::project(mock.clone()));
        let _home = seed_live_session(&mock, &server.uri());
        fern_cli_sdk::auth::set_active_store(super::super::active_key::project(
            std::sync::Arc::new(FailingWrites {
                inner: mock.clone(),
                slot: SCHEME,
            }),
        ));

        let err = fresh_login_jwt("test-cli").unwrap_err();
        let msg = err.to_string();

        assert!(
            msg.contains("auth login"),
            "the user must be told how to recover: {msg}"
        );
        assert!(
            msg.contains("consumed"),
            "and why the stored session is now unusable: {msg}"
        );
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
        assert!(
            err.contains("Not logged in"),
            "with no credential at all, the flat message is the true one: {err}"
        );
    }

    /// Someone who pasted an API key IS authenticated — `auth status` says so
    /// and every generated command works — so "Not logged in" is a false
    /// statement that sends them hunting for a problem they do not have. The
    /// message has to name what they actually lack: a user identity.
    #[tokio::test(flavor = "multi_thread")]
    #[serial_test::serial]
    async fn a_key_only_user_is_told_what_they_lack_not_that_they_are_logged_out() {
        clear_endpoint_override();
        let store = fresh_keyring();
        let _home = seed_discovery_cache(&AuthEndpoints {
            resource: resource_base_url().to_string(),
            authorization_endpoint: "https://example.invalid/a".to_string(),
            token_endpoint: "https://example.invalid/t".to_string(),
        });
        store.set("test-cli", KEY_SCHEME, "key_1:secret").unwrap();

        let err = fresh_login_jwt("test-cli").unwrap_err().to_string();

        assert!(
            !err.contains("Not logged in"),
            "they are logged in — just not with the credential this needs: {err}"
        );
        assert!(
            err.contains("browser login") && err.contains("auth login"),
            "it must say what to do: {err}"
        );
        assert!(
            err.contains("API key"),
            "and acknowledge the credential they already hold: {err}"
        );
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
