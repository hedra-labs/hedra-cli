//! End-to-end tests for the public `auth` and `workspaces` commands, driving
//! the **real compiled binary** against the file credential backend.
//!
//! Why a process-level harness rather than more unit tests: PR #102 shipped
//! 57 green unit tests and still broke `auth logout`. Those tests exercised
//! the `KeyAuth` projection and the SDK's generic auth handlers *separately*,
//! so nothing asked the one question that matters — after this command, which
//! credential does the CLI actually send? The logout test asserted that the
//! raw backing item was deleted, which was true, and missed that the
//! projection kept serving the credential from the workspace map.
//!
//! So every assertion here is stated in terms of the **resolved** credential,
//! observed on the wire via a mock server, not in terms of the store item a
//! command happened to write. See `authorization_header` below.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry): the generator
//! never emits this tree, which is exactly what makes that entry load-bearing.
//!
//! Deliberately no new dev-dependency: `Cargo.toml` is generator-owned, so a
//! dependency added for these tests would be reverted by the next
//! regeneration. `CARGO_BIN_EXE_hedra-cli` (set by Cargo for integration
//! tests) plus the already-present `tempfile` and `wiremock` cover it.

// The store-inspection helpers on `Sandbox` are used by the logout and
// rotation cases, which arrive alongside the fixes they cover. They live here
// from the start so the harness is complete in one place rather than growing
// a helper per commit.
#![allow(dead_code)]

use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use tempfile::TempDir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

const CLI: &str = "hedra-cli";

/// An isolated CLI installation: its own HOME, its own credential file, and
/// an environment scrubbed of the ambient `HEDRA_*` variables that would
/// otherwise let the developer's shell decide the outcome.
struct Sandbox {
    home: TempDir,
}

impl Sandbox {
    fn new() -> Self {
        Self {
            home: TempDir::new().expect("temp HOME"),
        }
    }

    /// Where `FileKeyringStore::user_default()` lands under this HOME.
    /// Mirrors the platform layout the SDK picks; the tests only run the
    /// paths CI runs, so macOS and Linux are enough.
    fn store_path(&self) -> PathBuf {
        let home = self.home.path();
        #[cfg(target_os = "macos")]
        let root = home.join("Library").join("Application Support");
        #[cfg(not(target_os = "macos"))]
        let root = home.join(".config");
        root.join(CLI).join("auth-keyring.json")
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_hedra-cli"));
        cmd.args(args)
            .env("HOME", self.home.path())
            .env("FERN_CLI_CREDENTIAL_STORE", "file")
            .env("NO_COLOR", "1")
            // The ambient environment must not reach into these tests:
            // HEDRA_API_KEY shadows the keyring outright, and HEDRA_ENV /
            // HEDRA_CLI_BASE_URL would retarget the very base URL some of
            // these cases are about.
            .env_remove("HEDRA_API_KEY")
            .env_remove("HEDRA_ENV")
            .env_remove("HEDRA_CLI_BASE_URL")
            .env_remove("HEDRA_AUTH_AUTHORIZE_URL")
            .env_remove("HEDRA_AUTH_TOKEN_URL")
            .stdin(Stdio::null());
        cmd
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().expect("run hedra-cli")
    }

    /// Run with `input` on stdin — the `auth login --with-token` paste path.
    fn run_with_stdin(&self, args: &[&str], input: &str) -> Output {
        use std::io::Write;
        let mut child = self
            .command(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn hedra-cli");
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(input.as_bytes())
            .expect("write stdin");
        child.wait_with_output().expect("wait hedra-cli")
    }

    /// Seed a workspace key map, the post-#102 storage shape. Written
    /// through the CLI's own paste path rather than by hand so the test
    /// depends on the store's real on-disk encoding, not a guess at it.
    fn seed_workspace_map(&self, map: &str) {
        let out = self.run_with_stdin(
            &["auth", "login", "--with-token", "--scheme", "WorkspaceKeys"],
            &format!("{map}\n"),
        );
        assert!(out.status.success(), "seeding the workspace map failed");
    }

    fn read_store(&self) -> serde_json::Value {
        let raw = std::fs::read_to_string(self.store_path()).unwrap_or_else(|_| "{}".to_string());
        serde_json::from_str(&raw).expect("credential store is JSON")
    }

    /// The raw value at a store slot, if any. Used only to distinguish "the
    /// item is gone" from "the item is stale" — never as the assertion
    /// itself, which is always about the resolved credential.
    fn raw_slot(&self, slot: &str) -> Option<String> {
        self.read_store()
            .get(slot)
            .and_then(|v| v.as_str())
            .map(str::to_string)
    }
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

/// **The oracle.** Point a real data-plane command at a mock server and
/// report the `Authorization` header it sent — i.e. the credential this
/// installation actually resolves right now.
///
/// `None` means the CLI sent no credential at all. That is the shape a
/// logged-out install must produce, and asserting it is the whole point:
/// "the item was deleted" and "the CLI stopped sending the key" are
/// different claims, and #102 satisfied only the first.
fn authorization_header(sandbox: &Sandbox) -> Option<String> {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [] })),
            )
            .mount(&server)
            .await;

        let out = sandbox.run(&["models", "list", "--base-url", &server.uri()]);

        let requests = server
            .received_requests()
            .await
            .expect("mock server recorded requests");
        assert!(
            !requests.is_empty(),
            "the CLI never reached the mock server; it exited with {:?}\nstderr:\n{}",
            out.status.code(),
            stderr(&out),
        );
        requests[0]
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string)
    })
}

/// A workspace map holding one bound key, active.
fn map_with(workspace: &str, key_id: &str, credential: &str) -> String {
    serde_json::json!({
        "active_workspace_id": workspace,
        "keys": {
            workspace: {
                "key_id": key_id,
                "credential": credential,
                "workspace_name": "Test Workspace",
                "expires_at": null,
            }
        },
        "unbound_key": null,
    })
    .to_string()
}

// ---------------------------------------------------------------------------
// Baseline: the projection itself, observed on the wire.
// ---------------------------------------------------------------------------

#[test]
fn the_active_workspace_key_is_what_reaches_the_server() {
    let s = Sandbox::new();
    s.seed_workspace_map(&map_with("ws_1", "key_1", "key_1:secret_one"));

    assert_eq!(
        authorization_header(&s).as_deref(),
        Some("Bearer key_1:secret_one"),
        "the map's active entry must be the credential on the wire"
    );
}

#[test]
fn a_pasted_key_reaches_the_server_when_no_map_exists() {
    let s = Sandbox::new();
    let out = s.run_with_stdin(
        &["auth", "login", "--with-token", "--scheme", "KeyAuth"],
        "pasted:secret\n",
    );
    assert!(out.status.success(), "paste failed: {}", stderr(&out));

    assert_eq!(
        authorization_header(&s).as_deref(),
        Some("Bearer pasted:secret"),
        "with no map, the raw KeyAuth item is the credential"
    );
}

#[test]
fn a_fresh_install_sends_no_credential() {
    let s = Sandbox::new();
    assert_eq!(
        authorization_header(&s),
        None,
        "an install that has never logged in must send no Authorization header"
    );
}

// ---------------------------------------------------------------------------
// Logout.
// ---------------------------------------------------------------------------

/// The headline regression. `auth logout --scheme KeyAuth` exited zero and
/// printed that the credential had been removed while the projection kept
/// serving it from the map, so `auth status` stayed active and every request
/// stayed signed. Asserting on the wire is what makes this test meaningful:
/// the old behaviour genuinely did delete the raw item it was asked about.
#[test]
fn logout_stops_the_cli_sending_a_credential() {
    let s = Sandbox::new();
    s.seed_workspace_map(&map_with("ws_1", "key_1", "key_1:secret_one"));
    assert!(authorization_header(&s).is_some(), "precondition");

    let out = s.run(&["auth", "logout", "--scheme", "KeyAuth"]);
    assert!(out.status.success(), "logout failed: {}", stderr(&out));

    assert_eq!(
        authorization_header(&s),
        None,
        "logout reported success, so the CLI must stop presenting a credential"
    );
}

/// Every held key is a live credential. Leaving the non-active ones behind
/// would let `workspaces select <other>` re-authenticate with no challenge,
/// which makes "logged out" false for anyone holding a second workspace.
#[test]
fn logout_clears_every_held_workspace_key() {
    let s = Sandbox::new();
    s.seed_workspace_map(
        &serde_json::json!({
            "active_workspace_id": "ws_1",
            "keys": {
                "ws_1": { "key_id": "key_1", "credential": "key_1:one",
                          "workspace_name": "One", "expires_at": null },
                "ws_2": { "key_id": "key_2", "credential": "key_2:two",
                          "workspace_name": "Two", "expires_at": null },
            },
            "unbound_key": null,
        })
        .to_string(),
    );

    let out = s.run(&["auth", "logout", "--scheme", "KeyAuth"]);
    assert!(out.status.success(), "logout failed: {}", stderr(&out));

    assert_eq!(
        s.raw_slot("WorkspaceKeys"),
        None,
        "a logout that leaves usable secrets on disk is not a logout"
    );
}

/// An install that never migrated still has only the standalone item.
#[test]
fn logout_works_on_an_unmigrated_install() {
    let s = Sandbox::new();
    let out = s.run_with_stdin(
        &["auth", "login", "--with-token", "--scheme", "KeyAuth"],
        "legacy:secret\n",
    );
    assert!(out.status.success(), "paste failed: {}", stderr(&out));
    assert!(authorization_header(&s).is_some(), "precondition");

    let out = s.run(&["auth", "logout", "--scheme", "KeyAuth"]);
    assert!(out.status.success(), "logout failed: {}", stderr(&out));

    assert_eq!(authorization_header(&s), None);
}

// ---------------------------------------------------------------------------
// Token paste.
// ---------------------------------------------------------------------------

/// Rotation. Pasting a replacement key on a migrated install used to write a
/// raw item the projection then ignored, so the CLI went on presenting the
/// key the user was trying to replace — the exact failure mode someone
/// rotating a leaked credential is trying to avoid.
#[test]
fn a_pasted_key_replaces_the_one_the_map_was_serving() {
    let s = Sandbox::new();
    s.seed_workspace_map(&map_with("ws_1", "key_1", "key_1:leaked"));

    let out = s.run_with_stdin(
        &["auth", "login", "--with-token", "--scheme", "KeyAuth"],
        "key_9:rotated\n",
    );
    assert!(out.status.success(), "paste failed: {}", stderr(&out));

    assert_eq!(
        authorization_header(&s).as_deref(),
        Some("Bearer key_9:rotated"),
        "the pasted key must be what goes on the wire, not the map's older answer"
    );
}
