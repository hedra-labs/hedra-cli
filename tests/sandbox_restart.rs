//! `sandbox restart`, run as the built binary against a mock v3 API.
//!
//! The embedded spec predates `POST /sandboxes/{sandbox}/restart`, so the
//! mock stands in for the route as diffusion#9253 shapes it: a `replica`
//! query parameter, and the sandbox in answer, as `stop` answers, with the
//! replica reading `status: running` and `restarting: true` until the new
//! container runs.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry).

mod sandbox_stream_support;

use std::process::Output;

use sandbox_stream_support::*;
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn restart_path() -> String {
    format!("/v3/sandboxes/{ID}/restart")
}

/// Two instances. "restarting" is how v3 answers a replica being replaced
/// in place: `status: running` with `restarting: true`.
fn two(zero: &str, one: &str) -> Value {
    let instance = |replica: i64, status: &str| {
        let restarting = status == "restarting";
        json!({
            "replica": replica,
            "status": if restarting { "running" } else { status },
            "restarting": restarting,
            "started_at": "2026-11-03T12:01:00Z",
        })
    };
    json!([instance(0, zero), instance(1, one)])
}

fn box_of(status: &str, instances: Value) -> Value {
    replicated(ID, "box", status, 2, instances)
}

/// A sandbox `ID`, also named `box`, whose reads answer `reads` in turn,
/// repeating the last, and whose restart route answers `restart`.
async fn server(reads: Vec<Value>, restart: ResponseTemplate) -> MockServer {
    let server = MockServer::start().await;
    let answers: Vec<ResponseTemplate> = reads
        .into_iter()
        .map(|r| ResponseTemplate::new(200).set_body_json(r))
        .collect();
    let sequence = std::sync::Arc::new(Sequence::new(answers));
    for name in [ID, "box"] {
        let sequence = sequence.clone();
        Mock::given(method("GET"))
            .and(path(format!("/v3/sandboxes/{name}")))
            .respond_with(move |r: &wiremock::Request| wiremock::Respond::respond(&*sequence, r))
            .mount(&server)
            .await;
    }
    Mock::given(method("POST"))
        .and(path(restart_path()))
        .respond_with(restart)
        .mount(&server)
        .await;
    server
}

fn ok(body: Value) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(body)
}

/// `hedra-cli <args>` with the wait's budget cut to `budget_ms`, so a wait
/// that never ends fails the test in seconds, not after 15 minutes. The
/// wait polls every 2 s, so a test that needs a second read gives it 5 s.
fn run_with_budget(run: &Run, args: &[&str], budget_ms: u64) -> Output {
    let mut cmd = run.command(args);
    cmd.env("HEDRA_CLI_TEST_START_BUDGET_MS", budget_ms.to_string());
    cmd.output().unwrap()
}

fn restart(run: &Run, args: &[&str]) -> Output {
    run_with_budget(run, args, 5_000)
}

async fn restarts(server: &MockServer) -> Vec<Option<String>> {
    requests_to(server, "POST", &restart_path())
        .await
        .iter()
        .map(|r| query(r, "replica"))
        .collect()
}

#[tokio::test]
async fn replica_0_is_restarted_and_waited_for() {
    let server = server(
        vec![box_of("active", two("running", "running"))],
        ok(box_of("active", two("restarting", "running"))),
    )
    .await;
    let run = Run::new(&server);
    let out = restart(&run, &["sandbox", "restart", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(restarts(&server).await, [Some("0".into())]);
    assert!(
        err.contains(&format!(
            "`{ID}` is restarting; waiting for it to run again."
        )),
        "{err}"
    );
    assert!(err.contains(&format!("`{ID}` is running again, ")), "{err}");
    // Piped, the sandbox prints as JSON, as the generated `stop` prints it.
    let printed: Value = serde_json::from_str(&stdout(&out)).expect("JSON on stdout");
    assert_eq!(printed["id"], ID);
    // Every request carries the spec version the generated calls send.
    for r in server.received_requests().await.unwrap() {
        assert!(
            r.headers.get("x-hedra-spec-version").is_some(),
            "{} {} has no X-Hedra-Spec-Version",
            r.method,
            r.url
        );
    }
}

#[tokio::test]
async fn replica_1_of_a_named_sandbox_is_restarted_and_waited_for() {
    let server = server(
        vec![
            box_of("active", two("running", "running")),
            box_of("active", two("running", "restarting")),
            box_of("active", two("running", "running")),
        ],
        ok(box_of("active", two("running", "restarting"))),
    )
    .await;
    let run = Run::new(&server);
    let out = restart(&run, &["sandbox", "restart", "box", "--replica", "1"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    // The name is resolved once, and the restart asks for the id.
    assert_eq!(restarts(&server).await, [Some("1".into())]);
    assert!(
        err.contains("box is active (replica 0 running, replica 1 restarting)."),
        "{err}"
    );
    assert!(
        err.contains("Replica 1 of `box` is running again, "),
        "{err}"
    );
}

#[tokio::test]
async fn no_wait_prints_the_routes_answer_without_reading_again() {
    let server = server(
        vec![box_of("active", two("running", "running"))],
        ok(box_of("active", two("running", "restarting"))),
    )
    .await;
    let run = Run::new(&server);
    let out = restart(
        &run,
        &["sandbox", "restart", ID, "--replica", "1", "--no-wait"],
    );
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert!(
        err.contains(&format!(
            "Replica 1 of `{ID}` is restarting. Check on it with `hedra-cli sandbox get \
             --sandbox {ID}`."
        )),
        "{err}"
    );
    let printed: Value = serde_json::from_str(&stdout(&out)).expect("JSON on stdout");
    assert_eq!(printed["instances"][1]["restarting"], true);
    assert_eq!(
        requests_to(&server, "GET", &format!("/v3/sandboxes/{ID}"))
            .await
            .len(),
        1
    );
}

#[tokio::test]
async fn an_unknown_replica_is_refused_before_the_request() {
    let server = server(
        vec![box_of("active", two("running", "running"))],
        ok(box_of("active", two("running", "running"))),
    )
    .await;
    let run = Run::new(&server);
    let out = restart(&run, &["sandbox", "restart", ID, "--replica", "2"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(19), "{err}");
    assert_eq!(
        class_line(&err),
        Some(
            format!(
                "hedra-cli: Rejected request (19): `{ID}` has no replica 2: it has 2 replicas, \
                 0 to 1."
            )
            .as_str()
        ),
        "{err}"
    );
    assert!(restarts(&server).await.is_empty());
}

#[tokio::test]
async fn a_stopped_sandbox_is_refused_before_the_request() {
    let mut ended = box_of("ended", two("stopped", "stopped"));
    ended["end_reason"] = json!("caller_stopped");
    let server = server(vec![ended], ok(box_of("active", two("running", "running")))).await;
    let run = Run::new(&server);
    let out = restart(&run, &["sandbox", "restart", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(13), "{err}");
    assert!(
        class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Not ready or stopped (13): ")),
        "{err}"
    );
    assert!(restarts(&server).await.is_empty());
}

/// The route refuses a replica that is not running (ENG-13053) with 400
/// FAILED_PRECONDITION: Not ready or stopped.
#[tokio::test]
async fn the_routes_not_running_refusal_exits_13() {
    let server = server(
        vec![box_of("active", two("running", "stopped"))],
        envelope(
            400,
            "FAILED_PRECONDITION",
            "Replica 1 is not running.",
            None,
        ),
    )
    .await;
    let run = Run::new(&server);
    let out = restart(&run, &["sandbox", "restart", ID, "--replica", "1"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(13), "{err}");
    assert!(err.contains("Replica 1 is not running."), "{err}");
    assert_eq!(restarts(&server).await.len(), 1);
}

#[tokio::test]
async fn a_wait_that_times_out_names_the_replica_and_exits_13() {
    let server = server(
        vec![
            box_of("active", two("running", "running")),
            box_of("active", two("running", "restarting")),
        ],
        ok(box_of("active", two("running", "restarting"))),
    )
    .await;
    let run = Run::new(&server);
    let out = run_with_budget(&run, &["sandbox", "restart", ID, "--replica", "1"], 300);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(13), "{err}");
    assert!(
        class_line(&err).is_some_and(|l| l.contains(&format!(
            ": Replica 1 of `{ID}` is still starting after "
        ))),
        "{err}"
    );
    assert!(stdout(&out).is_empty());
}

/// A replica that reads running throughout may or may not have restarted:
/// exit 17, naming `get`, not "still starting".
#[tokio::test]
async fn a_restart_never_seen_exits_17() {
    let server = server(
        vec![box_of("active", two("running", "running"))],
        ok(box_of("active", two("running", "running"))),
    )
    .await;
    let run = Run::new(&server);
    let out = run_with_budget(&run, &["sandbox", "restart", ID, "--replica", "1"], 300);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(17), "{err}");
    assert!(
        class_line(&err).is_some_and(|l| l.contains(&format!(
            ": Replica 1 of `{ID}` reads running, but hedra-cli did not see it restart in "
        ))),
        "{err}"
    );
}

/// As a table, the sandbox prints as the generated `get` prints the same
/// sandbox: de-hijacked, every field kept.
#[tokio::test]
async fn the_table_is_the_generated_gets() {
    let body = box_of("active", two("running", "restarting"));
    let server = server(vec![body.clone()], ok(body)).await;
    let run = Run::new(&server);
    let out = restart(&run, &["sandbox", "restart", ID, "--no-wait", "--human"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let get = run.run(&["sandbox", "get", "--sandbox", ID, "--human"]);
    assert_eq!(get.status.code(), Some(0), "{}", stderr(&get));
    let table = stdout(&out);
    assert!(table.contains("restarting"), "{table}");
    assert_eq!(table.trim_end(), stdout(&get).trim_end());
}

/// `--query` and `--quiet` shape stdout as they do the generated `get`'s.
#[tokio::test]
async fn query_and_quiet_shape_stdout_as_gets() {
    let body = box_of("active", two("running", "restarting"));
    let server = server(vec![body.clone()], ok(body)).await;
    let run = Run::new(&server);
    for flags in [&["--query", "id"][..], &["--quiet"][..]] {
        let mut args = vec!["sandbox", "restart", ID, "--no-wait"];
        args.extend_from_slice(flags);
        let out = restart(&run, &args);
        assert_eq!(out.status.code(), Some(0), "{flags:?}: {}", stderr(&out));
        let mut get_args = vec!["sandbox", "get", "--sandbox", ID];
        get_args.extend_from_slice(flags);
        let get = run.run(&get_args);
        assert_eq!(stdout(&out), stdout(&get), "{flags:?}");
    }
    assert_eq!(restarts(&server).await.len(), 2);
}

/// A query that does not compile is refused before any request.
#[tokio::test]
async fn a_bad_query_is_refused_before_any_request() {
    let server = server(
        vec![box_of("active", two("running", "running"))],
        ok(box_of("active", two("running", "running"))),
    )
    .await;
    let run = Run::new(&server);
    let out = restart(&run, &["sandbox", "restart", ID, "--query", "[["]);
    assert_eq!(out.status.code(), Some(19), "{}", stderr(&out));
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn dry_run_sends_nothing() {
    let server = server(
        vec![box_of("active", two("running", "running"))],
        ok(box_of("active", two("running", "running"))),
    )
    .await;
    let run = Run::new(&server);
    let out = restart(&run, &["sandbox", "restart", ID, "--dry-run"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(server.received_requests().await.unwrap().is_empty());
}
