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

/// The API generation this CLI was built against, read from the embedded spec
/// rather than hardcoded, so a spec bump cannot silently desync the header.
fn spec_version() -> String {
    let spec: serde_json::Value =
        serde_json::from_str(include_str!("../cli/hedra/openapi0.json"))
            .expect("embedded spec is valid JSON");
    spec["info"]["version"]
        .as_str()
        .expect("embedded spec has info.version")
        .to_string()
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
/// comes from `CARGO_PKG_VERSION` — which in an integration test resolves to the
/// root `hedra-cli` package.
#[tokio::test]
async fn openapi_path_sends_user_agent() {
    let server = mock_server().await;
    let client = fern_cli_sdk::http::HttpConfig::new("hedra")
        .expect("HttpConfig::new")
        .build_client()
        .expect("build_client");

    let _ = client.get(format!("{}/models", server.uri())).send().await;

    let req = capture_one(&server).await;
    assert_header(&req, "user-agent", &format!("hedra-cli/{}", env!("CARGO_PKG_VERSION")));
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
    assert_header(&req, "user-agent", &format!("hedra-cli/{}", env!("CARGO_PKG_VERSION")));
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
    assert_header(&req, "user-agent", &format!("hedra-cli/{}", env!("CARGO_PKG_VERSION")));
}

// ---------------------------------------------------------------------------
// Scenario 3 — the SDK with no executor injected
// ---------------------------------------------------------------------------

/// Build a `ModelsClient` that talks straight to `base_url` with no executor,
/// so the SDK applies its own headers.
fn direct_models_client(base_url: String) -> hedra_sdk::api::ModelsClient {
    let config = hedra_sdk::ClientConfig { base_url, ..Default::default() };
    hedra_sdk::api::ModelsClient::new(config).expect("ModelsClient::new")
}

/// Without an executor the SDK applies its own headers, so all four arrive.
/// This passing is what makes scenario 2's failure attributable to the executor.
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

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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
fn executor_models_client(base_url: String) -> hedra_sdk::api::ModelsClient {
    let http_config = fern_cli_sdk::http::HttpConfig::new("hedra").expect("HttpConfig::new");
    let auth: fern_cli_sdk::auth::provider::DynAuthProvider =
        Arc::new(fern_cli_sdk::auth::provider::NoAuthProvider);
    let executor = Arc::new(fern_cli_sdk::sdk_executor::CliExecutor::new(
        http_config,
        auth,
        Vec::new(),
        Some(base_url.clone()),
    ));
    let adapter =
        Arc::new(CliExecutorAdapter(executor)) as Arc<dyn hedra_sdk::RequestExecutor>;
    let config = hedra_sdk::ClientConfig { base_url, ..Default::default() };
    let http_client = hedra_sdk::HttpClient::with_executor(adapter, config);
    hedra_sdk::api::ModelsClient { http_client }
}

/// Config-level identity headers must survive the CLI's executor.
///
/// DEFECT — fails today. `cli/hedra/sdk.rs` always builds the client with
/// `HttpClient::with_executor`, and `send_request`
/// (`hedra-sdk/src/core/http_client.rs:498-511`) takes the executor branch,
/// which calls `executor.execute(req)` without ever calling
/// `apply_custom_headers`. So `ClientConfig::custom_headers` is built and
/// discarded.
#[tokio::test]
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

/// The spec-version header must survive the CLI's executor.
///
/// DEFECT — fails today, and this is an ENG-10092 regression. That ticket added
/// X-Hedra-Spec-Version to the CLI surface and verified it by grepping the
/// GENERATED TREE, which cannot distinguish "emitted" from "delivered". The
/// generated methods inject it into `options.additional_headers`
/// (`hedra-sdk/src/api/resources/models/models.rs:45-51`), but request-level
/// headers are applied only inside `apply_custom_headers` — which the executor
/// branch skips. Asserted separately from the config-level headers because a
/// partial fix could restore one channel and not the other.
#[tokio::test]
async fn executor_path_preserves_spec_version_header() {
    let server = mock_server().await;
    let client = executor_models_client(server.uri());

    let _ = client
        .list(&hedra_sdk::api::ModelsListQueryRequest::default(), None)
        .await;

    let req = capture_one(&server).await;
    assert_header(&req, "x-hedra-spec-version", &spec_version());
}

/// The executor path still identifies the CLI, even while dropping SDK headers.
/// Passes today: the User-Agent is a default header on the reqwest client that
/// `HttpConfig::build_client` produces, so it does not travel through
/// `apply_custom_headers`. This is what confirms the executor is sending the
/// request at all, rather than the scenario silently not firing.
#[tokio::test]
async fn executor_path_still_sends_user_agent() {
    let server = mock_server().await;
    let client = executor_models_client(server.uri());

    let _ = client
        .list(&hedra_sdk::api::ModelsListQueryRequest::default(), None)
        .await;

    let req = capture_one(&server).await;
    assert_header(&req, "user-agent", &format!("hedra-cli/{}", env!("CARGO_PKG_VERSION")));
}
