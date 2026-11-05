//! The generated `hedra-cli sandbox` commands, driving the **real compiled
//! binary** against a mock v3 API.
//!
//! The commands are generated from the embedded spec, so these tests pin what
//! a regeneration could change without anyone noticing: the command paths the
//! hooks in `cli/hedra-cli/custom.rs` name (a renamed path makes `hide` panic
//! or the table hook fail validation), the request each flag sends, and the
//! table a single sandbox renders as.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry).

use std::process::{Command, Output, Stdio};

use serde_json::{json, Value};
use tempfile::TempDir;
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

/// What one invocation did: the process result and every request the mock
/// server received.
struct Outcome {
    out: Output,
    requests: Vec<Request>,
}

impl Outcome {
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.out.stdout).into_owned()
    }

    fn output_text(&self) -> String {
        format!(
            "exit {:?}\nstdout:\n{}\nstderr:\n{}",
            self.out.status.code(),
            self.stdout(),
            String::from_utf8_lossy(&self.out.stderr)
        )
    }

    /// The single request the server received.
    fn request(&self) -> &Request {
        assert_eq!(self.requests.len(), 1, "{}", self.output_text());
        &self.requests[0]
    }

    /// The JSON body of the single request the server received.
    fn body(&self) -> Value {
        serde_json::from_slice(&self.request().body).expect("request body is JSON")
    }
}

/// A sandbox as `GET /sandboxes/{sandbox}` answers it, with every array field
/// non-empty, which is what made the generic table print only one of them.
fn sandbox() -> Value {
    json!({
        "id": "sandbox_0123456789abcdef",
        "name": "train",
        "status": "active",
        "url": "https://train--acme.example.com",
        "gpu": "h100:1",
        "gpu_model": "H100",
        "replicas": 1,
        "end_reason": null,
        "started_at": "2026-11-05T00:00:30Z",
        "ended_at": null,
        "ttl_ends_at": null,
        "billing": {
            "usd_per_hour": 2.5,
            "paid_usd": 2.5,
            "charged_usd": 0.1,
            "settled": false
        },
        "image": "ghcr.io/acme/trainer:1",
        "command": ["python", "train.py"],
        "env_keys": ["WANDB_PROJECT"],
        "instances": [{"replica": 0, "status": "running"}],
        "created_at": "2026-11-05T00:00:00Z"
    })
}

/// Run `hedra-cli <args> --base-url <mock>` in an isolated HOME with a key in
/// the environment. `responses` answer the requests in order; the last one
/// answers every request after it.
fn run(args: &[&str], responses: Vec<ResponseTemplate>) -> Outcome {
    let home = TempDir::new().expect("temp HOME");
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    rt.block_on(async {
        let server = MockServer::start().await;
        let count = responses.len();
        for (i, response) in responses.into_iter().enumerate() {
            let mock = Mock::given(wiremock::matchers::any()).respond_with(response);
            let mock = if i + 1 < count {
                mock.up_to_n_times(1).with_priority(1)
            } else {
                mock.with_priority(2)
            };
            mock.mount(&server).await;
        }

        let uri = server.uri();
        let out = Command::new(env!("CARGO_BIN_EXE_hedra-cli"))
            .args(args)
            .args(["--base-url", &uri])
            .current_dir(home.path())
            .env("HOME", home.path())
            .env("FERN_CLI_CREDENTIAL_STORE", "file")
            .env("HEDRA_API_KEY", "test-key")
            .env("NO_COLOR", "1")
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("APPDATA")
            .env_remove("HEDRA_ENV")
            .env_remove("HEDRA_CLI_BASE_URL")
            .env_remove("HEDRA_CLI_OUTPUT")
            .stdin(Stdio::null())
            .output()
            .expect("run hedra-cli");

        let requests = server
            .received_requests()
            .await
            .expect("mock server recorded requests");
        Outcome { out, requests }
    })
}

fn ok(body: Value) -> Vec<ResponseTemplate> {
    vec![ResponseTemplate::new(200).set_body_json(body)]
}

/// `sandbox create` with the required flags plus `extra`, answered 201.
fn create(extra: &[&str]) -> Outcome {
    let mut args = vec![
        "sandbox",
        "create",
        "--name",
        "train",
        "--image",
        "ghcr.io/acme/trainer:1",
        "--gpu",
        "h100:1",
        "--format",
        "json",
    ];
    args.extend_from_slice(extra);
    run(
        &args,
        vec![ResponseTemplate::new(201).set_body_json(sandbox())],
    )
}

/// Run the binary with no server, for commands that send nothing.
fn offline(args: &[&str]) -> Output {
    let home = TempDir::new().expect("temp HOME");
    Command::new(env!("CARGO_BIN_EXE_hedra-cli"))
        .args(args)
        .current_dir(home.path())
        .env("HOME", home.path())
        .env("FERN_CLI_CREDENTIAL_STORE", "file")
        .env("NO_COLOR", "1")
        .env_remove("HEDRA_API_KEY")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("APPDATA")
        .env_remove("HEDRA_ENV")
        .env_remove("HEDRA_CLI_BASE_URL")
        .env_remove("HEDRA_CLI_OUTPUT")
        .stdin(Stdio::null())
        .output()
        .expect("run hedra-cli")
}

fn text(out: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

// ---------------------------------------------------------------------------
// The command tree the hooks name
// ---------------------------------------------------------------------------

/// `hide(&["sandbox", "session"])` panics on a path a regeneration renamed,
/// and the table-only `["sandbox", "*"]` hook fails validation when no leaf
/// matches it. Building the tree piped and with `--format table` runs both.
#[test]
fn the_command_tree_builds_piped_and_as_a_table() {
    for args in [
        &["sandbox", "--help"][..],
        &["--format", "table", "sandbox", "--help"][..],
        &["sandbox", "session", "--help"][..],
        &["--format", "table", "sandbox", "get", "--help"][..],
    ] {
        let out = offline(args);
        assert!(out.status.success(), "{args:?}\n{}", text(&out));
        assert!(
            !text(&out).contains("matches no operations"),
            "{args:?}\n{}",
            text(&out)
        );
    }
}

#[test]
fn session_is_hidden_from_help_but_still_runs() {
    let help = offline(&["sandbox", "--help"]);
    let listed = String::from_utf8_lossy(&help.stdout);
    assert!(listed.contains("create"), "{}", text(&help));
    assert!(
        !listed
            .lines()
            .any(|line| line.trim_start().starts_with("session")),
        "`session` is listed:\n{listed}"
    );

    let outcome = run(
        &[
            "sandbox",
            "session",
            "--sandbox",
            "train",
            "--format",
            "json",
        ],
        ok(
            json!({"token": "t", "broker_url": "b:443", "expires_at": "2026-11-05T00:10:00Z", "replica": 0}),
        ),
    );
    assert!(outcome.out.status.success(), "{}", outcome.output_text());
    assert_eq!(outcome.request().url.path(), "/sandboxes/train/session");
}

// ---------------------------------------------------------------------------
// create: each request field as JSON
// ---------------------------------------------------------------------------

#[test]
fn create_sends_command_as_a_list_of_strings() {
    let outcome = create(&["--command", r#"["python","train.py","--port","8000"]"#]);
    assert_eq!(
        outcome.body()["command"],
        json!(["python", "train.py", "--port", "8000"])
    );
}

#[test]
fn create_sends_a_repeated_command_flag_as_a_list() {
    let outcome = create(&["--command", "python", "--command", "train.py"]);
    assert_eq!(outcome.body()["command"], json!(["python", "train.py"]));
}

#[test]
fn create_sends_command_null() {
    let outcome = create(&["--command", "null"]);
    let body = outcome.body();
    assert!(body.get("command").is_some_and(Value::is_null), "{body}");
}

#[test]
fn create_sends_ports_as_a_list_of_objects() {
    let outcome = create(&["--ports", r#"[{"port":8000,"health":"/healthz"}]"#]);
    assert_eq!(
        outcome.body()["ports"],
        json!([{"port": 8000, "health": "/healthz"}])
    );
}

#[test]
fn create_sends_ports_null() {
    let outcome = create(&["--ports", "null"]);
    let body = outcome.body();
    assert!(body.get("ports").is_some_and(Value::is_null), "{body}");
}

#[test]
fn create_sends_env_as_a_string_map() {
    let outcome = create(&["--env", r#"{"WANDB_PROJECT":"train","EPOCHS":"3"}"#]);
    assert_eq!(
        outcome.body()["env"],
        json!({"WANDB_PROJECT": "train", "EPOCHS": "3"})
    );
}

#[test]
fn create_sends_resources_as_an_object() {
    let outcome = create(&[
        "--resources",
        r#"{"cpu":8,"memory_gb":64,"scratch_gb":100,"shm_gb":16}"#,
    ]);
    assert_eq!(
        outcome.body()["resources"],
        json!({"cpu": 8, "memory_gb": 64, "scratch_gb": 100, "shm_gb": 16})
    );
}

#[test]
fn create_sends_resource_fields_given_one_by_one_as_an_object() {
    let outcome = create(&["--resources.cpu", "8", "--resources.memory-gb", "64"]);
    assert_eq!(
        outcome.body()["resources"],
        json!({"cpu": 8, "memory_gb": 64})
    );
}

#[test]
fn create_sends_each_egress_mode_as_an_object() {
    for mode in ["all", "default", "none"] {
        let given = format!(r#"{{"mode":"{mode}"}}"#);
        let outcome = create(&["--egress", &given]);
        assert_eq!(outcome.body()["egress"], json!({"mode": mode}), "{mode}");
    }
}

#[test]
fn create_sends_egress_hosts_and_ranges_as_an_object() {
    let outcome = create(&[
        "--egress",
        r#"{"hosts":["api.github.com"],"cidrs":["203.0.113.0/24"],"ports":[443]}"#,
    ]);
    assert_eq!(
        outcome.body()["egress"],
        json!({"hosts": ["api.github.com"], "cidrs": ["203.0.113.0/24"], "ports": [443]})
    );
}

#[test]
fn create_sends_the_required_fields() {
    let outcome = create(&[]);
    let body = outcome.body();
    assert_eq!(outcome.request().url.path(), "/sandboxes");
    assert_eq!(body["name"], "train");
    assert_eq!(body["image"], "ghcr.io/acme/trainer:1");
    assert_eq!(body["gpu"], "h100:1");
    assert!(body.get("accelerator").is_none(), "{body}");
}

/// The runtime retries a POST on a 5xx only when the caller supplied the
/// key, and must send the same key on the retry.
#[test]
fn create_sends_the_idempotency_key_on_its_first_attempt_and_its_retry() {
    let outcome = run(
        &[
            "sandbox",
            "create",
            "--name",
            "train",
            "--image",
            "ghcr.io/acme/trainer:1",
            "--gpu",
            "h100:1",
            "--idempotency-key",
            "K-1",
            "--format",
            "json",
        ],
        vec![
            ResponseTemplate::new(503).set_body_json(json!({
                "error": {"type": "UNAVAILABLE", "message": "try again"}
            })),
            ResponseTemplate::new(201).set_body_json(sandbox()),
        ],
    );
    assert!(outcome.out.status.success(), "{}", outcome.output_text());
    assert_eq!(outcome.requests.len(), 2, "{}", outcome.output_text());
    for request in &outcome.requests {
        assert_eq!(
            request
                .headers
                .get("Idempotency-Key")
                .and_then(|v| v.to_str().ok()),
            Some("K-1")
        );
    }
}

// ---------------------------------------------------------------------------
// Filters and paths
// ---------------------------------------------------------------------------

fn query(request: &Request, key: &str) -> Option<String> {
    request
        .url
        .query_pairs()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.into_owned())
}

#[test]
fn ls_sends_its_filters() {
    let outcome = run(
        &[
            "sandbox", "ls", "--all", "true", "--name", "train", "--format", "json",
        ],
        ok(json!({"data": [], "has_more": false, "next_cursor": null})),
    );
    let request = outcome.request();
    assert_eq!(request.url.path(), "/sandboxes");
    assert_eq!(query(request, "all").as_deref(), Some("true"));
    assert_eq!(query(request, "name").as_deref(), Some("train"));
}

#[test]
fn saved_logs_sends_since() {
    let outcome = run(
        &[
            "sandbox",
            "saved-logs",
            "--sandbox",
            "train",
            "--since",
            "2026-11-05T00:00:00Z",
            "--format",
            "json",
        ],
        ok(json!({"content": "", "truncated": false})),
    );
    let request = outcome.request();
    assert_eq!(request.url.path(), "/sandboxes/train/logs");
    assert_eq!(
        query(request, "since").as_deref(),
        Some("2026-11-05T00:00:00Z")
    );
}

#[test]
fn get_given_a_name_requests_the_name_path() {
    let outcome = run(
        &["sandbox", "get", "--sandbox", "train", "--format", "json"],
        ok(sandbox()),
    );
    assert_eq!(outcome.request().url.path(), "/sandboxes/train");
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Without the table hook, the generic table prints only the first non-empty
/// array of a single object (`command`), and none of the sandbox's own fields.
#[test]
fn get_and_create_print_the_sandboxs_own_fields_as_a_table() {
    let get = run(
        &["sandbox", "get", "--sandbox", "train", "--format", "table"],
        ok(sandbox()),
    );
    let created = run(
        &[
            "sandbox",
            "create",
            "--name",
            "train",
            "--image",
            "ghcr.io/acme/trainer:1",
            "--gpu",
            "h100:1",
            "--format",
            "table",
        ],
        vec![ResponseTemplate::new(201).set_body_json(sandbox())],
    );
    for outcome in [get, created] {
        assert!(outcome.out.status.success(), "{}", outcome.output_text());
        let table = outcome.stdout();
        for field in [
            "sandbox_0123456789abcdef",
            "train",
            "active",
            "https://train--acme.example.com",
        ] {
            assert!(table.contains(field), "`{field}` missing:\n{table}");
        }
    }
}

/// The hook runs only for a table: JSON stays the server's answer.
#[test]
fn get_as_json_is_the_servers_answer() {
    let outcome = run(
        &["sandbox", "get", "--sandbox", "train", "--format", "json"],
        ok(sandbox()),
    );
    let printed: Value = serde_json::from_str(&outcome.stdout()).expect("stdout is JSON");
    assert_eq!(printed, sandbox());
}

#[test]
fn a_piped_error_prints_the_error_json_on_stdout() {
    let outcome = run(
        &["sandbox", "get", "--sandbox", "train"],
        vec![ResponseTemplate::new(404).set_body_json(json!({
            "error": {"type": "NOT_FOUND", "message": "Sandbox not found."}
        }))],
    );
    assert!(!outcome.out.status.success(), "{}", outcome.output_text());
    let printed: Value = serde_json::from_str(outcome.stdout().trim())
        .unwrap_or_else(|e| panic!("stdout is not JSON ({e}):\n{}", outcome.output_text()));
    assert!(
        printed.to_string().contains("Sandbox not found."),
        "{}",
        outcome.output_text()
    );
}

/// The table hook leaves a list envelope alone, so `ls` still prints one row
/// per sandbox rather than indexed keys.
#[test]
fn ls_as_a_table_prints_a_row_per_sandbox() {
    let mut second = sandbox();
    second["id"] = json!("sandbox_fedcba9876543210");
    second["name"] = json!("serve");
    let outcome = run(
        &["sandbox", "ls", "--format", "table"],
        ok(json!({"data": [sandbox(), second], "next_cursor": null})),
    );
    assert!(outcome.out.status.success(), "{}", outcome.output_text());
    let table = outcome.stdout();
    assert!(
        !table.contains("data.1"),
        "rows became indexed keys:\n{table}"
    );
    let rows: Vec<&str> = table
        .lines()
        .filter(|line| line.contains("sandbox_"))
        .collect();
    assert_eq!(rows.len(), 2, "{table}");
}
