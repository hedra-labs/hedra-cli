//! The generated `hedra-cli sandbox` commands, driving the **real compiled
//! binary** against a mock v3 API.
//!
//! The commands are generated from the embedded spec, so these tests pin what
//! a regeneration could change without anyone noticing: the command paths the
//! hooks in `cli/hedra-cli/custom.rs` name (a renamed path makes `hide` panic,
//! which fails every test here, or makes the table hook fail validation, which
//! the command-tree test catches), the request each flag sends, and the table
//! a single sandbox renders as.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry).

use std::path::Path;
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
        describe(&self.out)
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
/// `name` is a value no other field contains, so a table that drops it fails.
fn sandbox() -> Value {
    json!({
        "id": "sandbox_0123456789abcdef",
        "name": "nightly",
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

/// A sandbox as `POST /sandboxes` answers it: still `creating`, with no
/// `url`, `image` or `started_at` yet.
fn created_sandbox() -> Value {
    let mut created = sandbox();
    created["status"] = json!("creating");
    created["url"] = Value::Null;
    created["image"] = Value::Null;
    created["started_at"] = Value::Null;
    created["instances"] = json!([]);
    created
}

/// Variables a developer's shell may set that would send the binary's
/// requests somewhere else.
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

/// The binary with `home` as its home and working directory, and none of the
/// environment it would otherwise inherit that could change what a test
/// sees: every `HEDRA_*` variable in any case, the proxy variables and the
/// per-user config directories.
fn hedra_cli(home: &Path) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_hedra-cli"));
    for (name, _) in std::env::vars_os() {
        if name
            .to_str()
            .is_some_and(|name| name.to_ascii_uppercase().starts_with("HEDRA_"))
        {
            cmd.env_remove(name);
        }
    }
    for name in PROXY_VARIABLES {
        cmd.env_remove(name);
    }
    cmd.current_dir(home)
        .env("HOME", home)
        .env("FERN_CLI_CREDENTIAL_STORE", "file")
        .env("NO_COLOR", "1")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("APPDATA")
        .stdin(Stdio::null());
    cmd
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
        let out = hedra_cli(home.path())
            .args(args)
            .args(["--base-url", &uri])
            .env("HEDRA_API_KEY", "test-key")
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

/// `sandbox create` with the required flags plus `extra`, answered by
/// `responses` in order.
fn create_with(extra: &[&str], responses: Vec<ResponseTemplate>) -> Outcome {
    let mut args = vec![
        "sandbox",
        "create",
        "--name",
        "nightly",
        "--image",
        "ghcr.io/acme/trainer:1",
        "--gpu",
        "h100:1",
        "--ttl-seconds",
        "900",
    ];
    args.extend_from_slice(extra);
    run(&args, responses)
}

/// `sandbox create --format json` with the required flags plus `extra`,
/// answered 201.
fn create(extra: &[&str]) -> Outcome {
    let mut extra = extra.to_vec();
    extra.extend_from_slice(&["--format", "json"]);
    create_with(
        &extra,
        vec![ResponseTemplate::new(201).set_body_json(created_sandbox())],
    )
}

/// Run the binary with no server and no key, for commands that send
/// nothing, with `env` set on top of the isolated environment.
fn offline_with(args: &[&str], env: &[(&str, &str)]) -> Output {
    let home = TempDir::new().expect("temp HOME");
    let mut cmd = hedra_cli(home.path());
    cmd.args(args);
    for (name, value) in env {
        cmd.env(name, value);
    }
    cmd.output().expect("run hedra-cli")
}

fn describe(out: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// A `--help` run reads its format from `HEDRA_CLI_OUTPUT`, then the TTY,
/// because the format flag is not parsed yet. Piped, that is JSON; on a
/// terminal it is a table, which `HEDRA_CLI_OUTPUT=table` stands in for here.
/// A person reads `--help` on a terminal, and a script reads it piped, so
/// every hook check runs under both.
const HELP_FORMATS: [(&str, &[(&str, &str)]); 2] = [
    ("piped", &[]),
    ("a terminal", &[("HEDRA_CLI_OUTPUT", "table")]),
];

// ---------------------------------------------------------------------------
// The command tree the hooks name
// ---------------------------------------------------------------------------

/// `hide(&["sandbox", "session"])` panics on a path a regeneration renamed,
/// and the table-only `["sandbox", "*"]` hook fails validation, exiting
/// non-zero with "matches no operations", when no leaf matches it. Building
/// the tree piped and as on a terminal runs both.
#[test]
fn the_command_tree_builds_piped_and_as_a_table() {
    for (format, env) in HELP_FORMATS {
        for args in [
            &["sandbox", "--help"][..],
            &["sandbox", "session", "--help"][..],
            &["sandbox", "get", "--help"][..],
        ] {
            let out = offline_with(args, env);
            assert!(
                out.status.success(),
                "{args:?} {format}: the tree failed to build (a renamed path, or a hook \
                 pattern that matches no operations)\n{}",
                describe(&out)
            );
        }
    }
}

#[test]
fn session_is_hidden_from_help_but_still_runs() {
    for (format, env) in HELP_FORMATS {
        let help = offline_with(&["sandbox", "--help"], env);
        let listed = String::from_utf8_lossy(&help.stdout);
        assert!(listed.contains("create"), "{format}\n{}", describe(&help));
        assert!(
            !listed
                .lines()
                .any(|line| line.trim_start().starts_with("session")),
            "`session` is listed {format}:\n{listed}"
        );
    }

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
fn create_rejects_non_integer_lifetimes_from_every_input_without_sending() {
    for lifetime in [Value::Null, json!(120.5)] {
        let body = json!({"name": "nightly", "image": "python:3.12", "gpu": "h100:1", "ttl_seconds": lifetime});
        for input in ["--json", "--params"] {
            let outcome = run(
                &[
                    "sandbox",
                    "create",
                    input,
                    &body.to_string(),
                    "--format",
                    "json",
                ],
                ok(created_sandbox()),
            );
            assert!(
                !outcome.out.status.success(),
                "{input} {lifetime}: {}",
                outcome.output_text()
            );
            assert!(
                outcome.requests.is_empty(),
                "{input} {lifetime}: {}",
                outcome.output_text()
            );
            assert!(
                outcome.output_text().contains("ttl_seconds"),
                "{}",
                outcome.output_text()
            );
        }
        let outcome = run(
            &[
                "sandbox",
                "create",
                "--name",
                "nightly",
                "--image",
                "python:3.12",
                "--gpu",
                "h100:1",
                "--ttl-seconds",
                &lifetime.to_string(),
            ],
            ok(created_sandbox()),
        );
        assert!(!outcome.out.status.success(), "{}", outcome.output_text());
        assert!(outcome.requests.is_empty(), "{}", outcome.output_text());
    }
}

#[test]
fn create_requires_a_lifetime_including_dry_runs() {
    let body = json!({"name": "nightly", "image": "python:3.12", "gpu": "h100:1"}).to_string();
    for args in [
        vec!["--json", body.as_str()],
        vec!["--json", body.as_str(), "--dry-run"],
        vec![
            "--name",
            "nightly",
            "--image",
            "python:3.12",
            "--gpu",
            "h100:1",
        ],
    ] {
        let mut command = vec!["sandbox", "create"];
        command.extend(args);
        let outcome = run(&command, ok(created_sandbox()));
        assert!(!outcome.out.status.success(), "{}", outcome.output_text());
        assert!(outcome.requests.is_empty(), "{}", outcome.output_text());
        assert!(
            outcome.output_text().contains("ttl_seconds"),
            "{}",
            outcome.output_text()
        );
    }
}

#[test]
fn create_accepts_both_lifetime_boundaries() {
    for seconds in [120, 86400] {
        let body = json!({"name": "nightly", "image": "python:3.12", "gpu": "h100:1", "ttl_seconds": seconds});
        let outcome = run(
            &["sandbox", "create", "--json", &body.to_string()],
            ok(created_sandbox()),
        );
        assert!(outcome.out.status.success(), "{}", outcome.output_text());
        assert_eq!(outcome.body()["ttl_seconds"], seconds);
    }
}

#[test]
fn create_leaves_lifetime_range_validation_to_the_api() {
    let message = "Lifetime refused by the API; contact sales@hedra.com.";
    for seconds in [0, 119, 86401, 31536000] {
        let body = json!({"name": "nightly", "image": "python:3.12", "gpu": "h100:1", "ttl_seconds": seconds});
        let outcome = run(
            &["sandbox", "create", "--json", &body.to_string()],
            vec![ResponseTemplate::new(400).set_body_json(json!({
                "error": {"type": "INVALID_ARGUMENT", "message": message, "param": "ttl_seconds"}
            }))],
        );
        assert!(!outcome.out.status.success(), "{}", outcome.output_text());
        assert_eq!(outcome.body()["ttl_seconds"], seconds);
        assert!(
            outcome.output_text().contains(message),
            "{}",
            outcome.output_text()
        );
    }
}

#[test]
fn create_validates_a_body_read_from_stdin() {
    let home = TempDir::new().unwrap();
    let file = home.path().join("sandbox.json");
    std::fs::write(
        &file,
        r#"{"name":"nightly","image":"python:3.12","gpu":"h100:1","ttl_seconds":null}"#,
    )
    .unwrap();
    let out = hedra_cli(home.path())
        .args(["sandbox", "create", "--json", "-", "--dry-run"])
        .env("HEDRA_API_KEY", "test-key")
        .stdin(std::fs::File::open(file).unwrap())
        .output()
        .unwrap();
    assert!(!out.status.success(), "{}", describe(&out));
    assert!(
        describe(&out).contains("ttl_seconds"),
        "{}",
        describe(&out)
    );
}

#[test]
fn gpu_shell_dry_run_leaves_lifetime_range_validation_to_the_api() {
    for duration in ["119s", "86401s", "2d"] {
        let out = offline_with(
            &[
                "sandbox",
                "shell",
                "--gpu",
                "h100:1",
                "--image",
                "python:3.12",
                "--max-duration",
                duration,
                "--dry-run",
            ],
            &[("HEDRA_CLI_SANDBOX_PREVIEW", "1")],
        );
        assert!(out.status.success(), "{}", describe(&out));
    }
}

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
    assert_eq!(body["name"], "nightly");
    assert_eq!(body["image"], "ghcr.io/acme/trainer:1");
    assert_eq!(body["gpu"], "h100:1");
    assert!(body.get("accelerator").is_none(), "{body}");
    // The count is the route's default unless asked for.
    assert!(body.get("replicas").is_none(), "{body}");
}

/// `--replicas N` is a flag of the generated create, documented in its
/// help, and sends `replicas` as a number; so does `--json`.
#[test]
fn create_sends_replicas_from_its_flag_and_from_json() {
    let body = create(&["--replicas", "2"]).body();
    assert_eq!(body["replicas"], 2);
    let help = offline_with(&["sandbox", "create", "--help"], &[]);
    let help = String::from_utf8_lossy(&help.stdout);
    assert!(help.contains("--replicas <NUMBER>"), "{help}");
    let json =
        r#"{"name": "nightly", "image": "i", "gpu": "h100:1", "ttl_seconds": 900, "replicas": 3}"#;
    let body = run(
        &["sandbox", "create", "--json", json, "--format", "json"],
        vec![ResponseTemplate::new(201).set_body_json(created_sandbox())],
    )
    .body();
    assert_eq!(body["replicas"], 3);
}

fn idempotency_key(request: &Request) -> Option<String> {
    request
        .headers
        .get("Idempotency-Key")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
}

/// A 503, then the sandbox created.
fn unavailable_then_created() -> Vec<ResponseTemplate> {
    vec![
        ResponseTemplate::new(503).set_body_json(json!({
            "error": {"type": "UNAVAILABLE", "message": "try again"}
        })),
        ResponseTemplate::new(201).set_body_json(created_sandbox()),
    ]
}

/// `POST /sandboxes` is `x-fern-idempotent`, so the runtime retries it on a
/// 5xx with or without `--idempotency-key`; a key the caller supplies must be
/// the one sent on every attempt.
#[test]
fn create_sends_the_idempotency_key_on_its_first_attempt_and_its_retry() {
    let outcome = create_with(
        &["--idempotency-key", "K-1", "--format", "json"],
        unavailable_then_created(),
    );
    assert!(outcome.out.status.success(), "{}", outcome.output_text());
    assert_eq!(outcome.requests.len(), 2, "{}", outcome.output_text());
    for request in &outcome.requests {
        assert_eq!(idempotency_key(request).as_deref(), Some("K-1"));
    }
}

/// Without `--idempotency-key`, the runtime generates one key and sends it on
/// every attempt. A key per attempt would let one 503 create several billed
/// sandboxes.
#[test]
fn create_without_a_key_retries_with_one_generated_key() {
    let outcome = create_with(&["--format", "json"], unavailable_then_created());
    assert!(outcome.out.status.success(), "{}", outcome.output_text());
    assert_eq!(outcome.requests.len(), 2, "{}", outcome.output_text());
    let keys: Vec<Option<String>> = outcome.requests.iter().map(idempotency_key).collect();
    assert!(keys[0].is_some(), "no Idempotency-Key on the first attempt");
    assert_eq!(keys[0], keys[1], "the retry sent a different key");
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
        &["sandbox", "get", "--sandbox", "nightly", "--format", "json"],
        ok(sandbox()),
    );
    assert_eq!(outcome.request().url.path(), "/sandboxes/nightly");
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Without the table hook, the generic table prints only the first non-empty
/// array of a single object (`command`), and none of the sandbox's own fields.
#[test]
fn get_and_create_print_the_sandboxs_own_fields_as_a_table() {
    let get = run(
        &[
            "sandbox",
            "get",
            "--sandbox",
            "nightly",
            "--format",
            "table",
        ],
        ok(sandbox()),
    );
    let created = create_with(
        &["--format", "table"],
        vec![ResponseTemplate::new(201).set_body_json(created_sandbox())],
    );
    for (outcome, rows) in [
        (
            get,
            &[
                ["id", "sandbox_0123456789abcdef"],
                ["name", "nightly"],
                ["status", "active"],
                ["url", "https://train--acme.example.com"],
            ][..],
        ),
        (
            created,
            &[
                ["id", "sandbox_0123456789abcdef"],
                ["name", "nightly"],
                ["status", "creating"],
            ][..],
        ),
    ] {
        assert!(outcome.out.status.success(), "{}", outcome.output_text());
        let table = outcome.stdout();
        for row in rows {
            assert!(
                table
                    .lines()
                    .any(|line| line.split_whitespace().eq(row.iter().copied())),
                "no `{} {}` row:\n{table}",
                row[0],
                row[1]
            );
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

/// `sandbox ls` as a table is one narrow row per sandbox, its columns the
/// ones a person scans for, not every field of the sandbox flattened; an
/// empty list says so; `--format json` is the server's answer unchanged.
#[test]
fn ls_as_a_table_shows_only_the_scanned_columns() {
    let list = json!({"data": [sandbox()], "next_cursor": null});
    let outcome = run(&["sandbox", "ls", "--format", "table"], ok(list.clone()));
    assert!(outcome.out.status.success(), "{}", outcome.output_text());
    let table = outcome.stdout();
    let lines: Vec<&str> = table.trim_end().lines().collect();
    assert_eq!(lines.len(), 3, "{table}");
    assert_eq!(
        lines[0].split_whitespace().collect::<Vec<_>>(),
        ["created_at", "gpu", "id", "name", "status", "usd_per_hour"],
        "{table}"
    );
    let row: Vec<&str> = lines[2]
        .split("  ")
        .map(str::trim)
        .filter(|cell| !cell.is_empty())
        .collect();
    assert_eq!(
        row,
        [
            "2026-11-05 00:00:00 UTC",
            "h100:1",
            "sandbox_0123456789abcdef",
            "nightly",
            "active",
            "2.5"
        ],
        "{table}"
    );

    let empty = run(
        &["sandbox", "ls", "--format", "table"],
        ok(json!({"data": [], "next_cursor": null})),
    );
    assert!(empty.out.status.success(), "{}", empty.output_text());
    assert_eq!(empty.stdout().trim(), "(empty)");

    let raw = run(&["sandbox", "ls", "--format", "json"], ok(list.clone()));
    assert!(raw.out.status.success(), "{}", raw.output_text());
    let printed: Value = serde_json::from_str(&raw.stdout()).expect("JSON on stdout");
    assert_eq!(printed, list);
}
