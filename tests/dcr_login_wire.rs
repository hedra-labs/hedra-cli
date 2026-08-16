//! Wire tests for the hand-owned DCR+PKCE login flow (`cli/hedra-cli/custom.rs`).
//!
//! Each test spawns the real binary against a wiremock authorization server
//! and plays the browser itself: it reads the authorize URL the flow prints,
//! lifts the `state` out of it, and hits the loopback callback directly. That
//! exercises the full path — RFC 8414 discovery → RFC 7591 registration →
//! PKCE authorize/token exchange → keyring persistence — with no network and
//! no real browser.
//!
//! The child runs with `FERN_CLI_CREDENTIAL_STORE=file` and a temp `HOME`, so
//! the "keyring" is a JSON file this test can pre-seed and inspect, and the
//! host OS keychain is never touched.
//!
//! Tests that complete the PKCE dance bind the flow's pinned loopback port
//! (8484), so they are `#[serial]`.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use base64::Engine;
use serial_test::serial;
use sha2::Digest;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const BIN: &str = env!("CARGO_BIN_EXE_hedra-cli");
const REDIRECT_URI: &str = "http://127.0.0.1:8484/callback";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Where `FileKeyringStore::user_default()` puts the map for service
/// `hedra-cli`, given the `HOME` we hand the child. Mirrors
/// `oauth_common::config_dir()`.
fn keyring_file(home: &Path) -> PathBuf {
    #[cfg(target_os = "macos")]
    let config = home.join("Library").join("Application Support");
    #[cfg(target_os = "windows")]
    let config = home.join("AppData").join("Roaming");
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    let config = home.join(".config");
    config.join("hedra-cli").join("auth-keyring.json")
}

fn read_keyring_map(home: &Path) -> serde_json::Value {
    let raw = std::fs::read_to_string(keyring_file(home))
        .unwrap_or_else(|e| panic!("keyring file {}: {e}", keyring_file(home).display()));
    serde_json::from_str(&raw).expect("keyring file is a JSON map")
}

/// Spawn `hedra-cli auth login --no-browser` against `issuer`, hermetically.
fn spawn_login(issuer: &str, home: &Path) -> (Child, StderrLines) {
    let mut child = Command::new(BIN)
        .args(["auth", "login", "--no-browser"])
        .env_clear()
        .env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .env("FERN_CLI_CREDENTIAL_STORE", "file")
        .env("HEDRA_OAUTH_ISSUER", issuer)
        .env("NO_COLOR", "1")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn hedra-cli");
    let lines = StderrLines::tail(child.stderr.take().expect("stderr piped"));
    (child, lines)
}

/// Streams the child's stderr line-by-line through a channel so the test can
/// wait for specific progress lines with a deadline instead of blocking on a
/// full read of a process that is still listening for its callback.
struct StderrLines {
    rx: mpsc::Receiver<String>,
    seen: Vec<String>,
}

impl StderrLines {
    fn tail(stderr: impl Read + Send + 'static) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stderr).lines() {
                let Ok(line) = line else { break };
                if tx.send(line).is_err() {
                    break;
                }
            }
        });
        Self {
            rx,
            seen: Vec::new(),
        }
    }

    /// Wait until a line containing `needle` arrives; panics with everything
    /// seen so far on timeout.
    fn wait_for(&mut self, needle: &str, timeout: Duration) -> String {
        let deadline = Instant::now() + timeout;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            match self.rx.recv_timeout(remaining) {
                Ok(line) => {
                    self.seen.push(line.clone());
                    if line.contains(needle) {
                        return line;
                    }
                }
                Err(_) => panic!(
                    "timed out waiting for stderr line containing `{needle}`; saw:\n{}",
                    self.seen.join("\n")
                ),
            }
        }
    }
}

fn query_params(url: &str) -> Vec<(String, String)> {
    let qs = url.split_once('?').map(|(_, q)| q).unwrap_or("");
    form_urlencoded::parse(qs.as_bytes())
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect()
}

fn param<'a>(params: &'a [(String, String)], key: &str) -> Option<&'a str> {
    params
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
}

/// Act as the browser: deliver `code` + `state` to the flow's loopback
/// listener over a raw TCP GET.
fn hit_callback(state: &str, code: &str) {
    let mut sock = std::net::TcpStream::connect("127.0.0.1:8484").expect("connect to callback");
    // `state` is base64url from the flow's own generator — URL-safe as-is.
    let req = format!(
        "GET /callback?code={code}&state={state} HTTP/1.1\r\nHost: 127.0.0.1:8484\r\nConnection: close\r\n\r\n"
    );
    sock.write_all(req.as_bytes())
        .expect("write callback request");
    let mut resp = String::new();
    let _ = sock.read_to_string(&mut resp);
    assert!(
        resp.starts_with("HTTP/1.1 200"),
        "callback response was not 200:\n{resp}"
    );
}

fn wait_for_exit(child: &mut Child, timeout: Duration) -> std::process::ExitStatus {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait().expect("try_wait") {
            return status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("child did not exit within {timeout:?}");
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Drive one full login round against `server`: wait for the authorize URL,
/// verify its PKCE shape, deliver the callback, and require exit 0. Returns
/// the authorize URL's query parameters.
fn complete_login(server_uri: &str, home: &Path) -> Vec<(String, String)> {
    let (mut child, mut stderr) = spawn_login(server_uri, home);
    let url_line = stderr.wait_for("URL: ", Duration::from_secs(20));
    stderr.wait_for(
        "Listening on http://127.0.0.1:8484/callback",
        Duration::from_secs(10),
    );

    let url = url_line
        .split("URL: ")
        .nth(1)
        .expect("authorize URL after `URL: `");
    let params = query_params(url);
    assert_eq!(param(&params, "redirect_uri"), Some(REDIRECT_URI));
    assert_eq!(param(&params, "response_type"), Some("code"));
    assert_eq!(param(&params, "code_challenge_method"), Some("S256"));
    assert_eq!(param(&params, "scope"), Some("openid email offline_access"));

    hit_callback(param(&params, "state").expect("state param"), "test-code");
    let status = wait_for_exit(&mut child, Duration::from_secs(20));
    assert!(
        status.success(),
        "login exited with {status}; stderr:\n{}",
        stderr.seen.join("\n")
    );
    params
}

fn discovery_json(server_uri: &str, with_registration: bool) -> serde_json::Value {
    let mut meta = serde_json::json!({
        "issuer": server_uri,
        "authorization_endpoint": format!("{server_uri}/oauth2/authorize"),
        "token_endpoint": format!("{server_uri}/oauth2/token"),
        "code_challenge_methods_supported": ["S256"],
    });
    if with_registration {
        meta["registration_endpoint"] = serde_json::json!(format!("{server_uri}/oauth2/register"));
    }
    meta
}

async fn mount_discovery(server: &MockServer, with_registration: bool) {
    Mock::given(method("GET"))
        .and(path("/.well-known/oauth-authorization-server"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(discovery_json(&server.uri(), with_registration)),
        )
        .mount(server)
        .await;
}

async fn mount_token_endpoint(server: &MockServer) {
    Mock::given(method("POST"))
        .and(path("/oauth2/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "access_token": "at-wire-test",
            "refresh_token": "rt-wire-test",
            "token_type": "Bearer",
            "expires_in": 3600,
        })))
        .mount(server)
        .await;
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

/// First login mints a client via DCR, completes PKCE, and persists both the
/// registration and the token bundle; a second login reuses the registration
/// instead of registering again.
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn login_mints_client_completes_pkce_and_reuses_registration() {
    let server = MockServer::start().await;
    let home = tempfile::tempdir().expect("tempdir");

    mount_discovery(&server, true).await;
    mount_token_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/oauth2/register"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "client_id": "client_wire_test_1",
        })))
        .mount(&server)
        .await;
    // Preflight of a persisted client: a live client 302s into a login
    // session, not an error page.
    Mock::given(method("GET"))
        .and(path("/oauth2/authorize"))
        .respond_with(ResponseTemplate::new(302).insert_header(
            "location",
            format!(
                "{}/oauth2/authorize?authorization_session_id=abc",
                server.uri()
            ),
        ))
        .mount(&server)
        .await;

    // ── First login: mints the client ────────────────────────────────
    let params = complete_login(&server.uri(), home.path());
    assert_eq!(param(&params, "client_id"), Some("client_wire_test_1"));

    // The registration request carried the exact RFC 7591 body the AS was
    // proven to accept.
    let requests = server.received_requests().await.expect("recording");
    let register: Vec<_> = requests
        .iter()
        .filter(|r| r.method.as_str() == "POST" && r.url.path() == "/oauth2/register")
        .collect();
    assert_eq!(register.len(), 1, "expected exactly one DCR request");
    let body: serde_json::Value =
        serde_json::from_slice(&register[0].body).expect("DCR body is JSON");
    assert_eq!(body["client_name"], "hedra-cli");
    assert_eq!(body["redirect_uris"], serde_json::json!([REDIRECT_URI]));
    assert_eq!(
        body["grant_types"],
        serde_json::json!(["authorization_code", "refresh_token"])
    );
    assert_eq!(body["response_types"], serde_json::json!(["code"]));
    assert_eq!(body["token_endpoint_auth_method"], "none");

    // The token exchange used the minted client and a code_verifier that
    // actually hashes to the advertised code_challenge.
    let token_req = requests
        .iter()
        .find(|r| r.method.as_str() == "POST" && r.url.path() == "/oauth2/token")
        .expect("token exchange reached the AS");
    let form: Vec<(String, String)> = form_urlencoded::parse(&token_req.body)
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    assert_eq!(param(&form, "grant_type"), Some("authorization_code"));
    assert_eq!(param(&form, "client_id"), Some("client_wire_test_1"));
    assert_eq!(param(&form, "code"), Some("test-code"));
    assert_eq!(param(&form, "redirect_uri"), Some(REDIRECT_URI));
    let verifier = param(&form, "code_verifier").expect("code_verifier in token exchange");
    let expected_challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(sha2::Sha256::digest(verifier.as_bytes()));
    assert_eq!(
        param(&params, "code_challenge"),
        Some(expected_challenge.as_str())
    );

    // Registration and token bundle both persisted, in separate accounts.
    let map = read_keyring_map(home.path());
    let dcr: serde_json::Value =
        serde_json::from_str(map["BearerToken.dcr"].as_str().expect("DCR entry"))
            .expect("DCR entry is JSON");
    assert_eq!(dcr["client_id"], "client_wire_test_1");
    assert_eq!(dcr["issuer"], server.uri());
    assert_eq!(dcr["token_url"], format!("{}/oauth2/token", server.uri()));
    let bundle: serde_json::Value =
        serde_json::from_str(map["BearerToken"].as_str().expect("token entry"))
            .expect("bundle is JSON");
    assert_eq!(bundle["access_token"], "at-wire-test");
    assert_eq!(bundle["refresh_token"], "rt-wire-test");

    // ── Second login: reuses the persisted registration ──────────────
    let params = complete_login(&server.uri(), home.path());
    assert_eq!(param(&params, "client_id"), Some("client_wire_test_1"));
    let requests = server.received_requests().await.expect("recording");
    let register_count = requests
        .iter()
        .filter(|r| r.method.as_str() == "POST" && r.url.path() == "/oauth2/register")
        .count();
    assert_eq!(register_count, 1, "second login must not re-register");
}

/// A persisted client the AS no longer knows (authorize preflight 302s to
/// `error=application_not_found`) is re-registered once, and login proceeds
/// with the fresh client.
#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn evicted_client_is_reregistered_once() {
    let server = MockServer::start().await;
    let home = tempfile::tempdir().expect("tempdir");

    mount_discovery(&server, true).await;
    mount_token_endpoint(&server).await;
    Mock::given(method("POST"))
        .and(path("/oauth2/register"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
            "client_id": "client_fresh",
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/oauth2/authorize"))
        .respond_with(ResponseTemplate::new(302).insert_header(
            "location",
            format!("{}/oauth2/error?error=application_not_found", server.uri()),
        ))
        .mount(&server)
        .await;

    // Pre-seed a registration the AS will disown.
    let file = keyring_file(home.path());
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    let dead = serde_json::json!({
        "client_id": "client_dead",
        "issuer": server.uri(),
        "token_url": format!("{}/oauth2/token", server.uri()),
    });
    std::fs::write(
        &file,
        serde_json::to_string(&serde_json::json!({ "BearerToken.dcr": dead.to_string() })).unwrap(),
    )
    .unwrap();

    let params = complete_login(&server.uri(), home.path());
    assert_eq!(param(&params, "client_id"), Some("client_fresh"));

    let map = read_keyring_map(home.path());
    let dcr: serde_json::Value =
        serde_json::from_str(map["BearerToken.dcr"].as_str().unwrap()).unwrap();
    assert_eq!(dcr["client_id"], "client_fresh");
}

/// Live smoke test against Hedra's staging AuthKit — the prototype's manual
/// verification, kept runnable:
///
/// ```text
/// cargo test --test dcr_login_wire -- --ignored --nocapture
/// ```
///
/// Asserts DCR mints and persists a client and the flow reaches the
/// listening-for-callback state, then kills the child — it never completes
/// the browser step. Each run registers a fresh client (tempdir HOME).
#[test]
#[serial]
#[ignore = "hits the live staging AuthKit"]
fn staging_smoke_mints_client_and_binds_listener() {
    const STAGING_ISSUER: &str = "https://upbeat-skate-34-staging.authkit.app";
    let home = tempfile::tempdir().expect("tempdir");

    let (mut child, mut stderr) = spawn_login(STAGING_ISSUER, home.path());
    stderr.wait_for("Registered OAuth client", Duration::from_secs(30));
    let url_line = stderr.wait_for("URL: ", Duration::from_secs(10));
    stderr.wait_for(
        "Listening on http://127.0.0.1:8484/callback",
        Duration::from_secs(10),
    );
    let _ = child.kill();
    let _ = child.wait();

    let url = url_line
        .split("URL: ")
        .nth(1)
        .expect("authorize URL after `URL: `");
    assert!(
        url.starts_with(&format!("{STAGING_ISSUER}/oauth2/authorize?")),
        "authorize URL points at staging: {url}"
    );
    let params = query_params(url);
    let client_id = param(&params, "client_id")
        .expect("client_id in authorize URL")
        .to_string();
    assert!(
        client_id.starts_with("client_"),
        "expected an AuthKit client id, got `{client_id}`"
    );
    assert_eq!(param(&params, "redirect_uri"), Some(REDIRECT_URI));

    let map = read_keyring_map(home.path());
    let dcr: serde_json::Value =
        serde_json::from_str(map["BearerToken.dcr"].as_str().expect("DCR entry")).unwrap();
    assert_eq!(dcr["client_id"], client_id.as_str());
    assert_eq!(dcr["issuer"], STAGING_ISSUER);
    assert_eq!(dcr["token_url"], format!("{STAGING_ISSUER}/oauth2/token"));

    println!("── staging login (killed before the browser step) ──");
    for line in &stderr.seen {
        println!("{line}");
    }
    println!("── persisted registration (BearerToken.dcr) ──");
    println!("{}", serde_json::to_string_pretty(&dcr).unwrap());
}

/// An AS that advertises no `registration_endpoint` fails the login cleanly —
/// no listener, no panic, an actionable message.
#[tokio::test(flavor = "multi_thread")]
async fn missing_registration_endpoint_fails_cleanly() {
    let server = MockServer::start().await;
    let home = tempfile::tempdir().expect("tempdir");

    mount_discovery(&server, false).await;

    let (mut child, mut stderr) = spawn_login(&server.uri(), home.path());
    let status = wait_for_exit(&mut child, Duration::from_secs(20));
    assert!(
        !status.success(),
        "login must fail without a registration_endpoint"
    );
    stderr.wait_for("registration_endpoint", Duration::from_secs(5));
}
