//! What a server actually receives from this CLI.
//!
//! Three scenarios, one per way a request can leave this binary:
//!
//!   1. OpenAPI path      — `HttpConfig::build_client`, what real commands use today
//!   2. SDK via executor  — replicates `cli/hedra/sdk.rs`, what a custom command hits
//!   3. SDK direct        — `HttpClient::new`, isolates which layer drops headers
//!
//! Assertions state the INTENDED contract, so some fail today. Each failure is
//! annotated with its root cause. See
//! docs/superpowers/specs/2026-08-10-wire-contract-harness-design.md
//!
//! These tests read the ambient environment. `HttpConfig` resolves
//! `HEDRA_USER_AGENT_SUFFIX`, `HEDRA_PROXY`, `HEDRA_CA_BUNDLE`, and
//! `HEDRA_INSECURE` (documented at `src/http.rs:15-20`), so a shell exporting
//! any of them can turn these red for reasons that say nothing about the CLI.
//! CI runs clean, so the dependency is documented rather than guarded — an
//! `EnvGuard`/`serial_test` rework would cost more than it buys here.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// A received request's header value, as a UTF-8 string.
fn header_value(req: &Request, name: &str) -> Option<String> {
    req.headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn assert_header(req: &Request, name: &str, expected: &str) {
    match header_value(req, name) {
        Some(actual) => assert_eq!(
            actual, expected,
            "header `{name}`: expected `{expected}`, server received `{actual}`"
        ),
        None => panic!("header `{name}`: expected `{expected}`, server received nothing"),
    }
}

/// The spec the CLI was generated from, parsed from the copy embedded in the
/// binary rather than fetched, so these helpers see exactly what ships.
fn embedded_spec() -> serde_json::Value {
    serde_json::from_str(include_str!("../cli/hedra/openapi0.json"))
        .expect("embedded spec is valid JSON")
}

/// The API generation this CLI was built against, read from `info.version`
/// rather than hardcoded, so a spec bump cannot silently desync the header.
///
/// This governs the SDK's own copy of the value: the generator bakes a literal
/// into each resource method's `additional_headers`
/// (`hedra-sdk/src/api/resources/models/models.rs:45-51`), and `info.version`
/// is the in-repo field that literal tracks. The CLI path uses a *different*
/// source — see [`global_header_default`].
fn spec_version() -> String {
    embedded_spec()["info"]["version"]
        .as_str()
        .expect("embedded spec has info.version")
        .to_string()
}

/// The `x-fern-global-headers` declarations from the embedded spec, as the
/// `(header, value)` pairs the CLI resolves for every outgoing request.
///
/// This reproduces the real runtime chain, which the executor scenario below
/// otherwise misses entirely:
///
///   1. `cli/hedra/openapi0.json` declares `x-fern-global-headers`, each entry
///      carrying an optional `default`.
///   2. `src/openapi/app.rs:1568-1633` registers one hidden global clap arg per
///      entry, with `.default_value(default)` when the spec supplies one.
///   3. `src/openapi/binding.rs:303-309` reads those matches back through
///      `resolve_global_header_value` into `BindingEntry::global_headers`.
///   4. `AppContext::build_sdk_executor` (`src/openapi/app.rs:2288-2295`) hands
///      that vector to `CliExecutor::new`.
///   5. `CliExecutor::build_request` (`src/sdk_executor.rs:227-231`) stamps each
///      pair onto every request it builds.
///
/// Entries without a `default` are skipped: with no flag, no env var, and no
/// default, `resolve_global_header_value` yields `None` and the CLI sends
/// nothing — so skipping them matches runtime behaviour for a bare invocation.
fn cli_global_headers() -> Vec<(String, String)> {
    let spec = embedded_spec();
    spec["x-fern-global-headers"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or_default()
        .iter()
        .filter_map(|h| {
            Some((
                h["header"].as_str()?.to_string(),
                h["default"].as_str()?.to_string(),
            ))
        })
        .collect()
}

/// The value the CLI stamps on `name`, from the spec's global-header default.
///
/// Deliberately distinct from [`spec_version`]. Both read the same file and
/// both are `3.2.2` today, but they are different fields:
/// `x-fern-global-headers[].default` feeds the CLI's clap flag, `info.version`
/// feeds the SDK's generated literal. Nothing keeps them equal, so each
/// assertion reads the field that actually feeds the path it exercises.
fn global_header_default(name: &str) -> String {
    cli_global_headers()
        .into_iter()
        .find(|(header, _)| header.eq_ignore_ascii_case(name))
        .map(|(_, value)| value)
        .unwrap_or_else(|| {
            panic!("embedded spec declares `{name}` in x-fern-global-headers with a default")
        })
}

/// Stand up a mock server that answers `GET /models` with an empty JSON object.
///
/// The body deliberately does not match `ModelListResponse`. Scenarios 2 and 3
/// ignore the call's `Result`: the request is captured by wiremock before the
/// response is deserialized, so a decode failure costs nothing and saves this
/// harness from tracking the response schema.
async fn mock_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/models"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
        .mount(&server)
        .await;
    server
}

/// The single request the server received. Panics if not exactly one.
async fn capture_one(server: &MockServer) -> Request {
    let mut received = server
        .received_requests()
        .await
        .expect("wiremock is recording requests");
    assert_eq!(received.len(), 1, "expected exactly one captured request");
    received.remove(0)
}

// ---------------------------------------------------------------------------
// Scenario 1 — the OpenAPI path (what real commands use today)
// ---------------------------------------------------------------------------

/// The default User-Agent is `hedra-cli/<crate version>`.
///
/// The product token is derived from the `HttpConfig` name (`"hedra"` →
/// `hedra-cli`, per `src/user_agent.rs`) rather than hardcoded, and the version
/// comes from `CARGO_PKG_VERSION`.
///
/// That is a shape check, not a cross-crate one: the lib under test
/// (`fern_cli_sdk`) lives in the same package as this test, so the version it
/// stamps and the version `CARGO_PKG_VERSION` expands to cannot drift by
/// construction. Only `sdk_direct_reports_the_real_crate_version` compares
/// across crates.
#[tokio::test]
async fn openapi_path_sends_user_agent() {
    let server = mock_server().await;
    let client = fern_cli_sdk::http::HttpConfig::new("hedra")
        .expect("HttpConfig::new")
        .build_client()
        .expect("build_client");

    let _ = client.get(format!("{}/models", server.uri())).send().await;

    let req = capture_one(&server).await;
    assert_header(
        &req,
        "user-agent",
        &format!("hedra-cli/{}", env!("CARGO_PKG_VERSION")),
    );
}

/// A configured suffix is appended after the CLI's own product token.
#[tokio::test]
async fn openapi_path_appends_user_agent_suffix() {
    let server = mock_server().await;
    let client = fern_cli_sdk::http::HttpConfig::new("hedra")
        .expect("HttpConfig::new")
        .with_user_agent_suffix_override(Some("partner-app/3.1".to_string()))
        .build_client()
        .expect("build_client");

    let _ = client.get(format!("{}/models", server.uri())).send().await;

    let req = capture_one(&server).await;
    assert_header(
        &req,
        "user-agent",
        &format!("hedra-cli/{} partner-app/3.1", env!("CARGO_PKG_VERSION")),
    );
}

/// A blank suffix is ignored rather than producing a trailing space.
#[tokio::test]
async fn openapi_path_ignores_blank_user_agent_suffix() {
    let server = mock_server().await;
    let client = fern_cli_sdk::http::HttpConfig::new("hedra")
        .expect("HttpConfig::new")
        .with_user_agent_suffix_override(Some("   ".to_string()))
        .build_client()
        .expect("build_client");

    let _ = client.get(format!("{}/models", server.uri())).send().await;

    let req = capture_one(&server).await;
    assert_header(
        &req,
        "user-agent",
        &format!("hedra-cli/{}", env!("CARGO_PKG_VERSION")),
    );
}

/// A suffix that is not valid header content is ignored, not sent raw.
#[tokio::test]
async fn openapi_path_ignores_header_invalid_user_agent_suffix() {
    let server = mock_server().await;
    let client = fern_cli_sdk::http::HttpConfig::new("hedra")
        .expect("HttpConfig::new")
        .with_user_agent_suffix_override(Some("bad\nvalue".to_string()))
        .build_client()
        .expect("build_client");

    let _ = client.get(format!("{}/models", server.uri())).send().await;

    let req = capture_one(&server).await;
    assert_header(
        &req,
        "user-agent",
        &format!("hedra-cli/{}", env!("CARGO_PKG_VERSION")),
    );
}

// ---------------------------------------------------------------------------
// Scenario 3 — the SDK with no executor injected
// ---------------------------------------------------------------------------

/// Build a `ModelsClient` that talks straight to `base_url` with no executor,
/// so the SDK applies its own headers.
fn direct_models_client(base_url: String) -> hedra_sdk::api::ModelsClient {
    let config = hedra_sdk::ClientConfig {
        base_url,
        ..Default::default()
    };
    hedra_sdk::api::ModelsClient::new(config).expect("ModelsClient::new")
}

/// Without an executor the SDK applies its own headers, so three arrive with
/// correct values; the fourth (`X-Fern-SDK-Version`) is asserted separately
/// because its value is wrong. This passing is what makes scenario 2's
/// remaining failure attributable to the executor.
#[tokio::test]
async fn sdk_direct_sends_identity_headers() {
    let server = mock_server().await;
    let client = direct_models_client(server.uri());

    let _ = client
        .list(&hedra_sdk::api::ModelsListQueryRequest::default(), None)
        .await;

    let req = capture_one(&server).await;
    assert_header(&req, "x-fern-language", "Rust");
    assert_header(&req, "x-fern-sdk-name", "hedra_sdk");
    assert_header(&req, "x-hedra-spec-version", &spec_version());
}

/// The SDK version header must report the crate's real version.
///
/// DEFECT — fails today. `hedra-sdk/src/config.rs:38` hardcodes the literal
/// "0.1.0". It has not moved since the first commit, surviving 0.2.0 → 0.9.9 →
/// 1.0.0-dev and every regeneration: `fern generate --version` stamps the root
/// manifest and has no channel to this constant. Since #52 (ENG-10219) aligned
/// all three manifests, this is the last place in the repo still saying 0.1.0.
///
/// Comparing against CARGO_PKG_VERSION also transitively guards that alignment.
#[tokio::test]
#[ignore = "ENG-10226 defect 2: the generated literal says 0.1.0; un-ignored by the header-patch PR (#57)"]
async fn sdk_direct_reports_the_real_crate_version() {
    let server = mock_server().await;
    let client = direct_models_client(server.uri());

    let _ = client
        .list(&hedra_sdk::api::ModelsListQueryRequest::default(), None)
        .await;

    let req = capture_one(&server).await;
    assert_header(&req, "x-fern-sdk-version", env!("CARGO_PKG_VERSION"));
}

// ---------------------------------------------------------------------------
// Scenario 2 — the SDK with the CLI's executor injected
// ---------------------------------------------------------------------------

/// Mirror of `CliExecutorAdapter` in `cli/hedra/sdk.rs:19-37`.
///
/// Replicated rather than imported: that file is part of the `hedra` binary,
/// not the `fern_cli_sdk` lib, so an integration test cannot reach it. If the
/// generated bridge changes shape, update this to match.
struct CliExecutorAdapter(Arc<fern_cli_sdk::sdk_executor::CliExecutor>);

impl hedra_sdk::RequestExecutor for CliExecutorAdapter {
    fn execute(
        &self,
        request: reqwest::Request,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<reqwest::Response, Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            fern_cli_sdk::sdk_executor::SdkRequestExecutor::execute(&*self.0, request)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })
    }
}

/// Mirror of `client()` in `cli/hedra/sdk.rs:48-66`, pointed at the mock server.
///
/// The real bridge calls `ctx.build_sdk_executor()`, which reads its arguments
/// off the live `AppContext` — unreachable here, because building one requires
/// a parsed `clap::ArgMatches` and the whole binding stack. So each argument is
/// reconstructed by hand, and every hand-reconstruction is a place this replica
/// can lie about the real path. The known divergences, all deliberate:
///
/// - **`global_headers`** — real: `entries[0].global_headers`, resolved from the
///   spec's `x-fern-global-headers` defaults through clap. Here: the same values,
///   re-derived from the embedded spec by `cli_global_headers()`. This argument
///   was `Vec::new()` until the final review; that single simplification made
///   `executor_path_preserves_spec_version_header` fail and produced a false
///   ENG-10092 regression report. Do not "simplify" it back.
/// - **`auth_provider`** — real: whatever the binding resolved (API key, OAuth,
///   …). Here: `NoAuthProvider`, so no `Authorization` header is sent. The real
///   binary does send one; this harness asserts nothing about auth.
/// - **`base_url`** — real: `base_url_override`, normally `None`, so the SDK's
///   own `base_url` governs. Here: set in *both* places (executor override and
///   `ClientConfig`) to point at the mock server. Redundant but harmless — they
///   agree, so neither rewrite can send the request anywhere else.
///
/// Retries are left at `RetriesConfig::default()`, matching `CliExecutor::new`.
fn executor_models_client(base_url: String) -> hedra_sdk::api::ModelsClient {
    let http_config = fern_cli_sdk::http::HttpConfig::new("hedra").expect("HttpConfig::new");
    let auth: fern_cli_sdk::auth::provider::DynAuthProvider =
        Arc::new(fern_cli_sdk::auth::provider::NoAuthProvider);
    let executor = Arc::new(fern_cli_sdk::sdk_executor::CliExecutor::new(
        http_config,
        auth,
        cli_global_headers(),
        Some(base_url.clone()),
    ));
    let adapter = Arc::new(CliExecutorAdapter(executor)) as Arc<dyn hedra_sdk::RequestExecutor>;
    let config = hedra_sdk::ClientConfig {
        base_url,
        ..Default::default()
    };
    let http_client = hedra_sdk::HttpClient::with_executor(adapter, config);
    hedra_sdk::api::ModelsClient { http_client }
}

/// Config-level identity headers must survive the CLI's executor.
///
/// DEFECT — fails today, but not the one it first looks like. The SDK side is
/// working as documented: `with_executor`
/// (`hedra-sdk/src/core/http_client.rs:229-236`) states outright that "Auth
/// headers, custom headers, and retry logic are NOT applied by this client —
/// the executor's transport stack is expected to handle them", to prevent
/// double-retry and double-auth when the SDK is embedded in a CLI. So
/// `send_request` (`:498-511`) skipping `apply_custom_headers` on the executor
/// branch is the design, not the bug.
///
/// The bug is the other half of that contract going unhonoured: the CLI's
/// executor never picked up the `X-Fern-*` trio it thereby became responsible
/// for. `ClientConfig::custom_headers` (`hedra-sdk/src/config.rs:35-39`) holds
/// `X-Fern-Language` / `X-Fern-SDK-Name` / `X-Fern-SDK-Version`, and nothing in
/// `cli/hedra/sdk.rs` merges them into the channel that does reach the wire —
/// `CliExecutor`'s `global_headers`. Contrast
/// `executor_path_preserves_spec_version_header`, where the CLI *does* supply
/// the header through that channel and it arrives.
///
/// The fix therefore belongs in the generated bridge, not in `http_client.rs`.
#[tokio::test]
#[ignore = "ENG-10226 defect 1: the executor drops the X-Fern-* trio; un-ignored by the header-patch PR (#57)"]
async fn executor_path_preserves_config_identity_headers() {
    let server = mock_server().await;
    let client = executor_models_client(server.uri());

    let _ = client
        .list(&hedra_sdk::api::ModelsListQueryRequest::default(), None)
        .await;

    let req = capture_one(&server).await;
    assert_header(&req, "x-fern-language", "Rust");
    assert_header(&req, "x-fern-sdk-name", "hedra_sdk");
}

/// The spec-version header must survive the CLI's executor. It does — this is
/// the regression lock on ENG-10092, and nothing else in the repo pins it.
///
/// PASSES. An earlier revision of this test asserted the opposite and called
/// the failure an ENG-10092 regression. That was wrong, and the cause was in
/// this file: the replica passed `Vec::new()` as `CliExecutor`'s
/// `global_headers`. The real bridge passes `ctx.build_sdk_executor()`'s
/// `entries[0].global_headers`, which is not empty. A trace of the built
/// binary against a header-logging server settled it: `hedra` sends
/// `x-hedra-spec-version: 3.2.2`.
///
/// The header survives because the CLI re-supplies it on a channel the SDK's
/// executor branch does not touch. Two independent deliveries exist:
///
/// - SDK-side, dropped: the generated methods inject it into
///   `options.additional_headers`
///   (`hedra-sdk/src/api/resources/models/models.rs:45-51`), which only
///   `apply_custom_headers` reads — skipped on the executor branch.
/// - CLI-side, delivered: the spec's `x-fern-global-headers` default becomes a
///   global clap flag, lands in `BindingEntry::global_headers`, and
///   `CliExecutor::build_request` stamps it on every request. See
///   [`cli_global_headers`] for the full chain.
///
/// So ENG-10092 holds on the CLI's own path — but only via that second channel.
/// A refactor dropping global-header plumbing from `CliExecutor` would silently
/// undo it, since the SDK-side copy is still discarded. That is what this test
/// exists to catch. Asserted separately from the config-level headers because
/// the two travel on different channels and a change can break either alone.
#[tokio::test]
async fn executor_path_preserves_spec_version_header() {
    let server = mock_server().await;
    let client = executor_models_client(server.uri());

    let _ = client
        .list(&hedra_sdk::api::ModelsListQueryRequest::default(), None)
        .await;

    let req = capture_one(&server).await;
    // The global-header default, not `info.version`: this path is fed by the
    // clap flag. See `global_header_default` for why they are not the same.
    assert_header(
        &req,
        "x-hedra-spec-version",
        &global_header_default("X-Hedra-Spec-Version"),
    );
}

/// The executor path still identifies the CLI, even while dropping the
/// `X-Fern-*` trio.
///
/// Passes today: the User-Agent is a default header on the reqwest client that
/// `HttpConfig::build_client` produces, so it does not travel through
/// `apply_custom_headers`. Its role as a control is to show that the executor's
/// *client-level* headers survive — so the `X-Fern-*` absences next door are
/// specific to one header channel rather than a wholesale header wipe.
///
/// It is not what proves the request was sent: `capture_one` already asserts
/// exactly one received request, so a scenario that never fired fails with
/// `expected exactly one captured request` before any header is read.
#[tokio::test]
async fn executor_path_still_sends_user_agent() {
    let server = mock_server().await;
    let client = executor_models_client(server.uri());

    let _ = client
        .list(&hedra_sdk::api::ModelsListQueryRequest::default(), None)
        .await;

    let req = capture_one(&server).await;
    assert_header(
        &req,
        "user-agent",
        &format!("hedra-cli/{}", env!("CARGO_PKG_VERSION")),
    );
}
