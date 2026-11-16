//! `hedra-cli sandbox env` and `sandbox create --env`, driving the **real
//! compiled binary** against a mock v3 API.
//!
//! `ls`, `get` and `rm` are generated from the embedded spec; `set` is the
//! hand-written command in `cli/hedra-cli/sandbox/env.rs`. These pin the
//! request each sends (method, path, query, body), how an API error is
//! reported, and that a Secret's value is printed by nothing: not by `set`, in
//! any format, under `--dry-run` or `--debug`, nor by `get` or `ls` when an
//! answer carries one it should not.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry).

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use serde_json::{json, Value};
use tempfile::TempDir;
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SECRET: &str = "sk-live-0123456789abcdef";
const COLLECTION: &str = "/sandboxes/environment-variables";

struct Outcome {
    out: Output,
    requests: Vec<Request>,
}

impl Outcome {
    fn stdout(&self) -> String {
        String::from_utf8_lossy(&self.out.stdout).into_owned()
    }

    fn all_output(&self) -> String {
        format!(
            "{}{}",
            String::from_utf8_lossy(&self.out.stdout),
            String::from_utf8_lossy(&self.out.stderr)
        )
    }

    fn text(&self) -> String {
        describe(&self.out)
    }

    fn code(&self) -> Option<i32> {
        self.out.status.code()
    }

    fn request(&self) -> &Request {
        assert_eq!(self.requests.len(), 1, "{}", self.text());
        &self.requests[0]
    }

    fn body(&self) -> Value {
        serde_json::from_slice(&self.request().body).expect("request body is JSON")
    }

    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout()).unwrap_or_else(|e| panic!("{e}\n{}", self.text()))
    }

    fn assert_no_secret(&self) {
        assert!(
            !self.all_output().contains(SECRET),
            "the secret value was printed\n{}",
            self.text()
        );
    }
}

fn describe(out: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

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
/// environment it would otherwise inherit that could change what a test sees.
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
        .env_remove("APPDATA");
    cmd
}

/// Run `hedra-cli <args> --base-url <mock>` with `stdin` piped in (nothing
/// piped: an empty stdin) and a key in the environment. `responses` answer
/// the requests in order; the last answers every request after it.
fn run_with_stdin(args: &[&str], stdin: &[u8], responses: Vec<ResponseTemplate>) -> Outcome {
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
        let mut child = hedra_cli(home.path())
            .args(args)
            .args(["--base-url", &uri])
            .env("HEDRA_API_KEY", "test-key")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("run hedra-cli");
        {
            let mut pipe = child.stdin.take().expect("stdin");
            // A refusal can exit before reading; a closed pipe is fine then.
            let _ = pipe.write_all(stdin);
        }
        let out = child.wait_with_output().expect("wait for hedra-cli");

        let requests = server
            .received_requests()
            .await
            .expect("mock server recorded requests");
        Outcome { out, requests }
    })
}

fn run(args: &[&str], responses: Vec<ResponseTemplate>) -> Outcome {
    run_with_stdin(args, b"", responses)
}

fn answer(status: u16, body: Value) -> Vec<ResponseTemplate> {
    vec![ResponseTemplate::new(status).set_body_json(body)]
}

fn error(status: u16, code: &str, message: &str) -> Vec<ResponseTemplate> {
    answer(
        status,
        json!({"error": {"code": code, "message": message, "retryable": false}}),
    )
}

/// A variable's metadata, as list items and the set and delete answers carry it.
fn meta(name: &str, kind: &str) -> Value {
    json!({
        "name": name,
        "type": kind,
        "created_at": "2026-11-16T00:00:00Z",
        "updated_at": "2026-11-16T00:00:00Z"
    })
}

fn path_of(request: &Request) -> &str {
    request.url.path()
}

// ---------------------------------------------------------------------------
// The command tree
// ---------------------------------------------------------------------------

fn offline(args: &[&str]) -> Output {
    let home = TempDir::new().expect("temp HOME");
    hedra_cli(home.path())
        .args(args)
        .stdin(Stdio::null())
        .output()
        .expect("run hedra-cli")
}

#[test]
fn env_has_ls_get_set_and_rm() {
    let out = offline(&["sandbox", "env", "--help"]);
    assert!(out.status.success(), "{}", describe(&out));
    let help = String::from_utf8_lossy(&out.stdout);
    for verb in ["ls", "get", "set", "rm"] {
        assert!(
            help.lines().any(|l| l.trim_start().starts_with(verb)),
            "`sandbox env {verb}` missing\n{help}"
        );
    }
}

#[test]
fn set_takes_its_value_from_stdin_or_a_file_and_shows_how() {
    let out = offline(&["sandbox", "env", "set", "--help"]);
    assert!(out.status.success(), "{}", describe(&out));
    let help = String::from_utf8_lossy(&out.stdout);
    assert!(help.contains("--from-file <PATH>"), "{help}");
    assert!(help.contains("--type <TYPE>"), "{help}");
    assert!(
        !help.contains("--value"),
        "the generated --value flag is still offered\n{help}"
    );
    // The examples run the syntax this command accepts.
    assert!(
        help.contains("| hedra-cli sandbox env set --name OPENAI_KEY --type secret"),
        "{help}"
    );
    assert!(help.contains("{\"ref\": \"OPENAI_KEY\"}"), "{help}");
}

// ---------------------------------------------------------------------------
// ls, get, rm (generated)
// ---------------------------------------------------------------------------

#[test]
fn ls_lists_metadata_and_filters_by_type() {
    let listing = json!({
        "data": [meta("MODEL_URL", "config"), meta("OPENAI_KEY", "secret")],
        "next_cursor": null
    });
    let o = run(
        &["sandbox", "env", "ls", "--format", "json"],
        answer(200, listing.clone()),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(o.request().method.as_str(), "GET");
    assert_eq!(path_of(o.request()), COLLECTION);
    assert_eq!(o.request().url.query(), None);
    assert!(
        o.stdout().contains("MODEL_URL") && o.stdout().contains("OPENAI_KEY"),
        "{}",
        o.text()
    );

    let o = run(
        &[
            "sandbox", "env", "ls", "--type", "secret", "--format", "json",
        ],
        answer(
            200,
            json!({"data": [meta("OPENAI_KEY", "secret")], "next_cursor": null}),
        ),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(o.request().url.query(), Some("type=secret"));
}

#[test]
fn ls_never_prints_a_secret_value_an_answer_carries() {
    let mut leaky = meta("OPENAI_KEY", "secret");
    leaky["value"] = json!(SECRET);
    let listing = json!({"data": [meta("MODEL_URL", "config"), leaky], "next_cursor": null});
    for format in ["json", "table", "yaml", "csv", "jsonl"] {
        let o = run(
            &["sandbox", "env", "ls", "--format", format],
            answer(200, listing.clone()),
        );
        assert_eq!(o.code(), Some(0), "{format}: {}", o.text());
        o.assert_no_secret();
    }
}

#[test]
fn get_prints_a_config_value() {
    let mut config = meta("MODEL_URL", "config");
    config["value"] = json!("https://models.example/v1");
    let o = run(
        &[
            "sandbox",
            "env",
            "get",
            "--name",
            "MODEL_URL",
            "--format",
            "json",
        ],
        answer(200, config),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(o.request().method.as_str(), "GET");
    assert_eq!(path_of(o.request()), format!("{COLLECTION}/MODEL_URL"));
    assert_eq!(o.json()["value"], "https://models.example/v1");

    // `--query value` is the scriptable read of one value.
    let mut config = meta("MODEL_URL", "config");
    config["value"] = json!("https://models.example/v1");
    let o = run(
        &[
            "sandbox",
            "env",
            "get",
            "--name",
            "MODEL_URL",
            "--format",
            "json",
            "--query",
            "value",
        ],
        answer(200, config),
    );
    assert_eq!(o.json(), json!("https://models.example/v1"), "{}", o.text());
}

#[test]
fn get_prints_a_secret_as_metadata_only_in_every_format() {
    // diffusion never sends a Secret's value; this answer does, and the CLI
    // still prints none of it.
    let mut leaky = meta("OPENAI_KEY", "secret");
    leaky["value"] = json!(SECRET);
    for format in ["json", "table", "yaml", "csv", "jsonl"] {
        let o = run(
            &[
                "sandbox",
                "env",
                "get",
                "--name",
                "OPENAI_KEY",
                "--format",
                format,
            ],
            answer(200, leaky.clone()),
        );
        assert_eq!(o.code(), Some(0), "{format}: {}", o.text());
        assert!(o.stdout().contains("OPENAI_KEY"), "{format}: {}", o.text());
        o.assert_no_secret();
    }
}

#[test]
fn get_reports_a_missing_variable() {
    let o = run(
        &[
            "sandbox", "env", "get", "--name", "NOPE", "--format", "json",
        ],
        error(
            404,
            "NOT_FOUND",
            "Your workspace holds no variable named NOPE.",
        ),
    );
    assert_eq!(o.code(), Some(1), "{}", o.text());
    assert!(
        o.all_output().contains("holds no variable named NOPE"),
        "{}",
        o.text()
    );
}

#[test]
fn rm_deletes_by_name() {
    let o = run(
        &[
            "sandbox",
            "env",
            "rm",
            "--name",
            "OPENAI_KEY",
            "--format",
            "json",
        ],
        answer(200, meta("OPENAI_KEY", "secret")),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(o.request().method.as_str(), "DELETE");
    assert_eq!(path_of(o.request()), format!("{COLLECTION}/OPENAI_KEY"));
    assert!(o.request().body.is_empty());
}

// ---------------------------------------------------------------------------
// set (hand-written)
// ---------------------------------------------------------------------------

fn set(extra: &[&str], stdin: &[u8], responses: Vec<ResponseTemplate>) -> Outcome {
    let mut args = vec!["sandbox", "env", "set"];
    args.extend_from_slice(extra);
    run_with_stdin(&args, stdin, responses)
}

#[test]
fn set_posts_a_piped_secret_and_prints_only_its_metadata() {
    for (format, ending) in [("json", "\n"), ("table", "\r\n"), ("yaml", "")] {
        let stdin = format!("{SECRET}{ending}");
        let o = set(
            &[
                "--name",
                "OPENAI_KEY",
                "--type",
                "secret",
                "--format",
                format,
            ],
            stdin.as_bytes(),
            answer(201, meta("OPENAI_KEY", "secret")),
        );
        assert_eq!(o.code(), Some(0), "{format}: {}", o.text());
        let request = o.request();
        assert_eq!(request.method.as_str(), "POST");
        assert_eq!(path_of(request), COLLECTION);
        assert_eq!(
            request
                .headers
                .get("content-type")
                .map(|v| v.to_str().unwrap()),
            Some("application/json")
        );
        assert!(
            request.headers.get("authorization").is_some(),
            "no credential sent"
        );
        // One trailing line ending dropped, nothing else.
        assert_eq!(
            o.body(),
            json!({"name": "OPENAI_KEY", "type": "secret", "value": SECRET}),
            "{format}"
        );
        assert!(o.stdout().contains("OPENAI_KEY"), "{format}: {}", o.text());
        o.assert_no_secret();
    }
}

#[test]
fn set_reads_a_config_from_a_file_byte_for_byte() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("model_url.txt");
    std::fs::write(&path, "https://models.example/v1\n").unwrap();
    let o = set(
        &[
            "--name",
            "MODEL_URL",
            "--type",
            "config",
            "--from-file",
            path.to_str().unwrap(),
            "--format",
            "json",
        ],
        b"ignored on stdin",
        answer(200, meta("MODEL_URL", "config")),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(
        o.body(),
        json!({"name": "MODEL_URL", "type": "config", "value": "https://models.example/v1\n"})
    );
    assert_eq!(o.json()["name"], "MODEL_URL");
}

#[test]
fn set_saves_an_empty_config_only_from_a_file() {
    let o = set(
        &[
            "--name",
            "EXTRA_ARGS",
            "--type",
            "config",
            "--from-file",
            "/dev/null",
            "--format",
            "json",
        ],
        b"",
        answer(201, meta("EXTRA_ARGS", "config")),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(o.body()["value"], "");

    let o = set(
        &[
            "--name",
            "EXTRA_ARGS",
            "--type",
            "config",
            "--format",
            "json",
        ],
        b"",
        answer(201, meta("EXTRA_ARGS", "config")),
    );
    assert_eq!(o.code(), Some(3), "{}", o.text());
    assert!(o.requests.is_empty(), "an empty stdin was sent");
    assert!(
        o.all_output().contains("--from-file /dev/null"),
        "{}",
        o.text()
    );
}

#[test]
fn set_refuses_local_mistakes_before_sending_and_never_quotes_the_value() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join(SECRET);
    let missing = missing.to_str().unwrap();
    let oversized = "a".repeat(16 * 1024 + 1);
    let value_flag = format!("--value={SECRET}");
    let cases: Vec<(&str, Vec<&str>, Vec<u8>)> = vec![
        (
            "value on the command line",
            vec!["--name", "K", "--type", "secret", "--value", SECRET],
            vec![],
        ),
        (
            "value as --value=",
            vec!["--name", "K", "--type", "secret", &value_flag],
            vec![],
        ),
        (
            "value as a positional",
            vec!["--name", "K", "--type", "secret", SECRET],
            vec![],
        ),
        (
            "value as the name",
            vec!["--name", "sk-live-0123456789abcdef", "--type", "secret"],
            b"x".to_vec(),
        ),
        (
            "empty secret",
            vec!["--name", "K", "--type", "secret"],
            vec![],
        ),
        (
            "oversized secret",
            vec!["--name", "K", "--type", "secret"],
            oversized.into_bytes(),
        ),
        (
            "NUL",
            vec!["--name", "K", "--type", "secret"],
            format!("{SECRET}\0").into_bytes(),
        ),
        (
            "not UTF-8",
            vec!["--name", "K", "--type", "secret"],
            [SECRET.as_bytes(), b"\xff"].concat(),
        ),
        (
            "missing file",
            vec!["--name", "K", "--type", "secret", "--from-file", missing],
            vec![],
        ),
    ];
    for (case, args, stdin) in cases {
        let mut args = args;
        args.extend_from_slice(&["--format", "json"]);
        let o = set(&args, &stdin, answer(201, meta("K", "secret")));
        assert_eq!(o.code(), Some(3), "{case}: {}", o.text());
        assert!(o.requests.is_empty(), "{case}: a request was sent");
        o.assert_no_secret();
    }
}

#[test]
fn set_requires_an_explicit_type() {
    for args in [
        vec!["--name", "K"],
        vec!["--name", "K", "--type", "plaintext"],
    ] {
        let o = set(&args, SECRET.as_bytes(), answer(201, meta("K", "secret")));
        assert_ne!(o.code(), Some(0), "{args:?}: {}", o.text());
        assert!(o.requests.is_empty(), "{args:?}: a request was sent");
        o.assert_no_secret();
    }
}

#[test]
fn set_dry_run_sends_nothing_and_redacts_the_value() {
    let o = set(
        &[
            "--name",
            "OPENAI_KEY",
            "--type",
            "secret",
            "--dry-run",
            "--format",
            "json",
        ],
        SECRET.as_bytes(),
        answer(201, meta("OPENAI_KEY", "secret")),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert!(o.requests.is_empty(), "--dry-run sent a request");
    let preview = o.json();
    assert_eq!(preview["method"], "POST");
    assert!(
        preview["url"].as_str().unwrap().ends_with(COLLECTION),
        "{preview}"
    );
    assert_eq!(preview["body"]["name"], "OPENAI_KEY");
    assert_eq!(preview["body"]["type"], "secret");
    o.assert_no_secret();
}

#[test]
fn set_debug_does_not_dump_the_value() {
    let o = set(
        &[
            "--name",
            "OPENAI_KEY",
            "--type",
            "secret",
            "--debug",
            "--format",
            "json",
        ],
        SECRET.as_bytes(),
        answer(201, meta("OPENAI_KEY", "secret")),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(o.body()["value"], SECRET);
    o.assert_no_secret();
}

#[test]
fn set_reports_api_refusals_without_the_value() {
    let cases = [
        (409, "ALREADY_EXISTS", "Your workspace holds OPENAI_KEY as a config variable; delete it to save it as a secret."),
        (400, "INVALID_REQUEST", "name: must match ^[A-Z_][A-Z0-9_]{0,62}$"),
        (403, "FORBIDDEN", "This key lacks the sandboxes:write scope."),
        (404, "NOT_FOUND", "Your workspace has no sandbox account."),
    ];
    for (status, code, message) in cases {
        let o = set(
            &[
                "--name",
                "OPENAI_KEY",
                "--type",
                "secret",
                "--format",
                "json",
            ],
            SECRET.as_bytes(),
            error(status, code, message),
        );
        assert_eq!(o.code(), Some(1), "{status}: {}", o.text());
        assert_eq!(o.requests.len(), 1, "{status}: a refusal was retried");
        assert!(o.all_output().contains(message), "{status}: {}", o.text());
        o.assert_no_secret();
    }
}

#[test]
fn set_does_not_retry_a_server_error() {
    let o = set(
        &[
            "--name",
            "OPENAI_KEY",
            "--type",
            "secret",
            "--format",
            "json",
        ],
        SECRET.as_bytes(),
        error(
            503,
            "SERVICE_UNAVAILABLE",
            "Sandboxes are unavailable; retry.",
        ),
    );
    assert_eq!(o.code(), Some(1), "{}", o.text());
    // A set carries no idempotency key, so the runtime does not replay it.
    assert_eq!(o.requests.len(), 1, "{}", o.text());
    assert!(o.all_output().contains("retry"), "{}", o.text());
    o.assert_no_secret();
}

// ---------------------------------------------------------------------------
// sandbox create --env
// ---------------------------------------------------------------------------

fn create(env: &str) -> Outcome {
    let created = json!({
        "id": "sandbox_0123456789abcdef",
        "name": "trainer",
        "status": "creating",
        "gpu": "h100:1",
        "replicas": 1,
        "env_keys": ["LOG_LEVEL", "MODEL_URL", "OPENAI_API_KEY"],
        "instances": [],
        "created_at": "2026-11-16T00:00:00Z"
    });
    run(
        &[
            "sandbox",
            "create",
            "--name",
            "trainer",
            "--image",
            "ghcr.io/acme/trainer:1",
            "--gpu",
            "h100:1",
            "--ttl-seconds",
            "3600",
            "--env",
            env,
            "--format",
            "json",
        ],
        answer(201, created),
    )
}

#[test]
fn create_sends_literals_references_and_aliased_references_as_written() {
    let env = json!({
        "LOG_LEVEL": "info",
        "MODEL_URL": {"ref": "MODEL_URL"},
        "OPENAI_API_KEY": {"ref": "OPENAI_KEY"}
    });
    let o = create(&env.to_string());
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(o.request().method.as_str(), "POST");
    assert_eq!(path_of(o.request()), "/sandboxes");
    assert_eq!(o.body()["env"], env);
}

#[test]
fn create_refuses_a_malformed_env_value_without_quoting_it() {
    for bad in [
        json!({"OPENAI_API_KEY": {"ref": "OPENAI_KEY", "value": SECRET}}),
        json!({"OPENAI_API_KEY": {"value": SECRET}}),
        json!({"OPENAI_API_KEY": {"ref": 5}}),
        json!({"PORT": 8080}),
        json!({"OPENAI_API_KEY": [SECRET]}),
    ] {
        let o = create(&bad.to_string());
        assert_eq!(o.code(), Some(3), "{bad}: {}", o.text());
        assert!(o.requests.is_empty(), "{bad}: a request was sent");
        assert!(
            o.all_output().contains("naming a saved variable"),
            "{}",
            o.text()
        );
        o.assert_no_secret();
    }
}

#[test]
fn create_reports_an_unknown_reference() {
    let o = run(
        &[
            "sandbox",
            "create",
            "--name",
            "trainer",
            "--image",
            "ghcr.io/acme/trainer:1",
            "--gpu",
            "h100:1",
            "--ttl-seconds",
            "3600",
            "--env",
            r#"{"OPENAI_API_KEY": {"ref": "NOPE"}}"#,
            "--format",
            "json",
        ],
        answer(
            400,
            json!({"error": {
                "code": "INVALID_REQUEST",
                "message": "env.OPENAI_API_KEY: the sandbox could not use this environment variable reference.",
                "param": "env.OPENAI_API_KEY",
                "retryable": false
            }}),
        ),
    );
    assert_eq!(o.code(), Some(1), "{}", o.text());
    assert!(
        o.all_output().contains("env.OPENAI_API_KEY"),
        "{}",
        o.text()
    );
}

// ---------------------------------------------------------------------------
// The legacy secrets commands are unchanged
// ---------------------------------------------------------------------------

#[test]
fn secrets_set_still_posts_to_the_legacy_route() {
    let o = run(
        &[
            "sandbox",
            "secrets",
            "set",
            "--name",
            "OPENAI_KEY",
            "--value",
            "v",
            "--format",
            "json",
        ],
        answer(
            201,
            json!({"name": "OPENAI_KEY", "created_at": "2026-11-16T00:00:00Z"}),
        ),
    );
    assert_eq!(o.code(), Some(0), "{}", o.text());
    assert_eq!(path_of(o.request()), "/sandboxes/secrets");
    assert_eq!(o.body(), json!({"name": "OPENAI_KEY", "value": "v"}));
}
