//! `restart`: one replica of a sandbox restarted in place, keeping its
//! replica number while the other replicas keep running; then, unless
//! `--no-wait`, a wait for it to run again.

use std::time::Duration;

use fern_cli_sdk::formatter::{FormatError, OutputFormat, OutputPipeline};
use hedra_cli_sdk::api::SandboxStatus;
use tokio::time::Instant;

use super::api::{Answer, Api, CallError, ATTEMPT};
use super::error::{Class, Failure, SANDBOXES_NOT_AVAILABLE, SANDBOX_NOT_FOUND};
use super::lifecycle::{self, retry_wait, Timing, WaitFailed, ROUTE_BACKOFF};
use super::names::{self, Target};
use super::stream;

/// The request's and the wait's timings, as parameters so tests can shorten
/// them. The request's 429 waits come out of `wait.limit_wait`.
pub(crate) struct RestartTiming {
    /// Each attempt's timeout.
    pub(crate) attempt: Duration,
    /// The 429 budget, and the wait for the replica to run again.
    pub(crate) wait: Timing,
}

impl Default for RestartTiming {
    fn default() -> Self {
        Self {
            attempt: ATTEMPT,
            wait: Timing {
                ready_budget: lifecycle::start_budget(),
                ..Timing::default()
            },
        }
    }
}

/// What a restart came to: the sandbox to print, or the failure.
pub(crate) type Outcome = Result<Answer, Failure>;

/// Runs `restart` and prints its outcome: the sandbox on stdout through
/// `pipeline`, as the generated `get` and `stop` print it, or the failure
/// on stderr. Returns the exit code.
pub(crate) async fn run(
    api: &Api,
    target: &Target,
    replica: i64,
    wait: bool,
    json_errors: bool,
    pipeline: &OutputPipeline,
) -> i32 {
    stream::ignore_sigpipe();
    let outcome = restart(
        api,
        target,
        replica,
        wait,
        &RestartTiming::default(),
        &mut |line| stream::note(line),
    )
    .await;
    match outcome {
        Ok(answer) => print(answer.raw, pipeline, json_errors).await,
        Err(f) => {
            f.say(json_errors).await;
            f.class.code()
        }
    }
}

/// The sandbox on stdout as the generated sandbox commands print it: a
/// table de-hijacked (`views.rs`), every other format as the API answered,
/// and `--query` and `--quiet` applied.
async fn print(value: serde_json::Value, pipeline: &OutputPipeline, json_errors: bool) -> i32 {
    let value = match pipeline.format {
        OutputFormat::Table => super::super::views::dehijack(value),
        _ => value,
    };
    let mut out = std::io::stdout().lock();
    let written = pipeline
        .emit(&mut out, &value, false, true)
        .and_then(|()| std::io::Write::flush(&mut out).map_err(FormatError::Io));
    match written {
        Ok(()) => 0,
        Err(FormatError::Io(e)) if e.kind() == std::io::ErrorKind::BrokenPipe => {
            stream::die_by_sigpipe()
        }
        Err(e) => {
            let f = Failure::new(
                Class::OutputFailed,
                format!("Could not write the output: {e}."),
            );
            f.say(json_errors).await;
            f.class.code()
        }
    }
}

/// Reads the sandbox, refuses a replica it does not have and a sandbox that
/// stopped, asks the route to restart the replica, then waits for it to run
/// again unless `wait` is false. Each progress line goes to `say`.
pub(crate) async fn restart(
    api: &Api,
    target: &Target,
    replica: i64,
    wait: bool,
    timing: &RestartTiming,
    say: &mut dyn FnMut(&str),
) -> Outcome {
    let name = target.as_str();
    // The read resolves a name, and gives the replica's start, which the
    // wait compares a later read's against.
    let before = match target {
        Target::Name(name) => names::resolve_name(api, name).await?,
        Target::Id(id) => {
            api.read(id, ATTEMPT)
                .await
                .map_err(|e| e.failure(Some(id)))?
                .sandbox
        }
    };
    // The same refusal the stream verbs give, before any request that
    // could restart anything.
    if let Some(f) = stream::missing_replica(&before, name, replica) {
        return Err(f);
    }
    // A stopping, ended or failed sandbox has no instance left to restart.
    // A replica of a running sandbox that stopped is the route's to judge,
    // so a route that can restart one is not refused here.
    if matches!(
        before.status,
        SandboxStatus::Stopping | SandboxStatus::Ended | SandboxStatus::Failed
    ) {
        if let Some(f) = stream::not_running(&before, name, replica) {
            return Err(f);
        }
    }
    let started_at = stream::replica_at(&before, replica).and_then(|i| i.started_at);
    let started = Instant::now();
    let accepted = request(api, &before.id, name, replica, timing).await?;
    let subject = stream::subject(name, replica);
    if !wait {
        say(&format!(
            "{subject} is restarting. {}",
            stream::get_line(&before.id)
        ));
        return Ok(accepted);
    }
    say(&format!(
        "{subject} is restarting; waiting for it to run again."
    ));
    lifecycle::wait_restarted(
        api,
        &accepted.sandbox,
        name,
        replica,
        started_at,
        started,
        &timing.wait,
        say,
    )
    .await
    .map_err(|w: WaitFailed| {
        // A sandbox that stopped meanwhile, or whose replica stopped, gets
        // the line and next step a read before the restart would give. One
        // still starting, or never seen to restart, keeps the wait's line,
        // as does a wait whose read failed for good.
        w.last
            .and_then(|now| stream::not_running(&now, name, replica))
            .unwrap_or(w.failure)
    })
}

/// `POST /sandboxes/{id}/restart?replica=N`. A 429 is retried, for up to
/// the timing's `limit_wait` of waiting in all, each wait at least the
/// backoff's: it was not acted on. Nothing else is, since a restart is not
/// idempotent: a retry after an attempt that got no answer could restart
/// the replica a second time, as it comes back up.
async fn request(
    api: &Api,
    id: &str,
    name: &str,
    replica: i64,
    timing: &RestartTiming,
) -> Result<Answer, Failure> {
    let mut limited = Duration::ZERO;
    let mut backoff = ROUTE_BACKOFF;
    loop {
        let e = match api.restart(id, replica, timing.attempt).await {
            Ok(answer) => return Ok(answer),
            Err(e) => e,
        };
        if let CallError::Http(h) = &e {
            if h.status == 429 {
                // A `retry_after` of 0 would otherwise send the next
                // attempt at once, and count nothing against the budget.
                let wait = retry_wait(&e, backoff.wait()).max(backoff.wait());
                if limited + wait <= timing.wait.limit_wait {
                    limited += wait;
                    tokio::time::sleep(wait).await;
                    backoff.grow();
                    continue;
                }
            }
        }
        return Err(failure(&e, id, name, replica));
    }
}

/// What a restart request that failed means.
fn failure(e: &CallError, id: &str, name: &str, replica: i64) -> Failure {
    match e {
        // A 404 that is neither the sandbox's nor the feature's is the
        // route's own: an API without the restart route.
        CallError::Http(h)
            if h.status == 404
                && h.message() != SANDBOX_NOT_FOUND
                && h.message() != SANDBOXES_NOT_AVAILABLE =>
        {
            Failure::new(
                Class::Rejected,
                "This Hedra API cannot restart a replica yet.",
            )
            .with_trace(h.envelope.trace_id.clone())
        }
        // No answer, or one that may come after the restart began: the
        // replica may be restarting.
        CallError::Http(h) if h.status == 408 || h.status >= 500 => {
            unconfirmed(e, id, name, replica).with_trace(h.envelope.trace_id.clone())
        }
        CallError::Transport(_) | CallError::Timeout | CallError::Unreadable(_) => {
            unconfirmed(e, id, name, replica)
        }
        _ => e.failure(Some(name)),
    }
}

/// Unavailable, naming the cause and that the restart may have happened.
fn unconfirmed(e: &CallError, id: &str, name: &str, replica: i64) -> Failure {
    Failure::new(
        Class::Unavailable,
        format!(
            "Could not confirm the restart ({}): {} may be restarting.",
            lifecycle::describe(e),
            stream::subject(name, replica)
        ),
    )
    .then(stream::get_line(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    const ID: &str = "sandbox_3f2504e0-4f89-41d3-9a0c-0305e82c3301";

    fn sandbox(status: &str, instances: serde_json::Value) -> serde_json::Value {
        json!({
            "id": ID, "name": "box", "gpu": "h100:1", "gpu_model": "h100",
            "replicas": 2, "status": status, "created_at": "2026-11-03T12:00:00Z",
            "env_keys": [], "instances": instances,
            "billing": {"usd_per_hour": 2.5, "paid_usd": 0.0, "settled": false},
        })
    }

    fn two(zero: &str, one: &str) -> serde_json::Value {
        json!([
            {"replica": 0, "status": zero, "started_at": "2026-11-03T12:01:00Z"},
            {"replica": 1, "status": one, "started_at": "2026-11-03T12:01:00Z"},
        ])
    }

    fn short() -> RestartTiming {
        RestartTiming {
            attempt: Duration::from_millis(500),
            wait: Timing {
                limit_wait: Duration::from_millis(1_500),
                ready_budget: Duration::from_secs(2),
                fast_poll: Duration::from_millis(50),
                slow_poll: Duration::from_millis(50),
                elapsed_every: Duration::from_secs(60),
                ..Timing::default()
            },
        }
    }

    fn ok(body: serde_json::Value) -> ResponseTemplate {
        ResponseTemplate::new(200).set_body_json(body)
    }

    fn answer(status: u16, code: &str, retry_after: Option<u64>) -> ResponseTemplate {
        ResponseTemplate::new(status).set_body_json(json!({
            "error": {"code": code, "message": "m", "retry_after": retry_after},
            "trace_id": "t",
        }))
    }

    /// The sandbox read answers each of `reads` once, in order, then the
    /// last from then on; the restart route answers `restart`.
    async fn server(reads: Vec<serde_json::Value>, restart: ResponseTemplate) -> MockServer {
        let server = MockServer::start().await;
        let last = reads.last().cloned().unwrap();
        for read in reads {
            Mock::given(method("GET"))
                .and(path(format!("/sandboxes/{ID}")))
                .respond_with(ok(read))
                .up_to_n_times(1)
                .mount(&server)
                .await;
        }
        Mock::given(method("GET"))
            .and(path(format!("/sandboxes/{ID}")))
            .respond_with(ok(last))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(format!("/sandboxes/{ID}/restart")))
            .respond_with(restart)
            .mount(&server)
            .await;
        server
    }

    async fn posts(server: &MockServer) -> Vec<Request> {
        server
            .received_requests()
            .await
            .unwrap()
            .into_iter()
            .filter(|r| r.method.as_str() == "POST")
            .collect()
    }

    async fn go_with(
        server: &MockServer,
        replica: i64,
        wait: bool,
        timing: &RestartTiming,
    ) -> (Outcome, Vec<String>) {
        let api = Api::for_tests(&server.uri());
        let target = Target::parse(ID).unwrap();
        let mut lines = Vec::new();
        let outcome = restart(&api, &target, replica, wait, timing, &mut |l| {
            lines.push(l.to_string())
        })
        .await;
        (outcome, lines)
    }

    async fn go(server: &MockServer, replica: i64, wait: bool) -> (Outcome, Vec<String>) {
        go_with(server, replica, wait, &short()).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn replica_1_is_restarted_and_waited_for() {
        let server = server(
            vec![
                sandbox("active", two("running", "running")),
                sandbox("active", two("running", "restarting")),
                sandbox("active", two("running", "running")),
            ],
            ok(sandbox("active", two("running", "restarting"))),
        )
        .await;
        let (outcome, lines) = go(&server, 1, true).await;
        let s = outcome.unwrap().sandbox;
        assert_eq!(s.id, ID);
        let sent = posts(&server).await;
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0].url.query(), Some("replica=1"));
        assert_eq!(
            lines.first().map(String::as_str),
            Some(
                "Replica 1 of `sandbox_3f2504e0-4f89-41d3-9a0c-0305e82c3301` is restarting; \
                 waiting for it to run again."
            )
        );
        assert!(
            lines.iter().any(|l| l.contains("replica 1 restarting")),
            "{lines:?}"
        );
        assert!(
            lines.last().unwrap().contains("is running again, "),
            "{lines:?}"
        );
    }

    /// The route's answer showing the replica down counts: the first read
    /// after it may already show it running again, with the same start.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_routes_answer_counts_as_the_restart_beginning() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            ok(sandbox("active", two("running", "restarting"))),
        )
        .await;
        let started = Instant::now();
        let (outcome, lines) = go(&server, 1, true).await;
        outcome.unwrap();
        assert!(started.elapsed() < Duration::from_secs(1), "{lines:?}");
    }

    /// A read still showing the replica running from before the restart
    /// does not end the wait: only one after a read that showed it down.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_read_from_before_the_restart_does_not_end_the_wait() {
        let server = server(
            vec![
                sandbox("active", two("running", "running")),
                // The route answered while the replica still read running.
                sandbox("active", two("running", "running")),
                sandbox("active", two("starting", "running")),
                sandbox("active", two("running", "running")),
            ],
            ok(sandbox("active", two("running", "running"))),
        )
        .await;
        let (outcome, lines) = go(&server, 0, true).await;
        outcome.unwrap();
        assert!(
            lines.iter().any(|l| l.contains("replica 0 starting")),
            "{lines:?}"
        );
        assert!(lines.last().unwrap().starts_with("`sandbox_"), "{lines:?}");
    }

    /// Running throughout, as far as every read shows: the wait cannot tell
    /// whether the restart happened, and says so rather than "still
    /// starting".
    #[tokio::test(flavor = "multi_thread")]
    async fn a_restart_never_seen_says_so() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            ok(sandbox("active", two("running", "running"))),
        )
        .await;
        let f = go(&server, 1, true).await.0.unwrap_err();
        assert_eq!(f.class, Class::Unavailable);
        assert!(
            f.message.starts_with(&format!(
                "Replica 1 of `{ID}` reads running, but hedra-cli did not see it restart in "
            )),
            "{}",
            f.message
        );
        assert!(f.message.contains("sandbox get --sandbox"), "{}", f.message);
    }

    /// An answer without its instances shows nothing of the restart, so a
    /// read from before it does not end the wait.
    #[tokio::test(flavor = "multi_thread")]
    async fn an_answer_without_instances_does_not_open_the_wait() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            ok(sandbox("active", json!(null))),
        )
        .await;
        let f = go(&server, 1, true).await.0.unwrap_err();
        assert_eq!(f.class, Class::Unavailable, "{}", f.message);
    }

    /// A later start than the one before the restart counts as the
    /// restart, though no read caught the replica down.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_later_start_counts_as_the_restart() {
        let mut later = sandbox("active", two("running", "running"));
        later["instances"][0]["started_at"] = json!("2026-11-03T13:00:00Z");
        let server = server(
            vec![sandbox("active", two("running", "running")), later],
            ok(sandbox("active", two("running", "running"))),
        )
        .await;
        go(&server, 0, true).await.0.unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn no_wait_returns_the_routes_answer() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            ok(sandbox("active", two("restarting", "running"))),
        )
        .await;
        let (outcome, lines) = go(&server, 0, false).await;
        let answer = outcome.unwrap();
        assert_eq!(
            answer.sandbox.instances.unwrap()[0].status.to_string(),
            "restarting"
        );
        assert_eq!(answer.raw["instances"][0]["status"], "restarting");
        assert_eq!(lines.len(), 1, "{lines:?}");
        assert!(
            lines[0].ends_with(&format!(
                "is restarting. Check on it with `hedra-cli sandbox get --sandbox {ID}`."
            )),
            "{lines:?}"
        );
        // The read before the restart, and none after it.
        let gets = server.received_requests().await.unwrap();
        assert_eq!(gets.len(), 2, "{gets:?}");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn an_unknown_replica_is_refused_before_the_request() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            ok(sandbox("active", two("running", "running"))),
        )
        .await;
        let f = go(&server, 2, true).await.0.unwrap_err();
        assert_eq!(f.class, Class::Rejected);
        assert_eq!(
            f.message,
            format!("`{ID}` has no replica 2: it has 2 replicas, 0 to 1.")
        );
        assert!(posts(&server).await.is_empty());
    }

    /// Stopping, ended and failed sandboxes are refused before the request.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_stopped_sandbox_is_refused_before_the_request() {
        for status in ["stopping", "ended", "failed"] {
            let mut stopped = sandbox(status, two("stopped", "stopped"));
            if status != "failed" {
                stopped["end_reason"] = json!("caller_stopped");
            }
            let server = server(
                vec![stopped],
                ok(sandbox("active", two("running", "running"))),
            )
            .await;
            let f = go(&server, 1, true).await.0.unwrap_err();
            assert_eq!(f.class, Class::NotReady, "{status}");
            assert!(posts(&server).await.is_empty(), "{status}");
        }
    }

    /// The route's refusal of a replica that is not running is Not ready.
    #[tokio::test(flavor = "multi_thread")]
    async fn the_routes_not_running_refusal_is_not_ready() {
        let server = server(
            vec![sandbox("active", two("running", "stopped"))],
            answer(400, "FAILED_PRECONDITION", None),
        )
        .await;
        let f = go(&server, 1, true).await.0.unwrap_err();
        assert_eq!(f.class, Class::NotReady);
        assert_eq!(posts(&server).await.len(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_wait_past_its_budget_names_the_replica() {
        let server = server(
            vec![
                sandbox("active", two("running", "running")),
                sandbox("active", two("running", "restarting")),
            ],
            ok(sandbox("active", two("running", "restarting"))),
        )
        .await;
        let f = go(&server, 1, true).await.0.unwrap_err();
        assert_eq!(f.class, Class::NotReady);
        assert!(
            f.message
                .starts_with(&format!("Replica 1 of `{ID}` is still starting after ")),
            "{}",
            f.message
        );
    }

    /// A replica that stops during the wait gets the stream verbs' line.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_replica_that_stops_during_the_wait_says_so() {
        let mut stopped = sandbox("active", two("running", "stopped"));
        stopped["instances"][1]["failure_reason"] = json!("start_failed");
        let server = server(
            vec![sandbox("active", two("running", "running")), stopped],
            ok(sandbox("active", two("running", "restarting"))),
        )
        .await;
        let f = go(&server, 1, true).await.0.unwrap_err();
        assert_eq!(f.class, Class::NotReady);
        assert!(
            f.message.starts_with(&format!("Replica 1 of `{ID}`")),
            "{}",
            f.message
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_429_is_retried_then_answered() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ok(sandbox("active", two("running", "running"))))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(query_param("replica", "0"))
            .respond_with(answer(429, "RATE_LIMITED", Some(0)))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .respond_with(ok(sandbox("active", two("restarting", "running"))))
            .mount(&server)
            .await;
        go(&server, 0, false).await.0.unwrap();
        assert_eq!(posts(&server).await.len(), 2);
    }

    /// A `retry_after` of 0 still waits the backoff, so a route that keeps
    /// answering 429 ends in Limit within the budget, after a few attempts.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_429_with_no_wait_ends_in_limit() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            answer(429, "RATE_LIMITED", Some(0)),
        )
        .await;
        let timing = RestartTiming {
            wait: Timing {
                limit_wait: Duration::from_secs(4),
                ..short().wait
            },
            ..short()
        };
        // Bounded, so a loop that never counts its waits fails here rather
        // than hanging the suite.
        let f = tokio::time::timeout(Duration::from_secs(15), go_with(&server, 0, true, &timing))
            .await
            .expect("the 429 loop ends within its budget")
            .0
            .unwrap_err();
        assert_eq!(f.class, Class::Limit);
        // Waits of 1 s and 2 s fit the 4 s budget; 4 s more does not.
        assert_eq!(posts(&server).await.len(), 3);
    }

    /// Only a 429 is retried: a 5xx may come after the restart began.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_502_is_not_retried() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            answer(502, "BAD_GATEWAY", None),
        )
        .await;
        let timing = RestartTiming {
            wait: Timing {
                limit_wait: Duration::from_secs(60),
                ..short().wait
            },
            ..short()
        };
        let f = go_with(&server, 1, true, &timing).await.0.unwrap_err();
        assert_eq!(f.class, Class::Unavailable);
        assert!(
            f.message
                .starts_with("Could not confirm the restart (HTTP 502: m): "),
            "{}",
            f.message
        );
        assert!(f.message.contains("sandbox get --sandbox"), "{}", f.message);
        assert_eq!(posts(&server).await.len(), 1);
    }

    /// A request with no answer in time is not retried either.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_request_with_no_answer_is_not_retried() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            ok(sandbox("active", two("restarting", "running"))).set_delay(Duration::from_secs(2)),
        )
        .await;
        let timing = RestartTiming {
            wait: Timing {
                limit_wait: Duration::from_secs(60),
                ..short().wait
            },
            ..short()
        };
        let f = go_with(&server, 0, true, &timing).await.0.unwrap_err();
        assert_eq!(f.class, Class::Unavailable);
        assert!(
            f.message
                .starts_with("Could not confirm the restart (no answer in time): "),
            "{}",
            f.message
        );
        assert!(f.message.contains("sandbox get --sandbox"), "{}", f.message);
        assert_eq!(posts(&server).await.len(), 1);
    }

    /// v3 answers an unknown path with its envelope and "Not Found".
    #[tokio::test(flavor = "multi_thread")]
    async fn an_api_without_the_route_says_so() {
        let server = server(
            vec![sandbox("active", two("running", "running"))],
            ResponseTemplate::new(404).set_body_json(json!({
                "error": {"code": "NOT_FOUND", "message": "Not Found"},
            })),
        )
        .await;
        let f = go(&server, 0, true).await.0.unwrap_err();
        assert_eq!(f.class, Class::Rejected);
        assert_eq!(f.message, "This Hedra API cannot restart a replica yet.");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn the_routes_own_404s_keep_their_class() {
        for (message, class) in [
            (SANDBOX_NOT_FOUND, Class::NotFound),
            (SANDBOXES_NOT_AVAILABLE, Class::NotEnabled),
        ] {
            let server = server(
                vec![sandbox("active", two("running", "running"))],
                ResponseTemplate::new(404).set_body_json(json!({
                    "error": {"code": "NOT_FOUND", "message": message},
                })),
            )
            .await;
            let f = go(&server, 0, true).await.0.unwrap_err();
            assert_eq!(f.class, class, "{message}");
        }
    }
}
