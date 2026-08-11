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
