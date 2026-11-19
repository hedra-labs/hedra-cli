//! Creating a sandbox and waiting for it, and stopping one and waiting for
//! it to end: the loops whose retries the runtime's own cannot make safely.

use std::time::Duration;

use hedra_cli_sandbox::backoff::Backoff;
use hedra_cli_sdk::api::{
    CreateSandboxRequest, Sandbox, SandboxFailureReason, SandboxInstance, SandboxInstanceStatus,
    SandboxStatus,
};
use hedra_cli_sdk::RequestOptions;
use tokio::time::Instant;

use super::api::{attempt, Answer, Api, CallError, ATTEMPT};
use super::error::{escape_controls, Class, Failure};
use super::render::{human_duration, stop_reason};

/// How long `create` keeps retrying one create, under one key.
pub(crate) const CREATE_LOOP: Duration = Duration::from_secs(5 * 60);
/// The create loop's five minutes plus a full pool's fifteen.
pub(crate) const READY_BUDGET: Duration = Duration::from_secs(20 * 60);
/// A wait for a sandbox someone else created: the pool's 15 minutes.
pub(crate) const START_BUDGET: Duration = Duration::from_secs(15 * 60);

/// [`START_BUDGET`] for the waits of `logs -f`, `attach`, `port-forward`
/// and `restart`. Test hook (debug builds only):
/// `HEDRA_CLI_TEST_START_BUDGET_MS` sets it in milliseconds.
pub(crate) fn start_budget() -> Duration {
    #[cfg(debug_assertions)]
    if let Some(ms) = std::env::var("HEDRA_CLI_TEST_START_BUDGET_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
    {
        return Duration::from_millis(ms);
    }
    START_BUDGET
}
pub(crate) const CREATE_ATTEMPT: Duration = Duration::from_secs(45);
const STOP_DEADLINE: Duration = Duration::from_secs(5 * 60);
/// Between the create's and the stop's retries, whose deadlines run to
/// minutes: the cap spares a struggling route without leaving it untried for
/// long.
pub(crate) const ROUTE_BACKOFF: Backoff =
    Backoff::new(Duration::from_secs(1), Duration::from_secs(16));
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

/// A failed call in a few words, for a line that names the cause.
pub(crate) fn describe(error: &CallError) -> String {
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
    /// A spot create whose answer is not a spot sandbox: an API from before
    /// `spot` created it on demand, at the on-demand price.
    OnDemand(Box<Sandbox>),
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
#[cfg(test)]
pub(crate) async fn create(
    api: &Api,
    body: &CreateSandboxRequest,
    key: &str,
    timing: &Timing,
) -> Created {
    create_as(api, body, false, key, timing).await
}

/// [`create`], with `spot: {}` in the request when `spot` is set: a sandbox
/// on capacity its provider can take back, at the spot price. The answer to
/// a spot create that is not a spot sandbox is [`Created::OnDemand`].
pub(crate) async fn create_as(
    api: &Api,
    body: &CreateSandboxRequest,
    spot: bool,
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
    // A refusal after an unanswered attempt leaves the outcome unknown: that
    // attempt may have created the sandbox. It keeps the refusal's class.
    let refused = |failure: Failure, unanswered: bool| {
        if unanswered {
            Created::Unknown(failure.then(may_exist(name)))
        } else {
            Created::Refused(failure)
        }
    };
    let mut backoff = ROUTE_BACKOFF;
    loop {
        let answer = if spot {
            attempt(timing.create_attempt, create_spot(api, body, options())).await
        } else {
            attempt(
                timing.create_attempt,
                api.quiet.create(body, Some(options())),
            )
            .await
            .map(|sandbox| Created::Sandbox(Box::new(sandbox)))
        };
        let error = match answer {
            Ok(created) => return created,
            Err(e) => e,
        };
        let subject = Some(body.name.as_str());
        let wait = match &error {
            CallError::Http(e) if e.is(409, "ALREADY_IN_PROGRESS") => {
                retry_wait(&error, backoff.wait())
            }
            CallError::Http(e) if e.status == 429 => {
                let wait = retry_wait(&error, backoff.wait());
                if limited + wait > timing.limit_wait {
                    return refused(error.failure(subject), unanswered);
                }
                limited += wait;
                wait
            }
            CallError::Http(e) if e.status == 408 || e.status >= 500 => {
                retry_wait(&error, backoff.wait())
            }
            CallError::Transport(_) | CallError::Timeout => backoff.wait(),
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
        backoff.grow();
    }
}

/// One `POST /sandboxes` with `spot: {}` beside `body`'s fields, as the
/// generated `create` sends it but for that field, which this build's
/// request type does not carry. `spot` is an object and never a boolean: the
/// route takes its options there later. The answer's own `spot` says whether
/// the sandbox is one; an API from before the field ignores it and creates
/// the sandbox on demand.
async fn create_spot(
    api: &Api,
    body: &CreateSandboxRequest,
    options: RequestOptions,
) -> Result<Created, hedra_cli_sdk::ApiError> {
    let mut request = serde_json::to_value(body).map_err(hedra_cli_sdk::ApiError::Serialization)?;
    if let Some(fields) = request.as_object_mut() {
        fields.insert("spot".to_string(), serde_json::json!({}));
    }
    let raw: serde_json::Value = api
        .quiet
        .http_client
        .execute_request(
            reqwest::Method::POST,
            "sandboxes",
            Some(request),
            None,
            Some(options),
        )
        .await?;
    let sandbox: Sandbox =
        serde_json::from_value(raw.clone()).map_err(hedra_cli_sdk::ApiError::Serialization)?;
    Ok(if is_spot(&raw) {
        Created::Sandbox(Box::new(sandbox))
    } else {
        Created::OnDemand(Box::new(sandbox))
    })
}

/// Whether a sandbox, as the API answered it, is a spot one: `spot` is an
/// object, null or absent otherwise.
pub(crate) fn is_spot(raw: &serde_json::Value) -> bool {
    raw.get("spot").is_some_and(serde_json::Value::is_object)
}

/// Whether the API says on-demand capacity could hold a spot sandbox that
/// found none: `on_demand_available`, on the sandbox or on a failed
/// instance. None when the answer does not say.
fn on_demand_available(raw: &serde_json::Value) -> Option<bool> {
    let at = |v: &serde_json::Value| v.get("on_demand_available").and_then(|a| a.as_bool());
    at(raw).or_else(|| raw.get("instances")?.as_array()?.iter().find_map(at))
}

/// A spot sandbox's No capacity failure: there is no fallback to on-demand,
/// so the line says whether running it again without `--spot` would start.
fn spot_capacity(a: &Answer, failure: Failure) -> Failure {
    if failure.class != Class::NoCapacity || !is_spot(&a.raw) {
        return failure;
    }
    let gpu = escape_controls(&a.sandbox.gpu);
    let next = match on_demand_available(&a.raw) {
        Some(true) => format!(
            "On-demand `{gpu}` is free: create it again without `--spot`, at the on-demand price."
        ),
        Some(false) => format!(
            "None is free on demand either. Run `{}`.",
            super::render::GPUS_COMMAND
        ),
        None => format!(
            "A spot sandbox is never moved to on-demand: run `{}`, or create it again without `--spot`.",
            super::render::GPUS_COMMAND
        ),
    };
    Failure::new(
        Class::NoCapacity,
        format!("No spot `{gpu}` is free right now. {next}"),
    )
    .with_trace(failure.trace_id)
}

/// The next step when a create may have made a sandbox it never confirmed.
pub(crate) fn may_exist(name: &str) -> String {
    format!(
        "It may exist: see `{}`, and stop it with `{}`.",
        super::render::get_command(name),
        super::render::stop_command(name)
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
/// a replica marked `restarting` read as restarting, with an unknown
/// status's controls escaped.
fn progress(a: &Answer) -> String {
    let s = &a.sandbox;
    let replicas: Vec<String> = s
        .instances
        .iter()
        .flatten()
        .map(|i| {
            if i.restarting == Some(true) {
                format!("replica {} restarting", i.replica)
            } else {
                format!("replica {} {}", i.replica, i.status)
            }
        })
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

/// A poll judged as `waiter` needs: a reader follows its one replica alone,
/// so another replica's trouble neither holds nor fails its wait.
pub(crate) fn readiness(s: &Sandbox, waiter: Waiter) -> Readiness {
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
    let all = s.instances.as_deref().unwrap_or_default();
    let instances: Vec<_> = match waiter.replica() {
        Some(replica) => all.iter().filter(|i| i.replica == replica).collect(),
        None => all.iter().collect(),
    };
    let needed = match waiter {
        Waiter::Reader(_) | Waiter::Restart(_) => 1,
        Waiter::Stops => s.replicas.max(1),
    };
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
                        "No `{}` is free right now. Run `{}`.",
                        s.gpu,
                        super::render::GPUS_COMMAND
                    ),
                ))
            }
            Some(SandboxFailureReason::QuotaExceeded) => Class::Limit,
            _ => Class::NotReady,
        };
        let preempted = reason.is_some_and(super::render::preempted);
        let reason = reason.map_or("no reason given".to_string(), |r| r.to_string());
        let line = format!(
            "Replica {} of `{name}` failed to start: {reason}.",
            failed.replica
        );
        return Readiness::Failed(Failure::new(
            class,
            if preempted {
                format!("{line} {}", super::render::PREEMPTED)
            } else {
                line
            },
        ));
    }
    // Replica 0 first, whose output a plain `logs` reads.
    if let Some(stopped) = instances
        .iter()
        .filter(|i| matches!(i.status, SandboxInstanceStatus::Stopped))
        .min_by_key(|i| i.replica)
    {
        return Readiness::Failed(Failure::new(Class::NotReady, stopped_line(s, stopped)));
    }
    let running = instances
        .iter()
        .filter(|i| matches!(i.status, SandboxInstanceStatus::Running))
        .count() as i64;
    if running >= needed {
        Readiness::Ready
    } else {
        Readiness::Starting
    }
}

/// What a stopped instance means to a wait for it to run, then the next
/// step, which names the sandbox by id when it names it at all: a sandbox
/// whose wait fails may be stopped, and its name is then free for a new
/// one. A stopped instance is not always a command that exited: an idle or
/// TTL stop reads the same.
fn stopped_line(s: &Sandbox, stopped: &SandboxInstance) -> String {
    // A replica that ran has output, which `logs --replica` reads; another
    // that never ran names `get`, as the others may still be running.
    let whose = super::stream::whose(&s.name, stopped.replica);
    let step = super::stream::ended_step(s, stopped.replica);
    let reason = stopped
        .failure_reason
        .as_ref()
        .map(|f| format!(" ({f})"))
        .unwrap_or_default();
    let preempted = if stopped
        .failure_reason
        .as_ref()
        .is_some_and(super::render::preempted)
    {
        format!(" {}", super::render::PREEMPTED)
    } else {
        String::new()
    };
    escape_controls(&format!(
        "{whose} stopped before it was ready{reason}.{preempted} {step}"
    ))
}

/// Who waits for a sandbox to be ready, which sets the next step a failed
/// wait names.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Waiter {
    /// A reader of one replica's output or sessions, who did not start it.
    Reader(i64),
    /// A caller that stops the sandbox itself when the wait fails.
    Stops,
    /// `restart`, waiting for the one replica it restarted to run again.
    Restart(i64),
}

impl Waiter {
    /// The one replica a reader or a restart follows; None for every one.
    fn replica(self) -> Option<i64> {
        match self {
            Waiter::Reader(replica) | Waiter::Restart(replica) => Some(replica),
            Waiter::Stops => None,
        }
    }
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
    wait_ready_or_last(api, id, name, started, timing, waiter, say)
        .await
        .map_err(|w| w.failure)
}

/// A failed [`wait_ready`], with the last read of the sandbox it made.
pub(crate) struct WaitFailed {
    pub(crate) failure: Failure,
    /// None when no read succeeded.
    pub(crate) last: Option<Box<Sandbox>>,
}

/// [`wait_ready`], handing back the last read on failure, so a caller can
/// name the sandbox's state without reading it again.
pub(crate) async fn wait_ready_or_last(
    api: &Api,
    id: &str,
    name: &str,
    started: Instant,
    timing: &Timing,
    waiter: Waiter,
    say: &mut dyn FnMut(&str),
) -> Result<Sandbox, WaitFailed> {
    wait_until(api, id, name, started, timing, waiter, &mut |_| true, say)
        .await
        .map(|a| a.sandbox)
}

/// Whether `s` lists `replica` with `restarting: true`. v3 marks an
/// instance being replaced in place with that field and keeps its `status`
/// `running` and its first `started_at`, so the field is the only sign of
/// a restart in a read.
pub(crate) fn restarting(s: &Sandbox, replica: i64) -> bool {
    s.instances
        .iter()
        .flatten()
        .any(|i| i.replica == replica && i.restarting == Some(true))
}

/// Whether a read shows that the restart of `replica` began: the replica is
/// marked `restarting`, or is listed and not running, or the read lists
/// instances but not that one. A read with no instances, as `stop` answers
/// when its lookup fails, shows nothing either way.
pub(crate) fn restart_began(a: &Answer, replica: i64) -> bool {
    if a.sandbox.instances.is_none() {
        return false;
    }
    match super::stream::replica_at(&a.sandbox, replica) {
        None => true,
        Some(i) if !matches!(i.status, SandboxInstanceStatus::Running) => true,
        Some(i) => i.restarting == Some(true),
    }
}

/// Polls an accepted restart of `replica` until that replica runs again,
/// or cannot, passing each change once to `say`. `accepted` is the route's
/// answer. A read counts as running again only once it, or a read before
/// it, showed the restart begin ([`restart_began`]), and only while it no
/// longer marks the replica `restarting`, so the wait does not end on a
/// read taken before the replica went down or while it is being replaced.
pub(crate) async fn wait_restarted(
    api: &Api,
    accepted: &Answer,
    name: &str,
    replica: i64,
    started: Instant,
    timing: &Timing,
    say: &mut dyn FnMut(&str),
) -> Result<Answer, WaitFailed> {
    let mut began = restart_began(accepted, replica);
    let mut gate = |a: &Answer| {
        began |= restart_began(a, replica);
        began
    };
    wait_until(
        api,
        &accepted.sandbox.id,
        name,
        started,
        timing,
        Waiter::Restart(replica),
        &mut gate,
        say,
    )
    .await
}

/// The loop under [`wait_ready_or_last`] and [`wait_restarted`]: a read
/// [`readiness`] calls ready ends the wait only when `gate` passes it too.
#[allow(clippy::too_many_arguments)]
async fn wait_until(
    api: &Api,
    id: &str,
    name: &str,
    started: Instant,
    timing: &Timing,
    waiter: Waiter,
    gate: &mut dyn FnMut(&Answer) -> bool,
    say: &mut dyn FnMut(&str),
) -> Result<Answer, WaitFailed> {
    let deadline = started + timing.ready_budget;
    let mut last: Option<Box<Sandbox>> = None;
    let mut last_line = String::new();
    let mut last_elapsed = Instant::now();
    // The last read was ready but the gate held it: a restart not seen yet.
    let mut held = false;
    loop {
        let poll = Instant::now();
        match api.read(id, ATTEMPT).await {
            Ok(answer) => {
                let line = progress(&answer);
                if line != last_line {
                    say(&escape_controls(&line));
                    last_line = line;
                }
                // The gate sees every read, so a restart that begins and
                // ends between two ready reads is still counted.
                let passed = gate(&answer);
                held = false;
                let readiness = match (readiness(&answer.sandbox, waiter), waiter) {
                    // A replica being replaced reads `running`; it is not
                    // ready until the read stops marking it `restarting`.
                    (Readiness::Ready, Waiter::Restart(replica))
                        if restarting(&answer.sandbox, replica) =>
                    {
                        Readiness::Starting
                    }
                    (r, _) => r,
                };
                match readiness {
                    Readiness::Ready if passed => {
                        let took = human_duration(started.elapsed().as_secs() as i64);
                        say(&match waiter {
                            Waiter::Restart(replica) => format!(
                                "{} is running again, {took} after the restart.",
                                super::stream::subject(name, replica)
                            ),
                            _ => format!("`{name}` is ready, {took} after the create."),
                        });
                        return Ok(answer);
                    }
                    Readiness::Ready => {
                        held = true;
                        last = Some(Box::new(answer.sandbox));
                    }
                    Readiness::Failed(failure) => {
                        return Err(WaitFailed {
                            failure: spot_capacity(&answer, failure),
                            last: Some(Box::new(answer.sandbox)),
                        })
                    }
                    Readiness::Starting => last = Some(Box::new(answer.sandbox)),
                }
            }
            Err(e) => match &e {
                CallError::Http(h) if h.status == 429 => {
                    tokio::time::sleep(retry_wait(&e, timing.slow_poll)).await;
                }
                CallError::Http(h) if h.status == 408 || h.status >= 500 => {}
                CallError::Transport(_) | CallError::Timeout => {}
                // The read's own failure; an earlier read is stale beside it.
                _ => {
                    return Err(WaitFailed {
                        failure: e.failure(Some(name)),
                        last: None,
                    })
                }
            },
        }
        if Instant::now() >= deadline {
            let took = human_duration(started.elapsed().as_secs() as i64);
            let failure = match waiter {
                // Running all along, as far as the reads showed: whether it
                // restarted is unknown, as after a request with no answer.
                Waiter::Restart(replica) if held => Failure::new(
                    Class::Unavailable,
                    format!(
                        "{} reads running, but hedra-cli did not see it restart in {took}. \
                         Check on it with `{}`.",
                        super::stream::subject(name, replica),
                        super::render::get_command(id)
                    ),
                ),
                Waiter::Reader(replica) | Waiter::Restart(replica) => Failure::new(
                    Class::NotReady,
                    format!(
                        "{} is still starting after {took}. Check on it with `{}`.",
                        super::stream::subject(name, replica),
                        super::render::get_command(id)
                    ),
                ),
                Waiter::Stops => Failure::new(
                    Class::NotReady,
                    format!("`{name}` is still starting after {took}."),
                ),
            };
            return Err(WaitFailed { failure, last });
        }
        if last_elapsed.elapsed() >= timing.elapsed_every {
            let so_far = human_duration(started.elapsed().as_secs() as i64);
            say(&if held {
                format!("Waiting to see the restart, {so_far} so far.")
            } else {
                format!("Still starting, {so_far} so far.")
            });
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

/// `POST /sandboxes/{id}/stop` until the route accepts it or `deadline`
/// passes; returns the sandbox the route answered.
pub(crate) async fn request_stop(
    api: &Api,
    id: &str,
    name: &str,
    timing: &Timing,
    deadline: Instant,
) -> Result<Sandbox, Failure> {
    let mut backoff = ROUTE_BACKOFF;
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
                retry_wait(&e, backoff.wait())
            }
            CallError::Http(h) if h.status == 408 || h.status == 429 || h.status >= 500 => {
                retry_wait(&e, backoff.wait())
            }
            CallError::Transport(_) | CallError::Timeout => backoff.wait(),
            _ => return Err(e.failure(Some(name))),
        };
        if Instant::now() + wait >= deadline {
            let mut failure = Failure::new(
                Class::Unavailable,
                "Could not confirm the stop: it may still be running, or may still start.",
            );
            if let CallError::Http(h) = &e {
                failure.trace_id = h.envelope.trace_id.clone();
            }
            return Err(failure);
        }
        tokio::time::sleep(wait).await;
        backoff.grow();
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
                     `{}`.",
                    human_duration(started.elapsed().as_secs() as i64),
                    super::render::get_command(id)
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

    /// A stop's outcome.
    struct Stopped {
        sandbox: Sandbox,
    }

    /// `POST /sandboxes/{id}/stop`, retried on 409 `ALREADY_IN_PROGRESS` (a
    /// sandbox still `creating`), 408, 429, 5xx and transport errors, then
    /// `GET` until `ended` or `failed`, all under one five-minute deadline.
    async fn stop(api: &Api, id: &str, name: &str, timing: &Timing) -> Result<Stopped, Failure> {
        let started = Instant::now();
        let deadline = started + timing.stop_deadline;
        let accepted = request_stop(api, id, name, timing, deadline).await?;
        super::super::stream::note(format_args!(
            "Stopping `{name}`; billing ended at the stop request."
        ));
        let sandbox = wait_ended(api, id, name, timing, started, deadline, accepted).await?;
        Ok(Stopped { sandbox })
    }

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
        match readiness(s, Waiter::Stops) {
            Readiness::Failed(f) => Some(f.class),
            _ => None,
        }
    }

    /// A reader follows replica 0, so replica 1 failing or still starting
    /// neither fails nor holds its wait; a stop judges every replica.
    #[test]
    fn a_reader_waits_on_replica_zero_alone() {
        let mut s = sandbox(
            "active",
            json!([
                {"replica": 0, "status": "running", "failure_reason": null},
                {"replica": 1, "status": "failed", "failure_reason": "instance_failed"},
            ]),
            None,
        );
        s.replicas = 2;
        assert!(matches!(readiness(&s, Waiter::Reader(0)), Readiness::Ready));
        assert!(matches!(readiness(&s, Waiter::Stops), Readiness::Failed(_)));
        // A reader of replica 1 is judged by replica 1 alone.
        assert!(matches!(
            readiness(&s, Waiter::Reader(1)),
            Readiness::Failed(_)
        ));
        s.instances.as_mut().unwrap()[1].status = SandboxInstanceStatus::Starting;
        assert!(matches!(readiness(&s, Waiter::Reader(0)), Readiness::Ready));
        assert!(matches!(readiness(&s, Waiter::Stops), Readiness::Starting));
        assert!(matches!(
            readiness(&s, Waiter::Reader(1)),
            Readiness::Starting
        ));
        s.instances.as_mut().unwrap()[1].status = SandboxInstanceStatus::Running;
        s.instances.as_mut().unwrap()[0].status = SandboxInstanceStatus::Starting;
        assert!(matches!(readiness(&s, Waiter::Reader(1)), Readiness::Ready));
        assert!(matches!(
            readiness(&s, Waiter::Reader(0)),
            Readiness::Starting
        ));
    }

    /// A reader of replica N whose replica stopped is pointed at that
    /// replica's output; another replica's stop names `get`.
    #[test]
    fn a_readers_own_stopped_replica_names_its_logs() {
        let mut s = sandbox(
            "active",
            json!([
                {"replica": 0, "status": "running"},
                {"replica": 1, "status": "stopped", "started_at": "2026-11-03T12:01:00Z"},
            ]),
            None,
        );
        s.replicas = 2;
        let Readiness::Failed(f) = readiness(&s, Waiter::Reader(1)) else {
            panic!("replica 1 stopped");
        };
        assert_eq!(
            f.message,
            "Replica 1 of `train` stopped before it was ready. \
             Run `hedra-cli sandbox logs sandbox_1 --replica 1` for its output."
        );
    }

    /// A sandbox as the API answers it, typed and raw.
    fn answer_of(instances: serde_json::Value) -> Answer {
        let raw = serde_json::to_value(sandbox("active", json!(null), None))
            .map(|mut v| {
                v["instances"] = instances;
                v
            })
            .unwrap();
        Answer {
            sandbox: serde_json::from_value(raw.clone()).unwrap(),
            raw,
        }
    }

    /// Which reads show a restart of replica 0 begin. v3 keeps a replica
    /// being replaced `running` and marks it `restarting: true`.
    #[test]
    fn what_shows_a_restart_beginning() {
        let one = |status: &str, restarting: Option<bool>| {
            let mut i = json!({"replica": 0, "status": status,
                               "started_at": "2026-11-03T12:01:00Z"});
            if let Some(r) = restarting {
                i["restarting"] = json!(r);
            }
            answer_of(json!([i]))
        };
        for (a, began, why) in [
            (one("running", None), false, "running, no field"),
            (
                one("running", Some(false)),
                false,
                "running, not restarting",
            ),
            (one("running", Some(true)), true, "running, restarting"),
            (one("starting", None), true, "down"),
            (one("stopped", None), true, "down"),
            (
                answer_of(json!([{"replica": 1, "status": "running", "restarting": true}])),
                true,
                "listed without it",
            ),
            (answer_of(json!(null)), false, "no instances: shows nothing"),
        ] {
            assert_eq!(restart_began(&a, 0), began, "{why}");
        }
        // Another replica restarting is not this one.
        let other = answer_of(json!([
            {"replica": 0, "status": "running"},
            {"replica": 1, "status": "running", "restarting": true},
        ]));
        assert!(!restart_began(&other, 0));
        assert!(restarting(&other.sandbox, 1));
        assert!(!restarting(&other.sandbox, 0));
    }

    #[test]
    fn ready_once_every_replica_runs() {
        assert!(matches!(
            readiness(
                &sandbox("active", instance("running", None), None),
                Waiter::Stops
            ),
            Readiness::Ready
        ));
        assert!(matches!(
            readiness(
                &sandbox("active", instance("starting", None), None),
                Waiter::Stops
            ),
            Readiness::Starting
        ));
        assert!(matches!(
            readiness(&sandbox("creating", json!(null), None), Waiter::Stops),
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

    /// A stopped instance reads as the state line reads it, and the next
    /// step names the sandbox by id, whose name a stop frees: its output
    /// once it started, otherwise what its failure calls for.
    #[test]
    fn a_stopped_instance_names_the_sandbox_by_id() {
        let started = sandbox(
            "active",
            json!([{"replica": 0, "status": "stopped", "started_at": "2026-11-03T12:01:00Z"}]),
            None,
        );
        let never = sandbox("active", instance("stopped", Some("start_failed")), None);
        let mut second = sandbox(
            "active",
            json!([
                {"replica": 0, "status": "running", "started_at": "2026-11-03T12:01:00Z"},
                {"replica": 1, "status": "stopped"},
            ]),
            None,
        );
        second.replicas = 2;
        // Both stopped, replica 1 listed first: replica 0 is named.
        let mut both = sandbox(
            "active",
            json!([
                {"replica": 1, "status": "stopped"},
                {"replica": 0, "status": "stopped", "started_at": "2026-11-03T12:01:00Z"},
            ]),
            None,
        );
        both.replicas = 2;
        // Replica 1 ran, then stopped: its own output is named.
        let mut ran_1 = second.clone();
        ran_1.instances.as_mut().unwrap()[1].started_at =
            started.instances.as_ref().unwrap()[0].started_at;
        for (s, line) in [
            (
                ran_1,
                "Replica 1 of `train` stopped before it was ready. Run `hedra-cli sandbox logs \
                 sandbox_1 --replica 1` for its output.",
            ),
            (
                started,
                "`train`'s instance stopped before it was ready. Run `hedra-cli sandbox logs \
                 sandbox_1` for its output.",
            ),
            (
                never,
                "`train`'s instance stopped before it was ready (start_failed). Create another, \
                 or run `hedra-cli sandbox ls` for the running ones.",
            ),
            (
                both,
                "`train`'s instance stopped before it was ready. Run `hedra-cli sandbox logs \
                 sandbox_1` for its output.",
            ),
            (
                second,
                "Replica 1 of `train` stopped before it was ready. Check on it with `hedra-cli \
                 sandbox get --sandbox sandbox_1`.",
            ),
        ] {
            let Readiness::Failed(f) = readiness(&s, Waiter::Stops) else {
                panic!("not a failure")
            };
            assert_eq!(f.class, Class::NotReady);
            assert_eq!(f.message, line);
        }
    }

    #[test]
    fn an_ended_sandbox_names_its_reason_and_the_instances() {
        let s = sandbox(
            "ended",
            instance("failed", Some("start_failed")),
            Some("instance_failed"),
        );
        let Readiness::Failed(f) = readiness(&s, Waiter::Stops) else {
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

    #[test]
    fn a_preempted_instance_says_what_that_means() {
        let s = sandbox("active", instance("failed", Some("preempted")), None);
        let Readiness::Failed(f) = readiness(&s, Waiter::Stops) else {
            panic!("a preempted instance is a failure");
        };
        assert_eq!(f.class, Class::NotReady);
        assert_eq!(
            f.message,
            "Replica 0 of `train` failed to start: preempted. The provider took its GPU back, \
             and the charge stopped then; its disk is gone."
        );
        // Another reason reads as it did.
        let s = sandbox("active", instance("failed", Some("start_failed")), None);
        let Readiness::Failed(f) = readiness(&s, Waiter::Stops) else {
            panic!("a failed instance is a failure");
        };
        assert_eq!(
            f.message,
            "Replica 0 of `train` failed to start: start_failed."
        );
    }

    fn answered(spot: serde_json::Value, extra: serde_json::Value) -> Answer {
        let mut raw = serde_json::to_value(sandbox(
            "active",
            instance("failed", Some("no_capacity")),
            None,
        ))
        .unwrap();
        raw["spot"] = spot;
        if let Some(extra) = extra.as_object() {
            for (key, value) in extra {
                raw[key] = value.clone();
            }
        }
        Answer {
            sandbox: serde_json::from_value(raw.clone()).unwrap(),
            raw,
        }
    }

    fn no_capacity(a: &Answer) -> Failure {
        let Readiness::Failed(f) = readiness(&a.sandbox, Waiter::Stops) else {
            panic!("no capacity is a failure");
        };
        spot_capacity(a, f)
    }

    #[test]
    fn a_spot_sandbox_with_no_capacity_says_whether_on_demand_has_room() {
        let free = no_capacity(&answered(json!({}), json!({"on_demand_available": true})));
        assert_eq!(free.class, Class::NoCapacity);
        assert_eq!(
            free.message,
            "No spot `h100:1` is free right now. On-demand `h100:1` is free: create it again \
             without `--spot`, at the on-demand price."
        );
        let full = no_capacity(&answered(json!({}), json!({"on_demand_available": false})));
        assert_eq!(
            full.message,
            "No spot `h100:1` is free right now. None is free on demand either. Run \
             `hedra-cli sandbox gpus`."
        );
        // An API that does not say: no promise either way.
        let unsaid = no_capacity(&answered(json!({}), json!({})));
        assert_eq!(
            unsaid.message,
            "No spot `h100:1` is free right now. A spot sandbox is never moved to on-demand: \
             run `hedra-cli sandbox gpus`, or create it again without `--spot`."
        );
        // On the failed instance, where a later API may put it.
        let mut on_instance = answered(json!({}), json!({}));
        on_instance.raw["instances"][0]["on_demand_available"] = json!(true);
        assert!(no_capacity(&on_instance)
            .message
            .contains("On-demand `h100:1` is free"));
    }

    #[test]
    fn an_on_demand_sandbox_with_no_capacity_reads_as_before() {
        for spot in [json!(null), serde_json::Value::Null] {
            let f = no_capacity(&answered(spot, json!({"on_demand_available": true})));
            assert_eq!(
                f.message,
                "No `h100:1` is free right now. Run `hedra-cli sandbox gpus`."
            );
        }
        // Another failure of a spot sandbox is left alone.
        let mut a = answered(json!({}), json!({}));
        a.raw["instances"][0]["failure_reason"] = json!("start_failed");
        a.sandbox = serde_json::from_value(a.raw.clone()).unwrap();
        assert_eq!(
            no_capacity(&a).message,
            "Replica 0 of `train` failed to start: start_failed."
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

        /// `--spot` sends `spot: {}`, an object, beside the request's other
        /// fields and under the same key, and a spot answer is the sandbox.
        #[tokio::test(flavor = "multi_thread")]
        async fn a_spot_create_sends_spot_as_an_object() {
            let server = MockServer::start().await;
            let mut created = row("creating", "pending");
            created["spot"] = json!({});
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(ResponseTemplate::new(201).set_body_json(created))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let outcome = create_as(&api, &body(), true, "key-1", &short()).await;
            assert!(matches!(outcome, Created::Sandbox(s) if s.name == "train"));
            let sent = posts(&server).await;
            assert_eq!(sent.len(), 1);
            let request: serde_json::Value = serde_json::from_slice(&sent[0].body).unwrap();
            assert_eq!(request["spot"], json!({}));
            assert_eq!(request["name"], "train");
            assert_eq!(request["gpu"], "h100:1");
            assert_eq!(
                sent[0]
                    .headers
                    .get("Idempotency-Key")
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "key-1"
            );
        }

        /// An API from before `spot` ignores the field and creates the
        /// sandbox on demand, which the caller must not take for a spot one.
        #[tokio::test(flavor = "multi_thread")]
        async fn a_spot_create_answered_on_demand_is_told_apart() {
            for answer in [row("creating", "pending"), {
                let mut r = row("creating", "pending");
                r["spot"] = json!(null);
                r
            }] {
                let server = MockServer::start().await;
                Mock::given(method("POST"))
                    .and(path("/sandboxes"))
                    .respond_with(ResponseTemplate::new(201).set_body_json(answer))
                    .mount(&server)
                    .await;
                let api = Api::for_tests(&server.uri());
                let outcome = create_as(&api, &body(), true, "key-1", &short()).await;
                assert!(matches!(outcome, Created::OnDemand(s) if s.id == "sandbox_1"));
            }
        }

        /// Without `--spot` the request carries no `spot` at all.
        #[tokio::test(flavor = "multi_thread")]
        async fn an_on_demand_create_sends_no_spot() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(ResponseTemplate::new(201).set_body_json(row("creating", "pending")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let outcome = create_as(&api, &body(), false, "key-1", &short()).await;
            assert!(matches!(outcome, Created::Sandbox(_)));
            let sent = posts(&server).await;
            let request: serde_json::Value = serde_json::from_slice(&sent[0].body).unwrap();
            assert!(request.get("spot").is_none(), "{request}");
        }

        /// The route's refusal of `spot` creates nothing and is not retried.
        #[tokio::test(flavor = "multi_thread")]
        async fn a_refused_spot_create_is_a_refusal() {
            let server = MockServer::start().await;
            Mock::given(method("POST"))
                .and(path("/sandboxes"))
                .respond_with(ResponseTemplate::new(400).set_body_json(json!({
                    "error": {"code": "INVALID_ARGUMENT", "message": "m", "param": "spot",
                              "details": [{"field": "spot", "message": "m",
                                           "reason": "unsupported_combination"}]},
                })))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let Created::Refused(f) = create_as(&api, &body(), true, "key-1", &short()).await
            else {
                panic!("a 400 is a refusal");
            };
            assert_eq!(f.class, Class::Rejected);
            assert!(
                f.message.starts_with("This GPU model is not offered spot."),
                "{f:?}"
            );
            assert_eq!(posts(&server).await.len(), 1);
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
                Created::Unknown(f) => {
                    assert_eq!(f.class, Class::Limit);
                    assert!(
                        f.message.contains("hedra-cli sandbox stop --sandbox train"),
                        "{}",
                        f.message
                    );
                }
                _ => panic!("not unknown"),
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
            for waiter in [Waiter::Reader(0), Waiter::Stops] {
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
                assert!(
                    f.message.starts_with("`train` is still starting"),
                    "{}",
                    f.message
                );
                assert!(!f.message.contains("sandbox stop"), "{}", f.message);
            }
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn a_readers_wait_for_a_replica_past_0_names_that_replica() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/sandbox_1"))
                .respond_with(ResponseTemplate::new(200).set_body_json(row("active", "starting")))
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let f = wait_ready(
                &api,
                "sandbox_1",
                "train",
                Instant::now(),
                &short(),
                Waiter::Reader(1),
                &mut |_| {},
            )
            .await
            .err()
            .unwrap();
            assert_eq!(f.class, Class::NotReady);
            assert!(
                f.message
                    .starts_with("Replica 1 of `train` is still starting after "),
                "{}",
                f.message
            );
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
