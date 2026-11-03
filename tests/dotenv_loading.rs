//! End-to-end test that a `.env` cannot route the CLI's traffic through a
//! proxy, driving the **real compiled binary** from a directory holding one.
//!
//! The filter itself is unit-tested in `src/lib.rs`. This file covers what
//! those tests cannot: that `custom::register` loads `.env` through the
//! filter at all. It used to call `dotenvy::dotenv()` first, which set every
//! key before the runtime's filtered loader ran, so the deny-list never
//! applied; and the deny-list matched only upper case, so `http_proxy` passed
//! it anyway.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry), with no new
//! dev-dependency: `CARGO_BIN_EXE_hedra-cli` plus the present `tempfile` and
//! `wiremock`, as in `auth_commands.rs`.

use std::process::{Command, Stdio};

use tempfile::TempDir;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A port nothing listens on, so a request sent through the proxy fails.
const DEAD_PROXY: &str = "http://127.0.0.1:9";

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
