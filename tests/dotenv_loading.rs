//! End-to-end tests of what a `.env` can and cannot change, driving the
//! **real compiled binary** from a directory holding one: it cannot route the
//! CLI's traffic through a proxy or redirect `auth login`, and its
//! `HEDRA_ENV` still retargets the data plane.
//!
//! The allow-list itself is unit-tested in `cli/hedra-cli/dotenv.rs`. This
//! file covers what those tests cannot: that `custom::register` loads `.env`
//! through it at all, early enough for `HEDRA_ENV`, and that the runtime's own
//! `.env` loader, which admits far more, never runs. `register` used to call
//! `dotenvy::dotenv()`, which set every key in the `.env` (ENG-12596).
//!
//! Hand-written and .fernignore-protected (the `tests/` entry), with no new
//! dev-dependency: `CARGO_BIN_EXE_hedra-cli` plus the present `tempfile` and
//! `wiremock`, as in `auth_commands.rs`.

use std::io::Read;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A port nothing listens on, so a request sent through the proxy fails.
const DEAD_PROXY: &str = "http://127.0.0.1:9";

/// A data-plane base URL on that port, so login discovery fails fast.
const DEAD_BASE_URL: &str = "http://127.0.0.1:9/v3";

const PROXY_VARIABLES: [&str; 8] = [
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
    "no_proxy",
];

#[test]
fn a_dotenv_proxy_in_any_case_does_not_reach_the_transport() {
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({ "data": [] })),
            )
            .mount(&server)
            .await;

        // The mock is plain http, and reqwest takes `http_proxy` for an http
        // URL, so the lowercase key is the one that would reroute it.
        let project = TempDir::new().expect("project dir");
        std::fs::write(
            project.path().join(".env"),
            format!("http_proxy={DEAD_PROXY}\nAll_Proxy={DEAD_PROXY}\nSSL_CERT_DIR=/nonexistent\n"),
        )
        .expect("write .env");
        let home = TempDir::new().expect("temp HOME");

        let mut cmd = Command::new(env!("CARGO_BIN_EXE_hedra-cli"));
        cmd.args(["--format", "json", "models", "list"])
            .current_dir(project.path())
            .env("HOME", home.path())
            .env("FERN_CLI_CREDENTIAL_STORE", "file")
            .env("NO_COLOR", "1")
            // Exported here, where it is allowed: the filter drops it from .env.
            .env("HEDRA_CLI_BASE_URL", server.uri())
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("APPDATA")
            .env_remove("HEDRA_API_KEY")
            .env_remove("HEDRA_ENV")
            .env_remove("SSL_CERT_DIR")
            .stdin(Stdio::null());
        for name in PROXY_VARIABLES {
            cmd.env_remove(name);
        }
        let out = cmd.output().expect("run hedra-cli");

        let requests = server
            .received_requests()
            .await
            .expect("mock server recorded requests");
        assert_eq!(
            requests.len(),
            1,
            "the request did not reach the server directly; exit {:?}\nstderr:\n{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr),
        );
    });
}

/// `register` must load `.env` before `derive_base_url_from_hedra_env` reads
/// `HEDRA_ENV`, or a `.env`-only `HEDRA_ENV=staging` sends the login plane to
/// staging and the data plane to prod.
#[test]
fn a_dotenv_hedra_env_retargets_the_data_plane() {
    let project = TempDir::new().expect("project dir");
    std::fs::write(project.path().join(".env"), "HEDRA_ENV=staging\n").expect("write .env");
    let home = TempDir::new().expect("temp HOME");
    let out = Command::new(env!("CARGO_BIN_EXE_hedra-cli"))
        .args(["--dry-run", "models", "list"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .env("FERN_CLI_CREDENTIAL_STORE", "file")
        .env("NO_COLOR", "1")
        .env_remove("HEDRA_ENV")
        .env_remove("HEDRA_CLI_BASE_URL")
        .env_remove("HEDRA_API_KEY")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("APPDATA")
        .stdin(Stdio::null())
        .output()
        .expect("run hedra-cli");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stdout.contains("\"url\": \"https://api.staging.hedra.com/v3/models\""),
        "stdout:\n{stdout}\nstderr:\n{stderr}",
    );
    // Nothing in this `.env` is ignored, so nothing is reported.
    assert!(!stderr.contains(IGNORED_LINE), "stderr:\n{stderr}");
}

/// The start of the line `register` prints when it ignores `.env` keys.
const IGNORED_LINE: &str = "hedra-cli: ignored from .env (export them from your shell instead):";

/// Run `--version` from a directory whose `.env` holds `dotenv`, with
/// `exported` in the process environment, and return stderr.
fn version_stderr(dotenv: &str, exported: &[(&str, &str)]) -> String {
    let project = TempDir::new().expect("project dir");
    std::fs::write(project.path().join(".env"), dotenv).expect("write .env");
    let home = TempDir::new().expect("temp HOME");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hedra-cli"));
    for name in PROXY_VARIABLES {
        cmd.env_remove(name);
    }
    for (key, value) in exported {
        cmd.env(key, value);
    }
    let out = cmd
        .arg("--version")
        .current_dir(project.path())
        .env("HOME", home.path())
        .env("FERN_CLI_CREDENTIAL_STORE", "file")
        .env_remove("HEDRA_CLI_LOG")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("APPDATA")
        .stdin(Stdio::null())
        .output()
        .expect("run hedra-cli");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Ignored keys meant for the CLI are named once on stderr by default,
/// without `HEDRA_CLI_LOG`; an ignored key for another program is not.
#[test]
fn ignored_dotenv_keys_are_named_on_stderr() {
    let stderr = version_stderr(
        "HEDRA_CLI_OUTPUT=json\nRUST_LOG=trace\nhttps_proxy=http://127.0.0.1:9\n\
         Hedra_Cli_Insecure=1\n",
        &[],
    );
    let expected = format!("{IGNORED_LINE} https_proxy, Hedra_Cli_Insecure\n");
    assert_eq!(stderr.matches(&expected).count(), 1, "stderr:\n{stderr}");
}

/// A project `.env` holding only other programs' keys prints nothing.
#[test]
fn unrelated_dotenv_keys_are_not_named_on_stderr() {
    let stderr = version_stderr(
        "DATABASE_URL=postgres://localhost/app\nNODE_ENV=development\n",
        &[],
    );
    assert!(!stderr.contains(IGNORED_LINE), "stderr:\n{stderr}");
}

/// A key the shell already exports is not named: the exported value applies.
#[test]
fn an_exported_key_is_not_named_on_stderr() {
    let stderr = version_stderr(
        &format!("HTTPS_PROXY={DEAD_PROXY}\n"),
        &[("HTTPS_PROXY", DEAD_PROXY)],
    );
    assert!(!stderr.contains(IGNORED_LINE), "stderr:\n{stderr}");
}

const ENDPOINT_OVERRIDE_DOTENV: &str = "HEDRA_AUTH_AUTHORIZE_URL=https://dotenv.example/authorize\n\
                                        HEDRA_AUTH_TOKEN_URL=https://dotenv.example/token\n";

/// Output a login prints once it is waiting for the browser.
const LISTENING: &str = "Listening on";

/// Run `auth login --no-browser` from a directory whose `.env` sets both login
/// endpoint overrides, with `HEDRA_CLI_BASE_URL` at a dead port so discovery
/// fails fast. Returns stdout and stderr together. A login that gets as far
/// as waiting for the browser is killed once it says so, or after a deadline.
fn login_output(exported: &[(&str, &str)]) -> String {
    let project = TempDir::new().expect("project dir");
    std::fs::write(project.path().join(".env"), ENDPOINT_OVERRIDE_DOTENV).expect("write .env");
    let home = TempDir::new().expect("temp HOME");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hedra-cli"));
    cmd.args(["auth", "login", "--no-browser"])
        .current_dir(project.path())
        .env("HOME", home.path())
        .env("FERN_CLI_CREDENTIAL_STORE", "file")
        .env("NO_COLOR", "1")
        .env("HEDRA_CLI_BASE_URL", DEAD_BASE_URL)
        .env_remove("HEDRA_AUTH_AUTHORIZE_URL")
        .env_remove("HEDRA_AUTH_TOKEN_URL")
        .env_remove("HEDRA_API_KEY")
        .env_remove("HEDRA_ENV")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("APPDATA")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    for (key, value) in exported {
        cmd.env(key, value);
    }
    let mut child = cmd.spawn().expect("run hedra-cli");

    let output = Arc::new(Mutex::new(String::new()));
    let readers: Vec<_> = [
        Box::new(child.stdout.take().expect("stdout")) as Box<dyn Read + Send>,
        Box::new(child.stderr.take().expect("stderr")),
    ]
    .into_iter()
    .map(|mut stream| {
        let output = Arc::clone(&output);
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = stream.read(&mut buf) {
                if n == 0 {
                    break;
                }
                output
                    .lock()
                    .unwrap()
                    .push_str(&String::from_utf8_lossy(&buf[..n]));
            }
        })
    })
    .collect();

    let deadline = Instant::now() + Duration::from_secs(30);
    while child.try_wait().expect("poll hedra-cli").is_none() {
        if Instant::now() > deadline || output.lock().unwrap().contains(LISTENING) {
            let _ = child.kill();
            let _ = child.wait();
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    for reader in readers {
        reader.join().expect("output reader");
    }
    let output = output.lock().unwrap().clone();
    output
}

/// The runtime's `.env` filter admits the login endpoint overrides, so
/// `custom::register` ignores them when a `.env` set them: the login goes to
/// discovery instead, which fails at the dead base URL.
#[test]
fn a_dotenv_cannot_redirect_auth_login() {
    let output = login_output(&[]);
    assert!(!output.contains("dotenv.example"), "output:\n{output}");
    assert!(
        output.contains(&format!(
            "{IGNORED_LINE} HEDRA_AUTH_AUTHORIZE_URL, HEDRA_AUTH_TOKEN_URL"
        )),
        "output:\n{output}"
    );
    assert!(
        output.contains("/.well-known/oauth-protected-resource"),
        "login did not fall back to discovery; output:\n{output}"
    );
}

/// Exported from the shell, the overrides still apply, and the `.env` values
/// do not replace them.
#[test]
fn exported_endpoint_overrides_still_redirect_auth_login() {
    let output = login_output(&[
        ("HEDRA_AUTH_AUTHORIZE_URL", "https://shell.example/authorize"),
        ("HEDRA_AUTH_TOKEN_URL", "https://shell.example/token"),
    ]);
    assert!(
        output.contains("https://shell.example/authorize?"),
        "output:\n{output}"
    );
    assert!(!output.contains("dotenv.example"), "output:\n{output}");
    assert!(!output.contains(IGNORED_LINE), "output:\n{output}");
}
