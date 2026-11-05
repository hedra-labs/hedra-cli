//! Creating a sandbox and waiting for it, and stopping one and waiting for
//! it to end: the loops whose retries the runtime's own cannot make safely.

use std::time::Duration;

use hedra_cli_sdk::api::{
    CreateSandboxRequest, Sandbox, SandboxFailureReason, SandboxInstanceStatus, SandboxStatus,
};
use hedra_cli_sdk::RequestOptions;
use tokio::time::Instant;

use super::api::{attempt, Api, CallError, ATTEMPT};
use super::error::{escape_controls, Class, Failure};
use super::render::{human_duration, stop_reason};

/// How long `create` keeps retrying one create, under one key.
pub(crate) const CREATE_LOOP: Duration = Duration::from_secs(5 * 60);
/// The create loop's five minutes plus a full pool's fifteen.
pub(crate) const READY_BUDGET: Duration = Duration::from_secs(20 * 60);
pub(crate) const CREATE_ATTEMPT: Duration = Duration::from_secs(45);
const STOP_DEADLINE: Duration = Duration::from_secs(5 * 60);
/// The longest wait taken from an envelope's `retry_after`.
const MAX_RETRY_AFTER: Duration = Duration::from_secs(30);

/// The wait before retrying after `error`: the envelope's `retry_after`,
/// capped, or `backoff` when the answer carries none.
pub(crate) fn retry_wait(error: &CallError, backoff: Duration) -> Duration {
    error
        .http()
        .and_then(|e| e.envelope.retry_after)
        .map(|s| Duration::from_secs(s).min(MAX_RETRY_AFTER))
        .unwrap_or(backoff)
}

/// `limit`, cut to what is left before `deadline`.
fn within(limit: Duration, deadline: Instant) -> Duration {
    limit.min(deadline.saturating_duration_since(Instant::now()))
}

fn next_backoff(backoff: Duration) -> Duration {
    (backoff * 2).min(Duration::from_secs(16))
}

fn describe(error: &CallError) -> String {
    match error {
        CallError::Http(e) if !e.message().is_empty() => {
            format!("HTTP {}: {}", e.status, escape_controls(e.message()))
        }
        CallError::Http(e) => format!("HTTP {}", e.status),
        CallError::Timeout => "no answer in time".to_string(),
        CallError::Transport(detail) => escape_controls(detail),
        CallError::NoCredential(detail) | CallError::Unreadable(detail) => escape_controls(detail),
    }
}

/// A create's outcome.
pub(crate) enum Created {
    Sandbox(Box<Sandbox>),
    /// A definitive refusal: nothing was created.
    Refused(Failure),
    /// The sandbox may or may not exist.
    Unknown(Failure),
}

/// A 32-character random hex `Idempotency-Key`.
pub(crate) fn idempotency_key() -> String {
    use rand::Rng;
    let bytes: [u8; 16] = rand::thread_rng().gen();
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// `POST /sandboxes` under one `Idempotency-Key`, retried for up to five
/// minutes on the answers the key makes safe to retry: a transport error or
/// timeout, a 408 or any 5xx, a 409 `ALREADY_IN_PROGRESS` and, for 60 s of
/// waiting in all, a 429. Never a 402, since a retry after one is a new create
/// that runs the wallet's top-off again.
pub(crate) async fn create(
    api: &Api,
    body: &CreateSandboxRequest,
    key: &str,
    timing: &Timing,
) -> Created {
    let options = || RequestOptions {
        additional_headers: [("Idempotency-Key".to_string(), key.to_string())].into(),
        ..Default::default()
    };
    let name = body.name.as_str();
    let deadline = Instant::now() + timing.create_loop;
    let mut limited = Duration::ZERO;
    // Whether an earlier attempt may have created the sandbox unanswered.
    let mut unanswered = false;
    let refused = |failure: Failure, unanswered: bool| {
        if unanswered {
            Created::Refused(failure.then(may_exist(name)))
        } else {
            Created::Refused(failure)
        }
    };
    let mut backoff = Duration::from_secs(1);
    loop {
        let error = match attempt(
            timing.create_attempt,
            api.quiet.create(body, Some(options())),
        )
        .await
        {
            Ok(sandbox) => return Created::Sandbox(Box::new(sandbox)),
            Err(e) => e,
        };
        let subject = Some(body.name.as_str());
        let wait = match &error {
            CallError::Http(e) if e.is(409, "ALREADY_IN_PROGRESS") => retry_wait(&error, backoff),
            CallError::Http(e) if e.status == 429 => {
                let wait = retry_wait(&error, backoff);
                if limited + wait > timing.limit_wait {
                    return refused(error.failure(subject), unanswered);
                }
                limited += wait;
                wait
            }
            CallError::Http(e) if e.status == 408 || e.status >= 500 => retry_wait(&error, backoff),
            CallError::Transport(_) | CallError::Timeout => backoff,
            // An answer hedra-cli could not read may be a 201.
            CallError::Unreadable(_) => {
                return Created::Unknown(unknown(error.failure(subject), name))
            }
            _ => return refused(error.failure(subject), unanswered),
        };
        unanswered |= !matches!(&error, CallError::Http(e) if e.status == 429 || e.status == 409);
        if Instant::now() + wait > deadline {
            return Created::Unknown(unknown(error.failure(subject), name));
        }
        super::stream::note(format_args!(
            "Retrying the create in {}s ({}).",
            wait.as_secs().max(1),
            describe(&error)
        ));
        tokio::time::sleep(wait).await;
        backoff = next_backoff(backoff);
    }
}

/// The next step when a create may have made a sandbox it never confirmed.
pub(crate) fn may_exist(name: &str) -> String {
    format!(
        "It may exist: see `hedra-cli sandbox get --sandbox {name}`, and stop it with \
         `hedra-cli sandbox stop --sandbox {name}`."
    )
}

/// An outcome the create could not settle: Unavailable, with the next step.
fn unknown(f: Failure, name: &str) -> Failure {
    let mut f = Failure::new(Class::Unavailable, f.message).with_trace(f.trace_id);
    // The route.s message tells the caller to retry with the same
    // `Idempotency-Key`, which a person running the CLI cannot pass.
    if f.message.contains("Idempotency-Key") {
        f.message = format!("Could not tell whether `{name}` was created.");
    }
    f.then(may_exist(name))
}

/// The waits, as parameters so tests can shorten them.
pub(crate) struct Timing {
    pub(crate) create_loop: Duration,
    pub(crate) create_attempt: Duration,
    /// The 429 waits a create takes in all before it is Limit.
    pub(crate) limit_wait: Duration,
    pub(crate) ready_budget: Duration,
    pub(crate) fast_poll: Duration,
    pub(crate) slow_poll: Duration,
    pub(crate) fast_for: Duration,
    pub(crate) elapsed_every: Duration,
    pub(crate) stop_deadline: Duration,
    pub(crate) stop_attempt: Duration,
    pub(crate) stop_poll: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            create_loop: CREATE_LOOP,
            create_attempt: CREATE_ATTEMPT,
            limit_wait: Duration::from_secs(60),
            ready_budget: READY_BUDGET,
            fast_poll: Duration::from_secs(2),
            slow_poll: Duration::from_secs(5),
            fast_for: Duration::from_secs(60),
            elapsed_every: Duration::from_secs(15),
            stop_deadline: STOP_DEADLINE,
            stop_attempt: ATTEMPT,
            stop_poll: Duration::from_secs(2),
        }
    }
}

/// One line for what a poll saw: the sandbox's status and each replica's,
/// with an unknown status's controls escaped.
fn progress(s: &Sandbox) -> String {
    let replicas: Vec<String> = s
        .instances
        .iter()
        .flatten()
        .map(|i| format!("replica {} {}", i.replica, i.status))
        .collect();
    let line = if replicas.is_empty() {
        format!("{} is {}.", s.name, s.status)
    } else {
        format!("{} is {} ({}).", s.name, s.status, replicas.join(", "))
    };
    escape_controls(&line)
}

/// What a poll of a starting sandbox means.
pub(crate) enum Readiness {
    Ready,
    Starting,
    Failed(Failure),
}

pub(crate) fn readiness(s: &Sandbox) -> Readiness {
    let name = &s.name;
    match s.status {
        SandboxStatus::Failed => {
            return Readiness::Failed(Failure::new(
                Class::NotReady,
                format!("`{name}` failed before any instance was created, so nothing ran."),
            ))
        }
        SandboxStatus::Stopping | SandboxStatus::Ended => {
            let reason = stop_reason(s).unwrap_or_else(|| "no reason given".to_string());
            return Readiness::Failed(Failure::new(
                Class::NotReady,
                format!("`{name}` stopped before it was ready: {reason}."),
            ));
        }
        SandboxStatus::Active => {}
        _ => return Readiness::Starting,
    }
    let instances = s.instances.as_deref().unwrap_or_default();
    if let Some(failed) = instances
        .iter()
        .find(|i| matches!(i.status, SandboxInstanceStatus::Failed))
    {
        let reason = failed.failure_reason.as_ref();
        let class = match reason {
            Some(SandboxFailureReason::NoCapacity) => {
                return Readiness::Failed(Failure::new(
                    Class::NoCapacity,
                    format!(
                        "No `{}` is free right now. Run `hedra-cli sandbox gpus`.",
                        s.gpu
                    ),
                ))
            }
            Some(SandboxFailureReason::QuotaExceeded) => Class::Limit,
            _ => Class::NotReady,
        };
        let reason = reason.map_or("no reason given".to_string(), |r| r.to_string());
        return Readiness::Failed(Failure::new(
            class,
            format!(
                "Replica {} of `{name}` failed to start: {reason}.",
                failed.replica
            ),
        ));
    }
    if instances
        .iter()
        .any(|i| matches!(i.status, SandboxInstanceStatus::Stopped))
    {
        return Readiness::Failed(Failure::new(
            Class::NotReady,
            format!(
                "`{name}`'s command already exited before it was ready. Run \
                 `hedra-cli sandbox logs {name}` for its output."
            ),
        ));
    }
    let running = instances
        .iter()
        .filter(|i| matches!(i.status, SandboxInstanceStatus::Running))
        .count() as i64;
    if running >= s.replicas.max(1) {
        Readiness::Ready
    } else {
        Readiness::Starting
    }
}

/// Who waits for a sandbox to be ready, which sets the next step a failed
/// wait names.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Waiter {
    /// A reader of the sandbox's output, who did not start it.
    Reader,
    /// A caller that stops the sandbox itself when the wait fails.
    Stops,
}

/// Polls `GET /sandboxes/{id}` until the sandbox is ready or cannot be,
/// passing each change once to `say`. `started` is when the create began, so
/// the budget counts the create loop too.
pub(crate) async fn wait_ready(
    api: &Api,
    id: &str,
    name: &str,
    started: Instant,
    timing: &Timing,
    waiter: Waiter,
    say: &mut dyn FnMut(&str),
) -> Result<Sandbox, Failure> {
    let deadline = started + timing.ready_budget;
    let mut last_line = String::new();
    let mut last_elapsed = Instant::now();
    loop {
        let poll = Instant::now();
        match attempt(ATTEMPT, api.client.get(id, None)).await {
            Ok(sandbox) => {
                let line = progress(&sandbox);
                if line != last_line {
                    say(&escape_controls(&line));
                    last_line = line;
                }
                match readiness(&sandbox) {
                    Readiness::Ready => {
                        say(&format!(
                            "`{name}` is ready, {} after the create.",
                            human_duration(started.elapsed().as_secs() as i64)
                        ));
                        return Ok(sandbox);
                    }
                    Readiness::Failed(f) => return Err(f),
                    Readiness::Starting => {}
                }
            }
            Err(e) => match &e {
                CallError::Http(h) if h.status == 429 => {
                    tokio::time::sleep(retry_wait(&e, timing.slow_poll)).await;
                }
                CallError::Http(h) if h.status == 408 || h.status >= 500 => {}
                CallError::Transport(_) | CallError::Timeout => {}
                _ => return Err(e.failure(Some(name))),
            },
        }
        if Instant::now() >= deadline {
            let took = human_duration(started.elapsed().as_secs() as i64);
            let line = match waiter {
                Waiter::Reader => format!(
                    "`{name}` is still starting after {took}. Check on it with `hedra-cli \
                     sandbox get --sandbox {name}`."
                ),
                Waiter::Stops => format!("`{name}` is still starting after {took}."),
            };
            return Err(Failure::new(Class::NotReady, line));
        }
        if last_elapsed.elapsed() >= timing.elapsed_every {
            say(&format!(
                "Still starting, {} so far.",
                human_duration(started.elapsed().as_secs() as i64)
            ));
            last_elapsed = Instant::now();
        }
        let every = if started.elapsed() < timing.fast_for {
            timing.fast_poll
        } else {
            timing.slow_poll
        };
        let next = (poll + every).min(deadline);
        tokio::time::sleep_until(next).await;
    }
}

/// A stop's outcome.
#[cfg(test)]
pub(crate) struct Stopped {
    pub(crate) sandbox: Sandbox,
}

/// `POST /sandboxes/{id}/stop`, retried on 409 `ALREADY_IN_PROGRESS` (a
/// sandbox still `creating`), 408, 429, 5xx and transport errors, then
/// `GET` until `ended` or `failed`, all under one five-minute deadline.
#[cfg(test)]
pub(crate) async fn stop(
    api: &Api,
    id: &str,
    name: &str,
    timing: &Timing,
) -> Result<Stopped, Failure> {
    let started = Instant::now();
    let deadline = started + timing.stop_deadline;
    let accepted = request_stop(api, id, name, timing, deadline).await?;
    super::stream::note(format_args!(
        "Stopping `{name}`; billing ended at the stop request."
    ));
    let sandbox = wait_ended(api, id, name, timing, started, deadline, accepted).await?;
    Ok(Stopped { sandbox })
}

/// `POST /sandboxes/{id}/stop` until the route accepts it or `deadline`
/// passes; returns the sandbox the route answered.
pub(crate) async fn request_stop(
    api: &Api,
    id: &str,
    name: &str,
    timing: &Timing,
    deadline: Instant,
) -> Result<Sandbox, Failure> {
    let mut backoff = Duration::from_secs(1);
    let mut said_waiting = false;
    loop {
        let limit = within(timing.stop_attempt, deadline);
        let e = match attempt(limit, api.client.stop(id, None)).await {
            Ok(sandbox) => return Ok(sandbox),
            Err(e) => e,
        };
        let wait = match &e {
            CallError::Http(h) if h.is(409, "ALREADY_IN_PROGRESS") => {
                if !said_waiting {
                    super::stream::note(format_args!(
                        "`{name}` is still starting; waiting to stop it."
                    ));
                    said_waiting = true;
                }
                retry_wait(&e, backoff)
            }
            CallError::Http(h) if h.status == 408 || h.status == 429 || h.status >= 500 => {
                retry_wait(&e, backoff)
            }
            CallError::Transport(_) | CallError::Timeout => backoff,
            _ => return Err(e.failure(Some(name))),
        };
        if Instant::now() + wait >= deadline {
            let mut failure = Failure::new(
                Class::Unavailable,
                format!(
                    "Could not confirm the stop: `{name}` may still be running, or may still \
                     start. Retry with `{}`.",
                    super::render::stop_command(id)
                ),
            );
            if let CallError::Http(h) = &e {
                failure.trace_id = h.envelope.trace_id.clone();
            }
            return Err(failure);
        }
        tokio::time::sleep(wait).await;
        backoff = next_backoff(backoff);
    }
}

/// Polls an accepted stop until the sandbox is `ended` or `failed`, or
/// `deadline` passes.
pub(crate) async fn wait_ended(
    api: &Api,
    id: &str,
    name: &str,
    timing: &Timing,
    started: Instant,
    deadline: Instant,
    accepted: Sandbox,
) -> Result<Sandbox, Failure> {
    let mut last = accepted;
    loop {
        if matches!(last.status, SandboxStatus::Ended | SandboxStatus::Failed) {
            return Ok(last);
        }
        if Instant::now() + timing.stop_poll >= deadline {
            return Err(Failure::new(
                Class::Unavailable,
                format!(
                    "`{name}` is still stopping after {}; Hedra keeps stopping it. Check with \
                     `hedra-cli sandbox get --sandbox {id}`.",
                    human_duration(started.elapsed().as_secs() as i64)
                ),
            ));
        }
        tokio::time::sleep(timing.stop_poll).await;
        let limit = within(timing.stop_attempt, deadline);
        match attempt(limit, api.client.get(id, None)).await {
            Ok(s) => last = s,
            Err(CallError::Http(h)) if h.status < 500 && h.status != 408 && h.status != 429 => {
                return Err(CallError::Http(h).failure(Some(name)))
            }
            Err(_) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sandbox(status: &str, instances: serde_json::Value, end_reason: Option<&str>) -> Sandbox {
        serde_json::from_value(json!({
            "id": "sandbox_1", "name": "train", "gpu": "h100:1", "gpu_model": "h100",
            "replicas": 1, "status": status, "end_reason": end_reason,
            "created_at": "2026-11-03T12:00:00Z", "env_keys": [], "instances": instances,
            "billing": {"usd_per_hour": 1.0, "paid_usd": 0.0, "settled": false},
        }))
        .unwrap()
    }

    fn instance(status: &str, failure: Option<&str>) -> serde_json::Value {
        json!([{"replica": 0, "status": status, "failure_reason": failure}])
    }

    fn class(s: &Sandbox) -> Option<Class> {
        match readiness(s) {
            Readiness::Failed(f) => Some(f.class),
            _ => None,
        }
    }

    #[test]
    fn ready_once_every_replica_runs() {
        assert!(matches!(
            readiness(&sandbox("active", instance("running", None), None)),
            Readiness::Ready
        ));
        assert!(matches!(
            readiness(&sandbox("active", instance("starting", None), None)),
            Readiness::Starting
        ));
        assert!(matches!(
            readiness(&sandbox("creating", json!(null), None)),
            Readiness::Starting
        ));
    }

    #[test]
    fn each_ending_maps_to_its_class() {
        let failed = |reason| sandbox("active", instance("failed", Some(reason)), None);
        assert_eq!(class(&failed("image_pull_failed")), Some(Class::NotReady));
        assert_eq!(class(&failed("no_capacity")), Some(Class::NoCapacity));
        assert_eq!(class(&failed("quota_exceeded")), Some(Class::Limit));
        assert_eq!(
            class(&sandbox("active", instance("stopped", None), None)),
            Some(Class::NotReady)
        );
        assert_eq!(
            class(&sandbox(
                "stopping",
                instance("stopped", None),
                Some("caller_stopped")
            )),
            Some(Class::NotReady)
        );
        assert_eq!(
            class(&sandbox(
                "ended",
                instance("failed", Some("start_failed")),
                Some("instance_failed")
            )),
            Some(Class::NotReady)
        );
        assert_eq!(
            class(&sandbox("failed", json!(null), None)),
            Some(Class::NotReady)
        );
    }

    #[test]
    fn an_ended_sandbox_names_its_reason_and_the_instances() {
        let s = sandbox(
            "ended",
            instance("failed", Some("start_failed")),
            Some("instance_failed"),
        );
        let Readiness::Failed(f) = readiness(&s) else {
            panic!("not a failure")
        };
        assert!(
            f.message.contains("instance_failed (start_failed)"),
            "{}",
            f.message
        );
    }

    #[test]
    fn retry_after_is_capped() {
        let e = CallError::Http(super::super::error::HttpError::new(
            429,
            &json!({"error": {"code": "RATE_LIMITED", "message": "x", "retry_after": 600}})
                .to_string(),
        ));
        assert_eq!(retry_wait(&e, Duration::from_secs(1)), MAX_RETRY_AFTER);
        assert_eq!(
            retry_wait(&CallError::Timeout, Duration::from_secs(3)),
            Duration::from_secs(3)
        );
    }

    mod against_a_mock {
        use super::*;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        fn short() -> Timing {
            Timing {
                create_loop: Duration::from_secs(2),
                create_attempt: Duration::from_millis(500),
                limit_wait: Duration::from_millis(1_500),
                ready_budget: Duration::from_secs(1),
                fast_poll: Duration::from_millis(100),
                slow_poll: Duration::from_millis(100),
                fast_for: Duration::from_secs(60),
                elapsed_every: Duration::from_secs(60),
                stop_deadline: Duration::from_secs(2),
                stop_attempt: Duration::from_secs(15),
                stop_poll: Duration::from_millis(100),
            }
        }

        fn answer(status: u16, code: &str, retry_after: Option<u64>) -> ResponseTemplate {
            ResponseTemplate::new(status).set_body_json(json!({
                "error": {"code": code, "message": "m", "retry_after": retry_after},
                "trace_id": "t",
            }))
        }

        fn body() -> CreateSandboxRequest {
            serde_json::from_value(json!({"name": "train", "image": "i", "gpu": "h100:1"})).unwrap()
        }

        fn row(status: &str, instance: &str) -> serde_json::Value {
            serde_json::to_value(sandbox(status, instance_json(instance), None)).unwrap()
        }

        fn instance_json(status: &str) -> serde_json::Value {
            json!([{"replica": 0, "status": status}])
        }

        async fn posts(server: &MockServer) -> Vec<wiremock::Request> {
            server
                .received_requests()
                .await
                .unwrap()
                .into_iter()
                .filter(|r| r.method.as_str() == "POST")
                .collect()
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_create_still_failing_at_its_deadline_is_an_unknown_outcome() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(answer(503, "UNAVAILABLE", Some(0)))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            match create(&api, &body(), "k", &short()).await {
                Created::Unknown(f) => {
                    assert_eq!(f.class, Class::Unavailable);
                    assert!(
                        f.message.contains("hedra-cli sandbox get --sandbox train"),
                        "{}",
                        f.message
                    );
                    assert!(
                        f.message.contains("hedra-cli sandbox stop --sandbox train"),
                        "{}",
                        f.message
                    );
                }
                _ => panic!("not unknown"),
            }
            let sent = posts(&server).await;
            assert!(sent.len() > 1);
            assert!(sent
                .iter()
                .all(|r| r.headers.get("idempotency-key").unwrap() == "k"));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_201_that_cannot_be_read_is_an_unknown_outcome() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(ResponseTemplate::new(201).set_body_json(json!({"replicas": "one"})))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            assert!(matches!(
                create(&api, &body(), "k", &short()).await,
                Created::Unknown(_)
            ));
            assert_eq!(posts(&server).await.len(), 1);
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn rate_limits_past_the_budget_end_in_limit() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(answer(429, "RATE_LIMITED", Some(1)))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            match create(&api, &body(), "k", &short()).await {
                Created::Refused(f) => assert_eq!(f.class, Class::Limit),
                _ => panic!("not refused"),
            }
            assert_eq!(posts(&server).await.len(), 2);
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn rate_limits_after_an_unanswered_attempt_still_say_it_may_exist() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(answer(502, "BAD_GATEWAY", Some(0)))
                .up_to_n_times(1)
                .mount(&server)
                .await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(answer(429, "RATE_LIMITED", Some(1)))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                create_loop: Duration::from_secs(10),
                ..short()
            };
            match create(&api, &body(), "k", &timing).await {
                Created::Refused(f) => {
                    assert_eq!(f.class, Class::Limit);
                    assert!(
                        f.message.contains("hedra-cli sandbox stop --sandbox train"),
                        "{}",
                        f.message
                    );
                }
                _ => panic!("not refused"),
            }
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn rate_limits_alone_do_not_say_it_may_exist() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(answer(429, "RATE_LIMITED", Some(1)))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            match create(&api, &body(), "k", &short()).await {
                Created::Refused(f) => assert!(!f.message.contains("may exist"), "{}", f.message),
                _ => panic!("not refused"),
            }
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn the_ready_wait_rides_out_a_gateway_error() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(502).set_body_string("bad gateway"))
                .up_to_n_times(1)
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("active", "running")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                ready_budget: Duration::from_secs(10),
                ..short()
            };
            wait_ready(
                &api,
                "sandbox_1",
                "train",
                Instant::now(),
                &timing,
                Waiter::Stops,
                &mut |_| {},
            )
            .await
            .unwrap();
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn the_routes_idempotency_key_advice_is_replaced() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(ResponseTemplate::new(503).set_body_json(json!({
                    "error": {"code": "UNAVAILABLE", "retry_after": 0, "message":
                        "Could not tell whether the sandbox started; a retry with the same \
                         Idempotency-Key returns it."},
                })))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            match create(&api, &body(), "k", &short()).await {
                Created::Unknown(f) => {
                    assert!(!f.message.contains("Idempotency-Key"), "{}", f.message);
                    assert!(
                        f.message
                            .starts_with("Could not tell whether `train` was created."),
                        "{}",
                        f.message
                    );
                }
                _ => panic!("not unknown"),
            }
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_stop_rides_out_a_gateway_error_while_it_waits() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes/sandbox_1/stop"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("stopping", "running")))
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(502).set_body_string("bad gateway"))
                .up_to_n_times(1)
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("ended", "stopped")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let stopped = stop(&api, "sandbox_1", "train", &short()).await.unwrap();
            assert!(matches!(stopped.sandbox.status, SandboxStatus::Ended));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn an_attempt_past_its_timeout_is_retried_under_the_same_key() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(
                    ResponseTemplate::new(201)
                        .set_body_json(row("active", "starting"))
                        .set_delay(Duration::from_secs(5)),
                )
                .up_to_n_times(1)
                .mount(&server)
                .await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(ResponseTemplate::new(201).set_body_json(row("active", "starting")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                create_loop: Duration::from_secs(10),
                ..short()
            };
            assert!(matches!(
                create(&api, &body(), "k", &timing).await,
                Created::Sandbox(_)
            ));
            let sent = posts(&server).await;
            assert_eq!(sent.len(), 2);
            assert!(sent
                .iter()
                .all(|r| r.headers.get("idempotency-key").unwrap() == "k"));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_402_is_never_retried() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(answer(402, "INSUFFICIENT_BALANCE", None))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            match create(&api, &body(), "k", &short()).await {
                Created::Refused(f) => assert_eq!(f.class, Class::Payment),
                _ => panic!("not refused"),
            }
            assert_eq!(posts(&server).await.len(), 1);
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_stop_never_accepted_says_so_at_its_deadline() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes/sandbox_1/stop"))
                .respond_with(answer(409, "ALREADY_IN_PROGRESS", Some(0)))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let f = stop(&api, "sandbox_1", "train", &short())
                .await
                .err()
                .unwrap();
            assert_eq!(f.class, Class::Unavailable);
            assert!(
                f.message.contains("Could not confirm the stop"),
                "{}",
                f.message
            );
            assert!(
                f.message
                    .contains("hedra-cli sandbox stop --sandbox sandbox_1"),
                "{}",
                f.message
            );
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn an_accepted_stop_still_stopping_at_its_deadline_says_hedra_keeps_stopping_it() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes/sandbox_1/stop"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("stopping", "running")))
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("stopping", "running")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let f = stop(&api, "sandbox_1", "train", &short())
                .await
                .err()
                .unwrap();
            assert_eq!(f.class, Class::Unavailable);
            assert!(
                f.message.contains("Hedra keeps stopping it"),
                "{}",
                f.message
            );
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn the_ready_wait_ends_at_its_budget_without_a_stop_command() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("active", "starting")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            // A reader is not told to stop the sandbox, nor a caller that
            // stops it itself.
            for waiter in [Waiter::Reader, Waiter::Stops] {
                let f = wait_ready(
                    &api,
                    "sandbox_1",
                    "train",
                    Instant::now(),
                    &short(),
                    waiter,
                    &mut |_| {},
                )
                .await
                .err()
                .unwrap();
                assert_eq!(f.class, Class::NotReady);
                assert!(f.message.contains("still starting"), "{}", f.message);
                assert!(!f.message.contains("sandbox stop"), "{}", f.message);
            }
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn the_ready_wait_sleeps_a_429s_retry_after() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(answer(429, "RATE_LIMITED", Some(1)))
                .up_to_n_times(1)
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("active", "running")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                ready_budget: Duration::from_secs(10),
                ..short()
            };
            let started = Instant::now();
            wait_ready(
                &api,
                "sandbox_1",
                "train",
                started,
                &timing,
                Waiter::Stops,
                &mut |_| {},
            )
            .await
            .unwrap();
            assert!(started.elapsed() >= Duration::from_secs(1));
        }

        /// Answers each of `answers` once, in order, then `last` from then on.
        async fn mount_in_order(
            server: &MockServer,
            verb: &str,
            route: &str,
            answers: Vec<ResponseTemplate>,
            last: ResponseTemplate,
        ) {
            for answer in answers {
                Mock::given(method(verb))
                    .and(path(route))
                    .respond_with(answer)
                    .up_to_n_times(1)
                    .mount(server)
                    .await;
            }
            Mock::given(method(verb))
                .and(path(route))
                .respond_with(last)
                .mount(server)
                .await;
        }

        fn retryable(retry_after: u64) -> Vec<ResponseTemplate> {
            vec![
                answer(409, "ALREADY_IN_PROGRESS", Some(retry_after)),
                answer(429, "RATE_LIMITED", Some(retry_after)),
                answer(503, "UNAVAILABLE", Some(retry_after)),
            ]
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn create_waits_a_short_retry_after_not_its_backoff() {
            let server = MockServer::start().await;
            let created = ResponseTemplate::new(201).set_body_json(row("active", "starting"));
            mount_in_order(&server, "POST", "/sandboxes", retryable(0), created).await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                create_loop: Duration::from_secs(20),
                ..short()
            };
            let started = Instant::now();
            assert!(matches!(
                create(&api, &body(), "k", &timing).await,
                Created::Sandbox(_)
            ));
            assert_eq!(posts(&server).await.len(), 4);
            // Its backoff would wait 1 s, 2 s and 4 s.
            assert!(started.elapsed() < Duration::from_secs(1));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn create_waits_a_long_retry_after_not_its_backoff() {
            let server = MockServer::start().await;
            let created = ResponseTemplate::new(201).set_body_json(row("active", "starting"));
            let first = vec![answer(409, "ALREADY_IN_PROGRESS", Some(3))];
            mount_in_order(&server, "POST", "/sandboxes", first, created).await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                create_loop: Duration::from_secs(20),
                ..short()
            };
            let started = Instant::now();
            assert!(matches!(
                create(&api, &body(), "k", &timing).await,
                Created::Sandbox(_)
            ));
            assert_eq!(posts(&server).await.len(), 2);
            assert!(started.elapsed() >= Duration::from_secs(3));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn stop_waits_a_short_retry_after_not_its_backoff() {
            let server = MockServer::start().await;
            let ended = ResponseTemplate::new(200).set_body_json(row("ended", "stopped"));
            mount_in_order(
                &server,
                "POST",
                "/sandboxes/sandbox_1/stop",
                retryable(0),
                ended,
            )
            .await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                stop_deadline: Duration::from_secs(20),
                ..short()
            };
            let started = Instant::now();
            stop(&api, "sandbox_1", "train", &timing).await.unwrap();
            assert_eq!(posts(&server).await.len(), 4);
            // Its backoff would wait 1 s, 2 s and 4 s.
            assert!(started.elapsed() < Duration::from_secs(1));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn stop_waits_a_long_retry_after_not_its_backoff() {
            let server = MockServer::start().await;
            let ended = ResponseTemplate::new(200).set_body_json(row("ended", "stopped"));
            let first = vec![answer(409, "ALREADY_IN_PROGRESS", Some(3))];
            mount_in_order(&server, "POST", "/sandboxes/sandbox_1/stop", first, ended).await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                stop_deadline: Duration::from_secs(20),
                ..short()
            };
            let started = Instant::now();
            stop(&api, "sandbox_1", "train", &timing).await.unwrap();
            assert_eq!(posts(&server).await.len(), 2);
            assert!(started.elapsed() >= Duration::from_secs(3));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn the_ready_budget_counts_from_the_create() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("active", "starting")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                ready_budget: Duration::from_secs(3),
                ..short()
            };
            let called = Instant::now();
            let f = wait_ready(
                &api,
                "sandbox_1",
                "train",
                called - Duration::from_secs(3),
                &timing,
                Waiter::Stops,
                &mut |_| {},
            )
            .await
            .err()
            .unwrap();
            assert_eq!(f.class, Class::NotReady);
            assert!(called.elapsed() < Duration::from_secs(1));
        }

        async fn wait_lines(server: &MockServer, timing: &Timing) -> Vec<String> {
            let api = Api::for_tests(&server.uri());
            let mut lines = Vec::new();
            let _ = wait_ready(
                &api,
                "sandbox_1",
                "train",
                Instant::now(),
                timing,
                Waiter::Stops,
                &mut |l| lines.push(l.to_string()),
            )
            .await;
            lines
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn the_ready_wait_says_each_change_once() {
            let server = MockServer::start().await;
            let ok =
                |status, instance| ResponseTemplate::new(200).set_body_json(row(status, instance));
            let polls = vec![
                ResponseTemplate::new(200).set_body_json(sandbox("creating", json!(null), None)),
                ResponseTemplate::new(200).set_body_json(sandbox("creating", json!(null), None)),
                ok("active", "starting"),
                ok("active", "starting"),
            ];
            mount_in_order(
                &server,
                "GET",
                "/sandboxes/sandbox_1",
                polls,
                ok("active", "running"),
            )
            .await;
            let timing = Timing {
                ready_budget: Duration::from_secs(10),
                ..short()
            };
            let lines = wait_lines(&server, &timing).await;
            assert_eq!(
                lines[..3],
                [
                    "train is creating.",
                    "train is active (replica 0 starting).",
                    "train is active (replica 0 running).",
                ],
                "{lines:?}"
            );
            assert_eq!(lines.len(), 4, "{lines:?}");
            assert!(lines[3].starts_with("`train` is ready"), "{lines:?}");
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn the_ready_wait_says_the_elapsed_time_every_interval() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("active", "starting")))
                .mount(&server)
                .await;
            let timing = Timing {
                ready_budget: Duration::from_millis(1_000),
                elapsed_every: Duration::from_millis(300),
                ..short()
            };
            let lines = wait_lines(&server, &timing).await;
            let elapsed = lines
                .iter()
                .filter(|l| l.starts_with("Still starting, "))
                .count();
            // About one each 300 ms of a 1 s wait polled every 100 ms.
            assert!((2..=4).contains(&elapsed), "{lines:?}");
            assert_eq!(lines[0], "train is active (replica 0 starting).");
            assert_eq!(lines.len(), elapsed + 1, "{lines:?}");
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_stop_attempt_past_its_timeout_is_retried() {
            let server = MockServer::start().await;
            let ended = ResponseTemplate::new(200).set_body_json(row("ended", "stopped"));
            let hung = vec![ended.clone().set_delay(Duration::from_secs(5))];
            mount_in_order(&server, "POST", "/sandboxes/sandbox_1/stop", hung, ended).await;
            let api = Api::for_tests(&server.uri());
            let timing = Timing {
                stop_deadline: Duration::from_secs(10),
                stop_attempt: Duration::from_millis(300),
                ..short()
            };
            let started = Instant::now();
            stop(&api, "sandbox_1", "train", &timing).await.unwrap();
            assert_eq!(posts(&server).await.len(), 2);
            assert!(started.elapsed() < Duration::from_secs(3));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_stop_whose_attempts_hang_ends_by_its_deadline() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes/sandbox_1/stop"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(row("ended", "stopped"))
                        .set_delay(Duration::from_secs(10)),
                )
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            // The second attempt starts 1.8 s in, with 0.2 s left.
            let timing = Timing {
                stop_deadline: Duration::from_secs(2),
                stop_attempt: Duration::from_millis(800),
                ..short()
            };
            let started = Instant::now();
            let f = stop(&api, "sandbox_1", "train", &timing)
                .await
                .err()
                .unwrap();
            assert!(
                f.message.contains("Could not confirm the stop"),
                "{}",
                f.message
            );
            assert_eq!(posts(&server).await.len(), 2);
            assert!(started.elapsed() < Duration::from_millis(2_300));
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_stop_whose_polls_hang_ends_by_its_deadline() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes/sandbox_1/stop"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("stopping", "running")))
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(row("ended", "stopped"))
                        .set_delay(Duration::from_secs(10)),
                )
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            // The second poll starts 1.4 s in, with 0.6 s left.
            let timing = Timing {
                stop_deadline: Duration::from_secs(2),
                stop_attempt: Duration::from_millis(1_200),
                ..short()
            };
            let started = Instant::now();
            let f = stop(&api, "sandbox_1", "train", &timing)
                .await
                .err()
                .unwrap();
            assert!(
                f.message.contains("Hedra keeps stopping it"),
                "{}",
                f.message
            );
            assert!(started.elapsed() < Duration::from_millis(2_300));
        }
    }

    #[test]
    fn idempotency_keys_are_fresh_hex() {
        let (a, b) = (idempotency_key(), idempotency_key());
        assert_eq!(a.len(), 32);
        assert!(a.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(a, b);
    }
}
