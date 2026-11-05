//! `shell --gpu`: create a sandbox for one shell, open the shell, and stop
//! the sandbox when the shell ends.
//!
//! The sandbox is created idle with a 15-minute idle TTL, so one this
//! process never stops (a crash, a sleeping laptop, a dropped stream) ends
//! about 15 minutes after its last activity.

use std::collections::HashMap;
use std::time::Duration;

use hedra_cli_sandbox::session::Session;
use hedra_cli_sandbox::stream::status::reason;
use hedra_cli_sandbox::stream::transport::TransportConfig;
use hedra_cli_sdk::api::{CreateSandboxRequest, Sandbox, SandboxInstanceStatus, SandboxStatus};
use tokio::time::Instant;

use super::api::Api;
use super::error::{escape_controls, Class, Failure};
use super::exec::{self, Driven, Ended, Exec};
use super::lifecycle::{self, Created, Timing, Waiter};
use super::render;
use super::stream::{self, note, Minter, NextSig, Sig, Signals};

/// How long a create may run on after Windows' `CTRL_CLOSE_EVENT`, which
/// ends the process a few seconds later.
const CLOSE_GRACE: Duration = Duration::from_secs(3);

/// How long the sandbox may sit with no shell before it stops itself.
const IDLE_TTL_SECONDS: i64 = 15 * 60;
/// `--max-duration`'s default.
pub(crate) const DEFAULT_MAX_DURATION: i64 = 24 * 60 * 60;

pub(crate) struct GpuShell {
    pub(crate) gpu: String,
    pub(crate) image: String,
    pub(crate) ttl_seconds: i64,
    pub(crate) term: Option<String>,
}

static CREATED: std::sync::Mutex<Option<(String, String)>> = std::sync::Mutex::new(None);

/// Records the sandbox this invocation created, its name and id, for the
/// panic message.
fn remember(name: &str, id: &str) {
    if let Ok(mut c) = CREATED.lock() {
        *c = Some((name.to_string(), id.to_string()));
    }
}

/// The name and id of the sandbox this invocation created, if any.
pub(crate) fn created() -> Option<(String, String)> {
    CREATED.lock().ok().and_then(|c| c.clone())
}

/// `shell-` and six lowercase base32 characters.
pub(crate) fn generated_name() -> String {
    use rand::Rng;
    const ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";
    let mut rng = rand::thread_rng();
    let tail: String = (0..6)
        .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
        .collect();
    format!("shell-{tail}")
}

/// The sandbox's request.
fn request(shell: &GpuShell, name: &str) -> CreateSandboxRequest {
    CreateSandboxRequest {
        name: name.to_string(),
        image: shell.image.clone(),
        gpu: shell.gpu.clone(),
        replicas: None,
        resources: None,
        command: None,
        idle: Some(true),
        idle_ttl_seconds: Some(IDLE_TTL_SECONDS),
        restart: None,
        env: None,
        ttl_seconds: Some(shell.ttl_seconds),
        ports: None,
        secrets: None,
        egress: None,
    }
}

/// `id` is the created sandbox's id, which the stop command names.
fn idle_ttl_line(id: &str) -> String {
    format!(
        "Its idle TTL stops it about 15 minutes after its last activity; stop it sooner with \
         `{}`.",
        render::stop_command(id)
    )
}

/// Runs `shell --gpu` and returns the exit code.
pub(crate) async fn run(
    api: &Api,
    transport: TransportConfig,
    shell: GpuShell,
    json_errors: bool,
) -> i32 {
    let out = |f: Failure| async move {
        exec::say(&f, json_errors).await;
        255
    };
    // Everything that can be checked without a sandbox, before paying for one.
    if let Err(f) = stream::foreground_terminal() {
        return out(f).await;
    }
    stream::ignore_sigpipe();
    let mut signals = Signals::new();
    // No request to create has been sent: a signal here creates nothing.
    let warnings = tokio::select! {
        lines = super::warnings_before_create(api, &shell.gpu, super::PRE_CREATE_READ) => lines,
        sig = signals.next() => return sig.class().code(),
    };
    for line in warnings {
        note(line);
    }

    let started = Instant::now();
    let (sandbox, interrupted) =
        match create(api, &shell, &mut signals, json_errors, &Timing::default()).await {
            Ok(created) => created,
            Err(code) => return code,
        };
    let name = sandbox.name.clone();
    let id = sandbox.id.clone();
    remember(&name, &id);
    let timing = Timing::default();
    let stopper = Stopper {
        api,
        id: &id,
        name: &name,
        json: json_errors,
        timing: &timing,
    };
    if let Some(sig) = interrupted {
        // The create was finished only so the sandbox could be stopped.
        return stopper.stop(sig.class().code(), false, &mut signals).await;
    }

    let mut say = |l: &str| note(l);
    let ready = tokio::select! {
        r = lifecycle::wait_ready(api, &id, &name, started, &timing, Waiter::Stops, &mut say) => r,
        sig = signals.next() => return stopper.stop(sig.class().code(), false, &mut signals).await,
    };
    if let Err(f) = ready {
        return stopper.stop_after(&f, &mut signals).await;
    }

    let session = Session::new(
        Minter {
            api,
            id: id.clone(),
        },
        transport,
    );
    let shell_exec = Exec {
        argv: exec::shell_argv(),
        tty: true,
        no_stdin: false,
        env: HashMap::new(),
        term: shell.term.clone(),
    };
    let driven = exec::drive(api, &session, &id, &name, &shell_exec, &mut signals).await;
    finish(&stopper, driven, &mut signals).await
}

/// Creates the sandbox under a generated name, retrying once under a new
/// name and key when the name is taken. A Ctrl-C, SIGHUP or SIGTERM during
/// the create does not abandon it: the create runs on until it answers with
/// the sandbox, so the stop can name it. A second Ctrl-C abandons it, and
/// so does `CTRL_CLOSE_EVENT` after `CLOSE_GRACE`.
async fn create(
    api: &Api,
    shell: &GpuShell,
    signals: &mut impl NextSig,
    json_errors: bool,
    timing: &Timing,
) -> Result<(Sandbox, Option<Sig>), i32> {
    // The first signal, whose code a finished create exits with, and whether
    // a Ctrl-C has come, so the next abandons the create.
    let mut interrupted: Option<Sig> = None;
    let mut ctrl_c = false;
    for attempt_no in 0..2 {
        let name = generated_name();
        let key = lifecycle::idempotency_key();
        let body = request(shell, &name);
        note(format_args!("Creating `{name}`."));
        let created = lifecycle::create(api, &body, &key, timing);
        tokio::pin!(created);
        let abandon = |class| {
            Failure::new(
                class,
                format!(
                    "{name} may still be running; check it with hedra-cli sandbox get --sandbox \
                     {name} before stopping it with {}",
                    render::stop_command(&name)
                ),
            )
        };
        let closing = tokio::time::sleep(Duration::MAX);
        tokio::pin!(closing);
        let mut closed = false;
        let outcome = loop {
            tokio::select! {
                outcome = &mut created => break outcome,
                _ = &mut closing, if closed => {
                    exec::say(&abandon(Class::Hangup), json_errors).await;
                    return Err(255);
                }
                sig = signals.next_sig() => {
                    if sig == Sig::Int {
                        if ctrl_c {
                            exec::say(&abandon(Class::Interrupted), json_errors).await;
                            return Err(255);
                        }
                        ctrl_c = true;
                    }
                    if sig == Sig::Close && !closed {
                        closed = true;
                        closing.as_mut().reset(Instant::now() + CLOSE_GRACE);
                    }
                    interrupted.get_or_insert(sig);
                    // Awaited, not `note`, which blocks the runtime's thread
                    // while the terminal is slow to take the line.
                    stream::say_line(
                        "Finishing the create so the sandbox can be stopped; press Ctrl-C again to leave it.",
                    )
                    .await;
                }
            }
        };
        match outcome {
            Created::Sandbox(s) => return Ok((*s, interrupted)),
            Created::Refused(f) if f.reason.as_deref() == Some("ALREADY_EXISTS") => {
                // A live sandbox of the caller's holds the generated name;
                // nothing was created.
                if let Some(sig) = interrupted {
                    return Err(sig.class().code());
                }
                if attempt_no == 1 {
                    exec::say(&f.then("Rerun it."), json_errors).await;
                    return Err(255);
                }
            }
            Created::Refused(f) => {
                // A definitive refusal created nothing, so a signal that
                // arrived during the create needs no stop and no line.
                if let Some(sig) = interrupted {
                    return Err(sig.class().code());
                }
                exec::say(&f, json_errors).await;
                return Err(255);
            }
            Created::Unknown(f) => {
                // The sandbox may exist, so this is printed and exits 255 even
                // after a signal. Only an id recovered through this create's
                // own key is ever stopped; the name alone does not prove the
                // sandbox is ours.
                let note = unknown_note(&f.message);
                exec::say(&f.then(note), json_errors).await;
                return Err(255);
            }
        }
    }
    unreachable!("the loop returns on its second attempt")
}

/// The note after a create whose outcome is unknown, without repeating the
/// "could not tell" its message may already open with.
fn unknown_note(message: &str) -> &'static str {
    if message.starts_with("Could not tell whether") {
        "hedra-cli stops nothing."
    } else {
        "hedra-cli could not tell whether it was created, so it stops nothing."
    }
}

/// What becomes of the sandbox once the shell has failed.
#[derive(Debug, PartialEq, Eq)]
enum AfterFailure {
    /// The command never ran, so there is no state to keep: stop it.
    Stop,
    /// A dropped stream: keep it, since the shell may have state.
    Lost,
    /// hedra-cli's own failure after the command ran, such as a failed
    /// write: report it as itself and keep the sandbox.
    Kept,
}

fn after_failure(driven: &Driven, f: &Failure) -> AfterFailure {
    if !driven.started {
        return AfterFailure::Stop;
    }
    match &driven.status {
        // The broker ends a first Exec with these before forwarding `start`.
        Some(s)
            if !driven.answered
                && [
                    reason::INVALID_ARGUMENT,
                    reason::NOT_FOUND,
                    reason::INTERNAL,
                ]
                .iter()
                .any(|r| s.is(r)) =>
        {
            AfterFailure::Stop
        }
        Some(_) => AfterFailure::Lost,
        None if f.class == Class::ConnectionLost => AfterFailure::Lost,
        None => AfterFailure::Kept,
    }
}

/// What happens once the shell ends.
async fn finish(stopper: &Stopper<'_>, driven: Driven, signals: &mut impl NextSig) -> i32 {
    let Stopper {
        api,
        id,
        name,
        json,
        ..
    } = *stopper;
    match driven.ended {
        Ended::Exit(code) => stopper.stop(code, true, signals).await,
        // The terminal may be gone: stop without waiting for `ended`.
        Ended::Signal(sig) => stopper.stop(sig.class().code(), false, signals).await,
        Ended::Pipe => match stopper.stop(141, false, signals).await {
            141 => stream::die_by_sigpipe(),
            gave_up => gave_up,
        },
        Ended::Failed(ref f) => match after_failure(&driven, f) {
            AfterFailure::Stop => stopper.stop_after(f, signals).await,
            AfterFailure::Lost => {
                let mut f = Failure::new(
                    Class::ConnectionLost,
                    format!(
                        "Lost the connection to the sandbox{}. {}",
                        exec::broker_note(driven.status.as_ref()),
                        kept(api, id, name).await
                    ),
                );
                f.reason = driven.status.as_ref().and_then(|s| s.reason.clone());
                exec::say(&f, json).await;
                255
            }
            AfterFailure::Kept => {
                let f = f.clone().then(kept(api, id, name).await);
                exec::say(&f, json).await;
                255
            }
        },
    }
}

/// What to do about a sandbox kept after a dropped stream, from a fresh read.
async fn kept(api: &Api, id: &str, name: &str) -> String {
    let running = || {
        format!(
            "`{name}` is still running: reopen it within about 15 minutes with `hedra-cli sandbox \
             shell {name}`, or stop it with `{}`; otherwise its idle \
             TTL stops it about 15 minutes after its last activity. A reopened shell's exit does \
             not stop it, and a tmux server or a nohup or & process left in it does not count \
             as activity.",
            render::stop_command(id)
        )
    };
    let Ok(s) = super::api::attempt(super::api::ATTEMPT, api.client.get(id, None)).await else {
        return running();
    };
    let reason = render::stop_reason(&s)
        .map(|r| escape_controls(&r))
        .unwrap_or_else(|| "no reason given".into());
    match s.status {
        SandboxStatus::Ended => format!("The sandbox stopped: {reason}."),
        SandboxStatus::Stopping => format!("The sandbox is stopping: {reason}."),
        _ => match stream::replica0(&s) {
            Some(i)
                if matches!(
                    i.status,
                    SandboxInstanceStatus::Stopped | SandboxInstanceStatus::Failed
                ) =>
            {
                match &i.failure_reason {
                    Some(f) => format!(
                        "The sandbox's instance stopped ({}).",
                        escape_controls(&f.to_string())
                    ),
                    None => "The sandbox's instance stopped.".to_string(),
                }
            }
            _ => running(),
        },
    }
}

/// The sandbox `shell --gpu` stops, and how.
#[derive(Clone, Copy)]
struct Stopper<'a> {
    api: &'a Api,
    id: &'a str,
    name: &'a str,
    json: bool,
    timing: &'a Timing,
}

impl Stopper<'_> {
    /// Stops the sandbox and returns the exit code: `code` once the stop is
    /// accepted, 255 when it gives up or a Ctrl-C abandons it. With `wait` (the
    /// shell exited on its own), waits for `ended` after, saying so; a Ctrl-C
    /// during that wait ends it. A SIGHUP, SIGTERM or console close may mean the
    /// terminal is gone: it never abandons the stop, skips the wait, and after
    /// the shell's own exit becomes the exit code.
    async fn stop(&self, code: i32, wait: bool, signals: &mut impl NextSig) -> i32 {
        self.stop_with(code, wait, false, signals).await
    }

    /// Prints `f`, a failure before the shell ran, then stops the sandbox
    /// and exits 255. A stop that gives up adds the stop command and the
    /// idle TTL without a second class line.
    async fn stop_after(&self, f: &Failure, signals: &mut impl NextSig) -> i32 {
        exec::say(f, self.json).await;
        self.stop_with(255, false, true, signals).await
    }

    async fn stop_with(
        &self,
        mut code: i32,
        mut wait: bool,
        failed: bool,
        signals: &mut impl NextSig,
    ) -> i32 {
        let Stopper {
            api,
            id,
            name,
            json,
            timing,
        } = *self;
        let started = Instant::now();
        let deadline = started + timing.stop_deadline;
        let stop = lifecycle::request_stop(api, id, name, timing, deadline);
        tokio::pin!(stop);
        let accepted = loop {
            tokio::select! {
                r = &mut stop => break r,
                sig = signals.next_sig() => {
                    if sig != Sig::Int {
                        if wait {
                            code = sig.class().code();
                            wait = false;
                        }
                        continue;
                    }
                    // The stop was not answered: the sandbox may still be running.
                    let f = Failure::new(
                        Class::Interrupted,
                        format!(
                            "`{name}` may still be running; stop it with `{}`.",
                            render::stop_command(id)
                        ),
                    );
                    exec::say(&f, json).await;
                    return 255;
                }
            }
        };
        let accepted = match accepted {
            Ok(s) => s,
            Err(f) if failed => {
                stream::say_line(format_args!(
                    "Could not stop `{name}`: {} {}",
                    f.message,
                    idle_ttl_line(id)
                ))
                .await;
                return 255;
            }
            Err(mut f) => {
                f.message = format!("Could not stop `{name}`: {}", f.message);
                let f = f.then(idle_ttl_line(id));
                exec::say(&f, json).await;
                return 255;
            }
        };
        if !wait {
            return code;
        }
        note(format_args!(
            "Stopping `{name}`; this usually takes a minute or two. Billing ended at the stop request."
        ));
        tokio::select! {
            r = lifecycle::wait_ended(api, id, name, timing, started, deadline, accepted) => {
                if r.is_err() {
                    note(format_args!("`{name}` is stopping; Hedra keeps stopping it."));
                }
            }
            sig = signals.next_sig() => {
                if sig == Sig::Int {
                    note(format_args!("`{name}` is stopping."));
                } else {
                    code = sig.class().code();
                }
            }
        }
        code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_names_are_valid_sandbox_names() {
        for _ in 0..100 {
            let n = generated_name();
            assert!(super::super::names::is_sandbox_name(&n), "{n}");
            assert_eq!(n.len(), "shell-".len() + 6);
        }
    }

    fn driven(started: bool, answered: bool, reason: Option<&str>) -> Driven {
        Driven {
            ended: Ended::Exit(0),
            started,
            answered,
            status: reason.map(|r| hedra_cli_sandbox::stream::status::BrokerStatus {
                code: hedra_cli_sandbox::tonic::Code::Internal,
                reason: Some(r.to_string()),
                message: String::new(),
                metadata: Vec::new(),
            }),
        }
    }

    #[test]
    fn only_a_refusal_before_the_command_ran_stops_the_sandbox() {
        let lost = Failure::new(Class::ConnectionLost, "");
        let refused = Failure::new(Class::Rejected, "");
        assert_eq!(
            after_failure(&driven(false, false, None), &refused),
            AfterFailure::Stop
        );
        for r in [
            reason::INVALID_ARGUMENT,
            reason::NOT_FOUND,
            reason::INTERNAL,
        ] {
            assert_eq!(
                after_failure(&driven(true, false, Some(r)), &refused),
                AfterFailure::Stop,
                "{r}"
            );
            // Output came back, so the command ran.
            assert_eq!(
                after_failure(&driven(true, true, Some(r)), &refused),
                AfterFailure::Lost,
                "{r}"
            );
        }
        assert_eq!(
            after_failure(&driven(true, false, Some(reason::RATE_LIMITED)), &lost),
            AfterFailure::Lost
        );
        assert_eq!(
            after_failure(&driven(true, true, None), &lost),
            AfterFailure::Lost
        );
    }

    #[test]
    fn a_failed_write_after_the_command_ran_keeps_the_sandbox_as_itself() {
        let f = Failure::new(
            Class::OutputFailed,
            "Could not write the output: disk full.",
        );
        assert_eq!(
            after_failure(&driven(true, true, None), &f),
            AfterFailure::Kept
        );
    }

    /// Signals in order, then none.
    struct Injected(Vec<Sig>);

    impl NextSig for Injected {
        async fn next_sig(&mut self) -> Sig {
            match self.0.pop() {
                Some(sig) => sig,
                None => std::future::pending().await,
            }
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_signal_during_a_create_that_may_exist_still_exits_255() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        // An unanswered first attempt, then rate limits past the 429 budget:
        // the first attempt may have created the sandbox.
        Mock::given(method("POST"))
            .and(path("/sandboxes"))
            .respond_with(ResponseTemplate::new(502).set_body_string("bad gateway"))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path("/sandboxes"))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "error": {"code": "RATE_LIMITED", "message": "Slow down.", "retry_after": 1},
            })))
            .mount(&server)
            .await;
        let api = Api::for_tests(&server.uri());
        let shell = GpuShell {
            gpu: "h100:1".into(),
            image: "i".into(),
            ttl_seconds: DEFAULT_MAX_DURATION,
            term: None,
        };
        let timing = Timing {
            create_loop: Duration::from_secs(20),
            limit_wait: Duration::from_millis(500),
            ..Timing::default()
        };
        let created = tokio::time::timeout(
            Duration::from_secs(15),
            create(&api, &shell, &mut Injected(vec![Sig::Term]), false, &timing),
        )
        .await
        .expect("the create did not finish");
        // Not SIGTERM's 143: the failure, with the get and stop commands, is
        // printed and the exit is 255.
        assert_eq!(created.err(), Some(255));
    }

    #[test]
    fn an_unknown_create_says_it_could_not_tell_once() {
        assert_eq!(
            unknown_note("Could not tell whether `shell-aaaaaa` was created."),
            "hedra-cli stops nothing."
        );
        assert!(unknown_note("The request timed out.").contains("could not tell"));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn ctrl_close_during_a_hanging_create_abandons_it_within_its_grace() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sandboxes"))
            .respond_with(ResponseTemplate::new(201).set_delay(Duration::from_secs(60)))
            .mount(&server)
            .await;
        let api = Api::for_tests(&server.uri());
        let shell = GpuShell {
            gpu: "h100:1".into(),
            image: "i".into(),
            ttl_seconds: DEFAULT_MAX_DURATION,
            term: None,
        };
        let started = std::time::Instant::now();
        let created = tokio::time::timeout(
            Duration::from_secs(10),
            create(
                &api,
                &shell,
                &mut Injected(vec![Sig::Close]),
                false,
                &Timing::default(),
            ),
        )
        .await
        .expect("the create was not abandoned");
        assert_eq!(created.err(), Some(255));
        assert!(started.elapsed() < CLOSE_GRACE + Duration::from_secs(1));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_second_ctrl_c_abandons_the_create_after_a_sigterm() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sandboxes"))
            .respond_with(ResponseTemplate::new(201).set_delay(Duration::from_secs(60)))
            .mount(&server)
            .await;
        let api = Api::for_tests(&server.uri());
        let shell = GpuShell {
            gpu: "h100:1".into(),
            image: "i".into(),
            ttl_seconds: DEFAULT_MAX_DURATION,
            term: None,
        };
        // Signals pop from the end: SIGTERM, then two Ctrl-Cs.
        for signals in [
            vec![Sig::Int, Sig::Int, Sig::Term],
            vec![Sig::Int, Sig::Hup, Sig::Int],
        ] {
            let created = tokio::time::timeout(
                Duration::from_secs(5),
                create(
                    &api,
                    &shell,
                    &mut Injected(signals),
                    false,
                    &Timing::default(),
                ),
            )
            .await
            .expect("the second Ctrl-C did not abandon the create");
            assert_eq!(created.err(), Some(255));
        }
    }

    fn row(status: &str) -> serde_json::Value {
        serde_json::json!({
            "id": "sandbox_1", "name": "shell-abcdef", "gpu": "h100:1",
            "gpu_model": "h100", "replicas": 1, "status": status,
            "created_at": "2026-11-03T12:00:00Z", "env_keys": [], "url": null,
            "instances": [{"replica": 0, "status": "running"}],
            "billing": {"usd_per_hour": 2.5, "paid_usd": 0.0, "settled": false},
        })
    }

    /// The stop route answering `stop`, and `GET` answering `stopping`.
    async fn stop_server(stop: wiremock::ResponseTemplate) -> wiremock::MockServer {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/sandboxes/sandbox_1/stop"))
            .respond_with(stop)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/sandboxes/sandbox_1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(row("stopping")))
            .mount(&server)
            .await;
        server
    }

    fn short() -> Timing {
        Timing {
            stop_deadline: Duration::from_secs(2),
            stop_attempt: Duration::from_secs(1),
            stop_poll: Duration::from_millis(100),
            ..Timing::default()
        }
    }

    async fn stop_with(server: &wiremock::MockServer, wait: bool, signals: Vec<Sig>) -> i32 {
        let api = Api::for_tests(&server.uri());
        let timing = short();
        let stopper = Stopper {
            api: &api,
            id: "sandbox_1",
            name: "shell-abcdef",
            json: false,
            timing: &timing,
        };
        stopper.stop(3, wait, &mut Injected(signals)).await
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_stop_still_refused_409_at_its_deadline_gives_up_with_255() {
        let refused = wiremock::ResponseTemplate::new(409).set_body_json(serde_json::json!({
            "error": {"code": "ALREADY_IN_PROGRESS", "message": "Still creating.",
                      "retryable": true, "retry_after": 1},
        }));
        let server = stop_server(refused).await;
        let started = std::time::Instant::now();
        assert_eq!(stop_with(&server, true, vec![]).await, 255);
        assert!(started.elapsed() < Duration::from_secs(4));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn hangups_and_terminations_never_abandon_the_stop_and_skip_its_wait() {
        let slow = wiremock::ResponseTemplate::new(200)
            .set_body_json(row("stopping"))
            .set_delay(Duration::from_millis(300));
        let server = stop_server(slow).await;
        // Signals pop from the end.
        assert_eq!(
            stop_with(&server, true, vec![Sig::Hup, Sig::Hup]).await,
            129
        );
        assert_eq!(
            stop_with(&server, false, vec![Sig::Term, Sig::Hup]).await,
            3
        );
        // A Ctrl-C still abandons it, after a SIGTERM too.
        assert_eq!(
            stop_with(&server, false, vec![Sig::Int, Sig::Term]).await,
            255
        );
        assert_eq!(stop_with(&server, true, vec![Sig::Int]).await, 255);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_closed_stdout_with_a_stop_that_gives_up_exits_255() {
        let server = stop_server(wiremock::ResponseTemplate::new(403)).await;
        let api = Api::for_tests(&server.uri());
        let timing = short();
        let stopper = Stopper {
            api: &api,
            id: "sandbox_1",
            name: "shell-abcdef",
            json: false,
            timing: &timing,
        };
        let driven = Driven {
            ended: Ended::Pipe,
            started: true,
            answered: true,
            status: None,
        };
        assert_eq!(finish(&stopper, driven, &mut Injected(vec![])).await, 255);
    }

    #[test]
    fn the_request_is_idle_with_an_idle_ttl_and_a_lifetime() {
        let r = request(
            &GpuShell {
                gpu: "h100:1".into(),
                image: "i".into(),
                ttl_seconds: DEFAULT_MAX_DURATION,
                term: None,
            },
            "shell-abcdef",
        );
        assert_eq!(r.idle, Some(true));
        assert_eq!(r.idle_ttl_seconds, Some(IDLE_TTL_SECONDS));
        assert_eq!(r.ttl_seconds, Some(DEFAULT_MAX_DURATION));
        assert_eq!(r.command, None);
    }
}
