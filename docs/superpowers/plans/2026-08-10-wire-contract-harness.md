# Wire-Contract Harness Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `tests/wire_contract.rs`, a wiremock-backed executable specification of what a server actually receives from this CLI — the `X-Fern-*` identity headers, `X-Hedra-Spec-Version`, and the `User-Agent` shape.

**Architecture:** One integration test file with three scenarios matching the three ways a request leaves this CLI: the OpenAPI path (`HttpConfig::build_client`), the SDK-via-executor path (replicating `cli/hedra/sdk.rs`), and the SDK-direct path (`HttpClient::new`, no executor). Requests are captured by a local `wiremock` server and inspected. Assertions compare against sources of truth rather than frozen literals.

**Tech Stack:** Rust 2021, `wiremock` 0.6, `tokio` (both already dev-dependencies), `serde_json` (already a dependency).

**Spec:** `docs/superpowers/specs/2026-08-10-wire-contract-harness-design.md`

## Global Constraints

- **No new dependencies.** `wiremock` 0.6, `tokio` (`features = ["full"]`), `serial_test`, and `tempfile` are already in `[dev-dependencies]`; `serde_json` is a normal dependency. Adding a crate is out of scope.
- **Do not modify generator-owned files.** `hedra-sdk/**`, `src/**`, and `cli/hedra/sdk.rs` are absent from `.fernignore` and are reverted by the next regeneration. This change touches only `tests/`, `.fernignore`, and `docs/`.
- **Never assert a frozen version literal.** Compare against `env!("CARGO_PKG_VERSION")` and the embedded spec's `info.version`. A test hardcoding `"1.0.0-dev"` would need editing every release and would not catch the drift it exists to catch.
- **`tests/` must be added to `.fernignore`.** A file that exists but is not emitted is *deleted* by regeneration otherwise — see the `.gitattributes` entry and PR #29.
- **No local Rust toolchain.** Every verification step runs in a container:
  `docker run --rm -v "$PWD":/w -w /w rust:latest cargo test --locked --test wire_contract`
- **Red is the deliverable.** Three assertions are expected to fail. A *compile* error or a failure with a different message means the harness is wrong; a failure with the stated message is success.

---

## File Structure

| File | Responsibility |
|---|---|
| `tests/wire_contract.rs` (create) | All three scenarios plus shared assertion helpers. Single file — the scenarios share the helpers and the wiremock setup, and splitting them would separate code that changes together. |
| `.fernignore` (modify) | Add `tests/` so regeneration cannot delete the harness. |

Task ordering is deliberate: Task 1 delivers a **fully green** scenario first. It proves the wiremock plumbing, request capture, and assertion helpers work before any red assertion enters the file. Starting with a red scenario would make harness bugs indistinguishable from the real defects being documented.

---

### Task 1: Harness scaffold, `.fernignore` entry, and the OpenAPI-path User-Agent scenario

**Files:**
- Create: `tests/wire_contract.rs`
- Modify: `.fernignore`

**Interfaces:**
- Consumes: `fern_cli_sdk::http::HttpConfig` (`src/http.rs:127` `new`, `:178` `with_user_agent_suffix_override`, `:333` `build_client`).
- Produces: helpers `header_value(&wiremock::Request, &str) -> Option<String>`, `assert_header(&wiremock::Request, &str, &str)`, `spec_version() -> String`, `async fn mock_server() -> MockServer`, and `async fn capture_one(&MockServer) -> wiremock::Request`. Tasks 2 and 3 use all five.

- [ ] **Step 1: Add the `.fernignore` entry**

Append to `.fernignore`:

```
# The wire-contract harness. The generator never emits `tests/`, which is
# exactly why this entry is load-bearing: for a file that exists but is not
# emitted, .fernignore is the only thing stopping regeneration from DELETING
# it (see the .gitattributes entry above and PR #29).
tests/
```

- [ ] **Step 2: Write the scaffold and the first scenario**

Create `tests/wire_contract.rs`:

```rust
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
```

- [ ] **Step 3: Run the scenario — it must be fully green**

Run:

```bash
docker run --rm -v "$PWD":/w -w /w rust:latest \
  cargo test --locked --test wire_contract
```

Expected: 4 passed, 0 failed.

This is the one task whose success is green. If any of these fail, the harness itself is wrong — fix it before adding red scenarios, because after Task 2 a harness bug and a real defect look identical.

Two likely compile fixes, neither a design change:
- `wiremock::Request`'s header accessor. If `req.headers` is not a public `http::HeaderMap` in 0.6, adjust `header_value` only — every assertion goes through it.
- The exact suffix separator in `HttpConfig::user_agent` (`src/http.rs:215-230`). Read that function and match its real output; the *shape* is what matters, not a guessed separator.

- [ ] **Step 4: Commit**

```bash
git add tests/wire_contract.rs .fernignore
git commit -m "test: capture the CLI's outbound User-Agent contract

Adds tests/wire_contract.rs with the OpenAPI-path scenario: the default
User-Agent shape plus the three documented suffix behaviours. Asserts
against CARGO_PKG_VERSION rather than a literal, so a release cannot
silently break the shape.

.fernignore gains tests/ — the generator never emits that directory,
which is precisely why the entry is needed: regeneration deletes files
that exist but are not emitted."
```

---

### Task 2: SDK-direct scenario — prove the headers exist, and pin the stale constant

**Files:**
- Modify: `tests/wire_contract.rs`

**Interfaces:**
- Consumes: helpers from Task 1; `hedra_sdk::{ClientConfig, HttpClient}`, `hedra_sdk::api::{ModelsClient, ModelsListQueryRequest}`.
- Produces: nothing consumed by later tasks.

This scenario is why the suite is diagnostic rather than merely red. `HttpClient::new` takes the non-executor branch of `send_request` (`hedra-sdk/src/core/http_client.rs:498-511`), so `apply_custom_headers` runs and every header arrives. Its *presence* assertions pass. That is what localizes Task 3's failures to `with_executor` rather than to the header definitions.

Its one *value* assertion fails, on an unrelated defect: `X-Fern-SDK-Version` is a hardcoded `"0.1.0"` literal (`hedra-sdk/src/config.rs:38`) that `fern generate --version` has no channel to.

- [ ] **Step 1: Append the scenario**

Append to `tests/wire_contract.rs`:

```rust
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
```

- [ ] **Step 2: Run and confirm exactly one new failure, for the stated reason**

Run:

```bash
docker run --rm -v "$PWD":/w -w /w rust:latest \
  cargo test --locked --test wire_contract
```

Expected: 5 passed, 1 failed. The failure must be
`sdk_direct_reports_the_real_crate_version`, reading approximately:

```
header `x-fern-sdk-version`: expected `1.0.0-dev`, server received `0.1.0`
```

If `sdk_direct_sends_identity_headers` also fails, stop — the premise that the
non-executor path applies headers is wrong, and Task 3's conclusions would be
unsound. Re-read `send_request` before continuing.

- [ ] **Step 3: Commit**

```bash
git add tests/wire_contract.rs
git commit -m "test: pin X-Fern-SDK-Version against the crate version

Adds the SDK-direct scenario. Its presence assertions pass, proving the
SDK builds and applies all four identity headers when no executor is
injected — which is what will localize the executor-path failures.

The version assertion fails by design: config.rs hardcodes 0.1.0 and
fern generate --version has no channel to it."
```

---

### Task 3: SDK-via-executor scenario — the two bypass failures

**Files:**
- Modify: `tests/wire_contract.rs`

**Interfaces:**
- Consumes: helpers from Task 1; `fern_cli_sdk::sdk_executor::{CliExecutor, SdkRequestExecutor}`, `fern_cli_sdk::auth::provider::{DynAuthProvider, NoAuthProvider}`, `hedra_sdk::{ClientConfig, HttpClient, RequestExecutor}`.

This is the path a custom command hits. It replicates `cli/hedra/sdk.rs:19-66` because that file belongs to the binary, not the lib, and so cannot be imported by an integration test.

- [ ] **Step 1: Append the scenario**

Append to `tests/wire_contract.rs`:

```rust
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
```

- [ ] **Step 2: Run and confirm the full expected failure set**

Run:

```bash
docker run --rm -v "$PWD":/w -w /w rust:latest \
  cargo test --locked --test wire_contract
```

Expected: 6 passed, 3 failed. Exactly these three, and no others:

| Test | Expected message |
|---|---|
| `sdk_direct_reports_the_real_crate_version` | `x-fern-sdk-version`: expected `1.0.0-dev`, received `0.1.0` |
| `executor_path_preserves_config_identity_headers` | `x-fern-language`: expected `Rust`, received nothing |
| `executor_path_preserves_spec_version_header` | `x-hedra-spec-version`: expected `3.2.2`, received nothing |

`executor_path_still_sends_user_agent` **must pass**. If it fails, the executor
is not reaching the mock server at all and the two "received nothing" failures
prove nothing — fix the wiring before treating them as evidence.

- [ ] **Step 3: Commit**

```bash
git add tests/wire_contract.rs
git commit -m "test: document that the CLI executor drops every SDK header

Adds the SDK-via-executor scenario, replicating cli/hedra/sdk.rs. Two
assertions fail by design: with_executor skips apply_custom_headers, so
both the config-level X-Fern-* headers and the request-level
X-Hedra-Spec-Version are discarded.

The second is an ENG-10092 regression — that ticket verified the header
by grepping the generated tree, which cannot tell emitted from
delivered. The User-Agent assertion passes, confirming the executor does
reach the server and the absences are real."
```

---

### Task 4: Draft PR and the upstream fix ticket

**Files:** none (no code changes)

- [ ] **Step 1: Push and open the draft PR**

Use the `pr:new` skill (per CLAUDE.md, PRs go through it rather than manual `gh`). The PR body must state that **red CI is the intended outcome**, list the three failing tests with their root causes from Task 3's table, and note that no fix is included because every file involved is generator-owned.

- [ ] **Step 2: File the upstream fix ticket**

File against fern-config / fern-cli-generator, citing the three failing test names as acceptance criteria. Include the lead that makes this cheap:

> `CliExecutor` already carries a `global_headers: Vec<(String, String)>` channel (`src/sdk_executor.rs:64`), populated from the OpenAPI binding at `src/openapi/app.rs:2292`. The generated bridge in `cli/hedra/sdk.rs` builds `ClientConfig::default()` and hands it to `with_executor` without ever merging `custom_headers` into that channel. So the fix is plumbing in the generated bridge, not a rewrite of `http_client.rs`.

Note the separate second defect: the `X-Fern-SDK-Version` literal needs a channel from `fern generate --version`, and fixing the bypass alone leaves it reporting `0.1.0`.

- [ ] **Step 3: Link the ticket in the PR body and leave the PR in draft**

---

## Self-Review

**Spec coverage.** Three scenarios → Tasks 1/3/2 respectively. Sources-of-truth assertions → Task 1 helper `spec_version()` plus `env!("CARGO_PKG_VERSION")` throughout. `.fernignore` survival → Task 1 Step 1. Container verification → every run step. Three expected failures with distinct root causes → Task 3 Step 2 table. Deliverables 3 and 4 → Task 4. The spec's non-goal (no in-repo fix) is enforced by the Global Constraints.

**Placeholders.** None: every step carries runnable code or an exact command.

**Type consistency.** `header_value` / `assert_header` / `spec_version` / `mock_server` / `capture_one` are defined once in Task 1 and used with identical signatures in Tasks 2 and 3. `ModelsClient` is constructed two ways — `ModelsClient::new(config)` (Task 2, no executor) and the struct literal `ModelsClient { http_client }` (Task 3, executor injected) — matching `hedra-sdk/src/api/resources/models/models.rs:6-11` and `cli/hedra/sdk.rs:56-66`.

**No unused helpers.** The suite asserts the *intended* contract, so absences are expressed as failing presence assertions and no negative-assertion helper is needed. One was drafted and removed in the pre-flight scan: every helper defined in Task 1 is called by Tasks 1-3.
