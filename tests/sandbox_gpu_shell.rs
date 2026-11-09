//! `shell --gpu` on a pseudo-terminal, against a mock v3 API and a fake
//! broker: what it creates, and when it stops the sandbox.

#![cfg(unix)]

mod sandbox_stream_support;

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use sandbox_stream_support::*;
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

const WAIT: Duration = Duration::from_secs(30);
const GPU_ID: &str = "sandbox_9b2d5c1e-7a40-4f3b-8e61-2c5d9f0a1b7e";

/// Answers as `answer` does, and records that a request arrived, so a test
/// can signal hedra-cli only once it is waiting on that request.
struct Noted<R> {
    answer: R,
    arrived: Arc<AtomicBool>,
}

impl<R: Respond> Respond for Noted<R> {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        self.arrived.store(true, Ordering::SeqCst);
        self.answer.respond(req)
    }
}

/// Waits until `arrived` is set, failing the test after `WAIT`.
fn wait_until(arrived: &AtomicBool) {
    let deadline = std::time::Instant::now() + WAIT;
    while !arrived.load(Ordering::SeqCst) {
        assert!(
            std::time::Instant::now() < deadline,
            "the request never arrived"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// Answers each create with the sandbox it names, after `conflicts` 409s.
struct Create {
    conflicts: usize,
    seen: AtomicUsize,
}

impl Respond for Create {
    fn respond(&self, req: &Request) -> ResponseTemplate {
        if self.seen.fetch_add(1, Ordering::SeqCst) < self.conflicts {
            return envelope(
                409,
                "ALREADY_EXISTS",
                "A sandbox with this name exists.",
                None,
            );
        }
        let body: Value = serde_json::from_slice(&req.body).unwrap();
        let name = body["name"].as_str().unwrap();
        ResponseTemplate::new(201).set_body_json(sandbox(GPU_ID, name, "active", running()))
    }
}

async fn api(broker: &FakeBroker, conflicts: usize) -> MockServer {
    let stopped = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "ended",
        json!([{"replica": 0, "status": "stopped"}]),
    ));
    api_stopping_with(broker, conflicts, stopped).await
}

/// `api`, with the stop route answering `stop`.
async fn api_stopping_with(
    broker: &FakeBroker,
    conflicts: usize,
    stop: ResponseTemplate,
) -> MockServer {
    let get = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "active",
        running(),
    ));
    api_with(broker, conflicts, get, stop).await
}

/// `api_stopping_with`, also recording when the stop request arrives.
async fn api_stopping_noted(
    broker: &FakeBroker,
    stop: ResponseTemplate,
) -> (MockServer, Arc<AtomicBool>) {
    let arrived = Arc::new(AtomicBool::new(false));
    let get = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "active",
        running(),
    ));
    let noted = Noted {
        answer: stop,
        arrived: arrived.clone(),
    };
    (api_with(broker, 0, get, noted).await, arrived)
}

/// `api`, with `GET` answering `get` and the stop route `stop`.
async fn api_with(
    broker: &FakeBroker,
    conflicts: usize,
    get: impl Respond + 'static,
    stop: impl Respond + 'static,
) -> MockServer {
    let server = MockServer::start().await;
    mount_api(&server, broker, conflicts, get, stop).await;
    server
}

async fn mount_api(
    server: &MockServer,
    broker: &FakeBroker,
    conflicts: usize,
    get: impl Respond + 'static,
    stop: impl Respond + 'static,
) {
    Mock::given(method("POST"))
        .and(path("/v3/sandboxes"))
        .respond_with(Create {
            conflicts,
            seen: AtomicUsize::new(0),
        })
        .mount(server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{GPU_ID}")))
        .respond_with(get)
        .mount(server)
        .await;
    mount_session(server, GPU_ID, broker).await;
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{GPU_ID}/stop")))
        .respond_with(stop)
        .mount(server)
        .await;
}

/// What `shell --gpu` says when a signal arrives during the create.
const FINISHING: &str = "Finishing the create so the sandbox can be stopped";

const ARGS: [&str; 7] = [
    "sandbox",
    "shell",
    "--gpu",
    "h100:1",
    "--image",
    "pytorch/pytorch",
    "--max-duration",
];

fn on_pty(run: &Run, then: impl FnOnce(&OnPty)) -> Ended {
    let pty = Pty::open(24, 80);
    let mut args = ARGS.to_vec();
    args.push("2h");
    let child = pty.spawn(&run.command(&args));
    then(&child);
    let out = child.finish(WAIT);
    assert!(pty.cooked(), "the terminal was left raw");
    out
}

async fn creates(server: &MockServer) -> Vec<Value> {
    requests_to(server, "POST", "/v3/sandboxes")
        .await
        .iter()
        .map(|r| {
            let mut body: Value = serde_json::from_slice(&r.body).unwrap();
            body["idempotency-key"] = json!(r
                .headers
                .get("idempotency-key")
                .map(|v| v.to_str().unwrap().to_string()));
            body
        })
        .collect()
}

async fn stops(server: &MockServer) -> usize {
    requests_to(server, "POST", &format!("/v3/sandboxes/{GPU_ID}/stop"))
        .await
        .len()
}

#[tokio::test]
async fn it_creates_an_idle_sandbox_and_stops_it_when_the_shell_exits() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(3)]));
    let server = api(&broker, 0).await;
    let run = Run::new(&server);
    let out = on_pty(&run, |_| {});
    assert_eq!(out.code, 3, "{}", out.stderr);

    let sent = creates(&server).await;
    assert_eq!(sent.len(), 1);
    let body = &sent[0];
    assert_eq!(body["idle"], true);
    assert_eq!(body["idle_ttl_seconds"], 900);
    assert_eq!(body["ttl_seconds"], 7200);
    assert_eq!(body["gpu"], "h100:1");
    assert_eq!(body["image"], "pytorch/pytorch");
    assert!(body["name"].as_str().unwrap().starts_with("shell-"));
    assert!(body["idempotency-key"]
        .as_str()
        .is_some_and(|k| !k.is_empty()));
    assert_eq!(stops(&server).await, 1);
    let start = broker.calls()[0].start.clone().unwrap();
    assert!(start.tty);
    assert_eq!(start.argv[0], "sh");
    assert!(!out.stderr.contains("Connection lost"), "{}", out.stderr);
}

#[tokio::test]
async fn without_a_terminal_nothing_is_created() {
    let broker = FakeBroker::start();
    let server = api(&broker, 0).await;
    let mut args = ARGS.to_vec();
    args.push("2h");
    let out = Run::new(&server).run(&args);
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert!(
        class_line(&stderr(&out))
            .is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19): ")),
        "{}",
        stderr(&out)
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

/// A background job has the terminal on stdin and stdout but cannot take
/// it, so it is refused before the paid create, offering no form without a
/// terminal.
#[tokio::test]
async fn a_background_job_is_refused_before_the_create() {
    let broker = FakeBroker::start();
    let server = api(&broker, 0).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let mut args = ARGS.to_vec();
    args.push("2h");
    let child = pty.spawn(&in_background(&run.command(&args)));
    let out = child.finish(WAIT);
    assert_eq!(out.code, 255, "{}", out.stderr);
    assert_eq!(
        class_line(&out.stderr),
        Some(
            "hedra-cli: Rejected request (19): This needs the terminal: run it in the foreground."
        ),
        "{}",
        out.stderr
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn a_refusal_before_the_command_ran_stops_the_sandbox() {
    let after_start = |code, reason| {
        ExecScript::admit([
            Step::Start,
            Step::Fail(broker_status(
                code,
                reason,
                "refused before the command",
                &[],
            )),
        ])
    };
    let cases = [
        (
            after_start(Code::InvalidArgument, "invalid_argument"),
            "Rejected request (19)",
        ),
        (
            after_start(Code::NotFound, "not_found"),
            "Not ready or stopped (13)",
        ),
        (after_start(Code::Internal, "internal"), "Internal (1)"),
        (
            ExecScript::refuse(broker_status(
                Code::ResourceExhausted,
                "quota_exceeded",
                "the workspace is at its session limit",
                &[],
            )),
            "Limit (16)",
        ),
        // hedra-cli stores no scopes with a key, so a key without
        // `sandboxes:exec` creates the sandbox; the broker's refusal of the
        // first `Exec` is the check, and the sandbox is stopped.
        (
            ExecScript::refuse(broker_status(
                Code::PermissionDenied,
                "permission_denied",
                "the session lacks sandbox:exec",
                &[],
            )),
            "Missing scope (10)",
        ),
    ];
    for (script, class) in cases {
        let broker = FakeBroker::start();
        broker.push_exec(script);
        let server = api(&broker, 0).await;
        let out = on_pty(&Run::new(&server), |_| {});
        assert_eq!(out.code, 255, "{class}: {}", out.stderr);
        assert!(
            class_line(&out.stderr)
                .is_some_and(|l| l.starts_with(&format!("hedra-cli: {class}: "))),
            "{class}: {}",
            out.stderr
        );
        assert_eq!(stops(&server).await, 1, "{class}");
    }
}

/// A first session refused while the sandbox still reads running stops the
/// sandbox. The stop lines that follow are the next step, so the refusal
/// does not also say to check on it.
#[tokio::test]
async fn a_first_session_refused_while_running_names_no_check_before_the_stop() {
    let broker = FakeBroker::start();
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{GPU_ID}/session")))
        .respond_with(envelope(
            404,
            "NOT_FOUND",
            "No replica is serving sessions.",
            None,
        ))
        .with_priority(1)
        .mount(&server)
        .await;
    let get = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "active",
        running(),
    ));
    let stopped = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "ended",
        json!([{"replica": 0, "status": "stopped"}]),
    ));
    mount_api(&server, &broker, 0, get, stopped).await;
    let out = on_pty(&Run::new(&server), |_| {});
    assert_eq!(out.code, 255, "{}", out.stderr);
    // The name is the create's own, `shell-` and six random characters.
    let line = class_line(&out.stderr).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Not ready or stopped (13): `shell-")
            && line.ends_with("` is active."),
        "{}",
        out.stderr
    );
    assert!(!out.stderr.contains("Check on it"), "{}", out.stderr);
    assert_eq!(stops(&server).await, 1);
}

#[tokio::test]
async fn a_stream_dropped_after_start_keeps_the_sandbox_and_says_how_to_reopen_it() {
    let cases = [
        (
            Step::Fail(broker_status(
                Code::Unavailable,
                "sandbox_disconnected",
                "the supervisor went away",
                &[],
            )),
            " (the supervisor went away).",
        ),
        (
            Step::Fail(broker_status(
                Code::Unavailable,
                "rate_limited",
                "client fell behind the sandbox's output; reconnect",
                &[],
            )),
            " (client fell behind the sandbox's output; reconnect).",
        ),
        // The same codes as the stopping reasons, without an ErrorInfo.
        (
            Step::Fail(Status::internal("stream reset")),
            " (stream reset).",
        ),
        (Step::Fail(Status::not_found("gone")), " (gone)."),
        (Step::Close, ""),
    ];
    for (end, note) in cases {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, end]));
        let server = api(&broker, 0).await;
        let out = on_pty(&Run::new(&server), |_| {});
        assert_eq!(out.code, 255, "{note}: {}", out.stderr);
        let line = class_line(&out.stderr).unwrap_or_default();
        assert!(
            line.starts_with(&format!(
                "hedra-cli: Connection lost (18): Lost the connection to the sandbox{note}"
            )),
            "{line}"
        );
        let name = creates(&server).await[0]["name"]
            .as_str()
            .unwrap()
            .to_string();
        assert!(
            line.contains(&format!("`hedra-cli sandbox shell {name}`")),
            "{line}"
        );
        assert!(
            line.contains(&format!("`hedra-cli sandbox stop --sandbox {GPU_ID}`")),
            "{line}"
        );
        assert_eq!(stops(&server).await, 0, "{note}");
        assert_eq!(broker.calls().len(), 1, "start was sent again");
    }
}

#[tokio::test]
async fn a_signal_during_the_shell_sends_one_stop_and_exits_with_its_code() {
    for (sig, code) in [
        (libc::SIGTERM, 143),
        (libc::SIGHUP, 129),
        (libc::SIGINT, 130),
    ] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let server = api(&broker, 0).await;
        let out = on_pty(&Run::new(&server), |child| {
            broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
            child.signal(sig);
        });
        assert_eq!(out.code, code, "{}", out.stderr);
        assert_eq!(stops(&server).await, 1);
        assert_eq!(class_line(&out.stderr), None, "{}", out.stderr);
        broker.wait_for(WAIT, |c| c[0].client_gone);
    }
}

/// An unusable proxy is refused before the paid create, even with a bypass
/// list that the broker's host is not yet known against.
#[tokio::test]
async fn an_unusable_proxy_with_a_bypass_list_is_refused_before_the_create() {
    let broker = FakeBroker::start();
    let server = api(&broker, 0).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let mut args = ARGS.to_vec();
    args.push("2h");
    let mut cmd = run.command(&args);
    cmd.env("HTTPS_PROXY", "socks5://127.0.0.1:1080")
        .env("NO_PROXY", "example.com");
    let child = pty.spawn(&cmd);
    let out = child.finish(WAIT);
    assert_eq!(out.code, 255, "{}", out.stderr);
    assert_eq!(
        class_line(&out.stderr),
        Some(
            "hedra-cli: Rejected request (19): a SOCKS proxy is not supported for sandbox \
             sessions; use an http:// proxy (set by HTTPS_PROXY)"
        ),
        "{}",
        out.stderr
    );
    assert!(creates(&server).await.is_empty());
}

/// A bypass list that covers every host lets an unusable proxy through the
/// pre-create check, since the session would never use it.
#[tokio::test]
async fn a_bypass_list_covering_the_broker_skips_the_pre_create_proxy_check() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = api(&broker, 0).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let mut args = ARGS.to_vec();
    args.push("2h");
    let mut cmd = run.command(&args);
    cmd.env("HTTPS_PROXY", "socks5://127.0.0.1:1080")
        .env("NO_PROXY", "*");
    let child = pty.spawn(&cmd);
    let out = child.finish(WAIT);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert_eq!(creates(&server).await.len(), 1);
}

#[tokio::test]
async fn a_taken_name_is_retried_once_under_a_new_name_and_key() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = api(&broker, 1).await;
    let out = on_pty(&Run::new(&server), |_| {});
    assert_eq!(out.code, 0, "{}", out.stderr);
    let sent = creates(&server).await;
    assert_eq!(sent.len(), 2);
    assert_ne!(sent[0]["name"], sent[1]["name"]);
    assert_ne!(sent[0]["idempotency-key"], sent[1]["idempotency-key"]);
    assert_eq!(stops(&server).await, 1);
}

#[tokio::test]
async fn a_second_taken_name_is_rejected_and_stops_nothing() {
    let broker = FakeBroker::start();
    let server = api(&broker, 2).await;
    let out = on_pty(&Run::new(&server), |_| {});
    assert_eq!(out.code, 255, "{}", out.stderr);
    let err = out.stderr;
    let line = class_line(&err).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Rejected request (19): "),
        "{err}"
    );
    assert!(!line.contains("sandbox stop"), "{line}");
    assert_eq!(creates(&server).await.len(), 2);
    assert_eq!(stops(&server).await, 0);
    assert!(broker.calls().is_empty());
}

#[tokio::test]
async fn a_first_exec_that_answered_before_its_refusal_keeps_the_sandbox() {
    for reason in ["invalid_argument", "not_found", "internal"] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([
            Step::Start,
            Step::Stdout(b"$ ".to_vec()),
            Step::Fail(broker_status(Code::Internal, reason, "after output", &[])),
        ]));
        let server = api(&broker, 0).await;
        let out = on_pty(&Run::new(&server), |_| {});
        assert_eq!(out.code, 255, "{reason}: {}", out.stderr);
        let line = class_line(&out.stderr).unwrap_or_default();
        assert!(
            line.starts_with("hedra-cli: Connection lost (18): "),
            "{reason}: {line}"
        );
        assert!(
            line.contains("hedra-cli sandbox shell "),
            "{reason}: {line}"
        );
        assert_eq!(stops(&server).await, 0, "{reason}");
    }
}

#[tokio::test]
async fn ctrl_c_during_the_reads_before_the_create_creates_nothing() {
    let broker = FakeBroker::start();
    let server = api(&broker, 0).await;
    let arrived = Arc::new(AtomicBool::new(false));
    Mock::given(method("GET"))
        .and(path("/v3/sandboxes/gpus"))
        .respond_with(Noted {
            answer: ResponseTemplate::new(200)
                .set_body_json(json!({"data": []}))
                .set_delay(Duration::from_secs(4)),
            arrived: arrived.clone(),
        })
        .mount(&server)
        .await;
    let out = on_pty(&Run::new(&server), |child| {
        wait_until(&arrived);
        child.signal(libc::SIGINT);
    });
    assert_eq!(out.code, 130, "{}", out.stderr);
    assert!(creates(&server).await.is_empty());
    assert_eq!(stops(&server).await, 0);
}

/// A first Ctrl-C during the create does not abandon it; a second does, and
/// since no answer named the sandbox's id, hedra-cli stops nothing and says to
/// check the name before stopping it.
#[tokio::test]
async fn a_second_ctrl_c_abandons_the_create_and_stops_nothing() {
    let server = MockServer::start().await;
    let arrived = Arc::new(AtomicBool::new(false));
    Mock::given(method("POST"))
        .and(path("/v3/sandboxes"))
        .respond_with(Noted {
            answer: ResponseTemplate::new(201)
                .set_body_json(sandbox(GPU_ID, "shell-aaaaaa", "active", running()))
                .set_delay(Duration::from_secs(120)),
            arrived: arrived.clone(),
        })
        .mount(&server)
        .await;
    let out = on_pty(&Run::new(&server), |child| {
        wait_until(&arrived);
        child.signal(libc::SIGINT);
        // Once the first is taken, so the two are not coalesced.
        child.wait_for_stderr(FINISHING, WAIT);
        child.signal(libc::SIGINT);
    });
    assert_eq!(out.code, 255, "{}", out.stderr);
    let line = class_line(&out.stderr).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Interrupted (130): shell-")
            && line.contains(
                "may still be running; check it with hedra-cli sandbox get --sandbox shell-"
            )
            && line.contains("before stopping it with hedra-cli sandbox stop --sandbox shell-"),
        "{}",
        out.stderr
    );
    assert_eq!(creates(&server).await.len(), 1);
    assert_eq!(stops(&server).await, 0);
}

/// Mounts a create answering `create`, noting its arrival, with `GET`
/// answering `get` and the stop route a stopped sandbox.
async fn api_creating(create: impl Respond + 'static, get: Value) -> (MockServer, Arc<AtomicBool>) {
    let server = MockServer::start().await;
    let arrived = Arc::new(AtomicBool::new(false));
    Mock::given(method("POST"))
        .and(path("/v3/sandboxes"))
        .respond_with(Noted {
            answer: create,
            arrived: arrived.clone(),
        })
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{GPU_ID}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(get))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{GPU_ID}/stop")))
        .respond_with(ResponseTemplate::new(200).set_body_json(sandbox(
            GPU_ID,
            "shell-aaaaaa",
            "ended",
            json!([{"replica": 0, "status": "stopped"}]),
        )))
        .mount(&server)
        .await;
    (server, arrived)
}

/// One Ctrl-C during the create lets it finish, then stops the sandbox it
/// answered with, whether the first attempt answers or a replay does.
#[tokio::test]
async fn one_ctrl_c_during_the_create_stops_the_sandbox_it_answers_with() {
    let created = || {
        ResponseTemplate::new(201).set_body_json(sandbox(
            GPU_ID,
            "shell-aaaaaa",
            "active",
            running(),
        ))
    };
    let answers = [
        Sequence::new(vec![created().set_delay(Duration::from_secs(2))]),
        // A retried create replays the first's answer under the same key.
        Sequence::new(vec![
            envelope(
                409,
                "ALREADY_IN_PROGRESS",
                "The create is running.",
                Some(1),
            )
            .set_delay(Duration::from_secs(1)),
            created(),
        ]),
    ];
    for (i, answer) in answers.into_iter().enumerate() {
        let (server, arrived) =
            api_creating(answer, sandbox(GPU_ID, "shell-aaaaaa", "active", running())).await;
        let out = on_pty(&Run::new(&server), |child| {
            wait_until(&arrived);
            child.signal(libc::SIGINT);
        });
        assert_eq!(out.code, 130, "{i}: {}", out.stderr);
        assert!(out.stderr.contains(FINISHING), "{i}: {}", out.stderr);
        let keys: Vec<_> = creates(&server)
            .await
            .into_iter()
            .map(|c| c["idempotency-key"].clone())
            .collect();
        assert_eq!(keys.len(), i + 1, "{i}");
        assert!(keys.iter().all(|k| *k == keys[0]), "{i}: {keys:?}");
        assert_eq!(stops(&server).await, 1, "{i}");
    }
}

/// A Ctrl-C while the sandbox is starting stops it.
#[tokio::test]
async fn ctrl_c_during_the_ready_wait_stops_the_sandbox() {
    let starting = || {
        sandbox(
            GPU_ID,
            "shell-aaaaaa",
            "active",
            json!([{"replica": 0, "status": "starting"}]),
        )
    };
    let (server, _) = api_creating(
        ResponseTemplate::new(201).set_body_json(starting()),
        starting(),
    )
    .await;
    let out = on_pty(&Run::new(&server), |child| {
        child.wait_for_stderr("replica 0 starting", WAIT);
        child.signal(libc::SIGINT);
    });
    assert_eq!(out.code, 130, "{}", out.stderr);
    assert_eq!(stops(&server).await, 1);
}

#[tokio::test]
async fn ctrl_c_while_the_stop_after_sigterm_is_unanswered_exits_255() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let held = ResponseTemplate::new(200)
        .set_body_json(sandbox(GPU_ID, "shell-aaaaaa", "stopping", running()))
        .set_delay(Duration::from_secs(120));
    let (server, stop_arrived) = api_stopping_noted(&broker, held).await;
    let out = on_pty(&Run::new(&server), |child| {
        broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
        child.signal(libc::SIGTERM);
        wait_until(&stop_arrived);
        child.signal(libc::SIGINT);
    });
    assert_eq!(out.code, 255, "{}", out.stderr);
    let line = class_line(&out.stderr).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Interrupted (130): ") && line.contains("may still be running"),
        "{}",
        out.stderr
    );
    assert!(
        line.contains(&format!("hedra-cli sandbox stop --sandbox {GPU_ID}")),
        "{line}"
    );
}

#[tokio::test]
async fn a_second_sighup_while_the_stop_is_unanswered_does_not_abandon_it() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let slow = ResponseTemplate::new(200)
        .set_body_json(sandbox(GPU_ID, "shell-aaaaaa", "stopping", running()))
        .set_delay(Duration::from_secs(2));
    let (server, stop_arrived) = api_stopping_noted(&broker, slow).await;
    // Closing a terminal usually delivers two SIGHUPs.
    let out = on_pty(&Run::new(&server), |child| {
        broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
        child.signal(libc::SIGHUP);
        wait_until(&stop_arrived);
        child.signal(libc::SIGHUP);
    });
    assert_eq!(out.code, 129, "{}", out.stderr);
    assert_eq!(class_line(&out.stderr), None, "{}", out.stderr);
    assert_eq!(stops(&server).await, 1);
}

#[tokio::test]
async fn after_a_signal_the_stop_is_not_waited_out_until_ended() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    // The stop is accepted, and the sandbox never reads `ended`.
    let stopping = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "stopping",
        running(),
    ));
    let server = api_stopping_with(&broker, 0, stopping).await;
    let started = std::time::Instant::now();
    let out = on_pty(&Run::new(&server), |child| {
        broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
        child.signal(libc::SIGTERM);
    });
    assert_eq!(out.code, 143, "{}", out.stderr);
    assert!(started.elapsed() < Duration::from_secs(15));
    assert_eq!(stops(&server).await, 1);
}

#[tokio::test]
async fn a_failed_ready_wait_stops_the_sandbox() {
    let broker = FakeBroker::start();
    let failed = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "active",
        json!([{"replica": 0, "status": "failed", "failure_reason": "image_pull_failed"}]),
    ));
    let stopped = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "ended",
        json!([{"replica": 0, "status": "failed"}]),
    ));
    let server = api_with(&broker, 0, failed, stopped).await;
    let out = on_pty(&Run::new(&server), |_| {});
    assert_eq!(out.code, 255, "{}", out.stderr);
    let line = class_line(&out.stderr).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Not ready or stopped (13): ")
            && line.contains("image_pull_failed"),
        "{}",
        out.stderr
    );
    assert!(!line.contains("sandbox stop"), "{line}");
    assert_eq!(stops(&server).await, 1);
    assert!(broker.calls().is_empty());
}

/// A ready wait that sees the instance stop says so as `logs -f` does, and
/// names the sandbox's output by id: the stop that follows frees its name.
#[tokio::test]
async fn a_ready_wait_that_sees_the_instance_stop_names_its_output_by_id() {
    let broker = FakeBroker::start();
    let exited = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "active",
        json!([{"replica": 0, "status": "stopped", "started_at": "2026-11-03T12:01:00Z"}]),
    ));
    let stopped = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "ended",
        json!([{"replica": 0, "status": "stopped", "started_at": "2026-11-03T12:01:00Z"}]),
    ));
    let server = api_with(&broker, 0, exited, stopped).await;
    let out = on_pty(&Run::new(&server), |_| {});
    assert_eq!(out.code, 255, "{}", out.stderr);
    assert_eq!(
        class_line(&out.stderr).unwrap_or_default(),
        format!(
            "hedra-cli: Not ready or stopped (13): `shell-aaaaaa`'s instance stopped before it \
             was ready. Run `hedra-cli sandbox logs {GPU_ID}` for its output."
        ),
        "{}",
        out.stderr
    );
    assert!(!out.stderr.contains("already exited"), "{}", out.stderr);
    assert_eq!(stops(&server).await, 1);
    assert!(broker.calls().is_empty());
}

#[tokio::test]
async fn a_failed_ready_wait_whose_stop_gives_up_keeps_its_own_class_line() {
    let broker = FakeBroker::start();
    let failed = ResponseTemplate::new(200).set_body_json(sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "active",
        json!([{"replica": 0, "status": "failed", "failure_reason": "image_pull_failed"}]),
    ));
    let refused = envelope(401, "UNAUTHENTICATED", "Invalid API key.", None);
    let server = api_with(&broker, 0, failed, refused).await;
    let out = on_pty(&Run::new(&server), |_| {});
    assert_eq!(out.code, 255, "{}", out.stderr);
    let lines: Vec<&str> = out
        .stderr
        .lines()
        .filter(|l| l.starts_with("hedra-cli: "))
        .collect();
    assert_eq!(lines.len(), 1, "{}", out.stderr);
    assert!(
        lines[0].starts_with("hedra-cli: Not ready or stopped (13): "),
        "{}",
        out.stderr
    );
    assert!(
        out.stderr
            .contains(&format!("hedra-cli sandbox stop --sandbox {GPU_ID}"))
            && out.stderr.contains("idle TTL"),
        "{}",
        out.stderr
    );
    assert_eq!(stops(&server).await, 1);
}

#[tokio::test]
async fn a_mint_still_failing_after_its_retry_stops_the_sandbox() {
    let broker = FakeBroker::start();
    let server = MockServer::start().await;
    // Mounted first, so it answers before `api`'s session route.
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{GPU_ID}/session")))
        .respond_with(envelope(
            503,
            "UNAVAILABLE",
            "Sandbox sessions are unavailable.",
            Some(30),
        ))
        .with_priority(1)
        .mount(&server)
        .await;
    let row = |status| {
        ResponseTemplate::new(200).set_body_json(sandbox(GPU_ID, "shell-aaaaaa", status, running()))
    };
    mount_api(&server, &broker, 0, row("active"), row("ended")).await;
    let pty = Pty::open(24, 80);
    let mut args = ARGS.to_vec();
    args.push("2h");
    let out = pty
        .spawn(&Run::new(&server).command(&args))
        .finish(Duration::from_secs(90));
    assert_eq!(out.code, 255, "{}", out.stderr);
    assert!(
        class_line(&out.stderr).is_some_and(|l| l.starts_with("hedra-cli: Unavailable (17): ")),
        "{}",
        out.stderr
    );
    assert!(
        mints(&server, GPU_ID).await >= 2,
        "the mint was not retried"
    );
    assert_eq!(stops(&server).await, 1);
    assert!(broker.calls().is_empty());
}

#[tokio::test]
async fn a_stop_refused_401_exits_255_with_the_sign_in_line() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let refused = envelope(401, "UNAUTHENTICATED", "Invalid API key.", None);
    let server = api_stopping_with(&broker, 0, refused).await;
    let out = on_pty(&Run::new(&server), |_| {});
    assert_eq!(out.code, 255, "{}", out.stderr);
    let line = class_line(&out.stderr).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Not signed in (2): Could not stop `shell-"),
        "{}",
        out.stderr
    );
    assert!(
        line.contains(&format!("hedra-cli sandbox stop --sandbox {GPU_ID}")),
        "{line}"
    );
    assert_eq!(stops(&server).await, 1);
}

#[tokio::test]
async fn a_dropped_stream_says_how_the_sandbox_ended_without_a_reopen_command() {
    let mut ended = sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "ended",
        json!([{"replica": 0, "status": "stopped"}]),
    );
    ended["end_reason"] = json!("exited");
    let failed = sandbox(
        GPU_ID,
        "shell-aaaaaa",
        "active",
        json!([{"replica": 0, "status": "failed", "failure_reason": "oom_killed"}]),
    );
    for (after, says) in [
        (ended, "The sandbox stopped: exited."),
        (failed, "The sandbox's instance stopped (oom_killed)."),
    ] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Close]));
        // The ready wait reads it running; the read after the drop, `after`.
        let get = Sequence::new(vec![
            ResponseTemplate::new(200).set_body_json(sandbox(
                GPU_ID,
                "shell-aaaaaa",
                "active",
                running(),
            )),
            ResponseTemplate::new(200).set_body_json(after),
        ]);
        let server = api_with(&broker, 0, get, ResponseTemplate::new(500)).await;
        let out = on_pty(&Run::new(&server), |_| {});
        assert_eq!(out.code, 255, "{says}: {}", out.stderr);
        let line = class_line(&out.stderr).unwrap_or_default();
        assert!(
            line.starts_with("hedra-cli: Connection lost (18): Lost the connection to the sandbox")
                && line.ends_with(says),
            "{line}"
        );
        assert!(!line.contains("sandbox shell"), "{line}");
        assert_eq!(stops(&server).await, 0, "{says}");
    }
}

/// A read after the drop that fails says nothing of how the sandbox ended,
/// so it is taken as still running: the line still names the reopen and the
/// stop commands.
#[tokio::test]
async fn a_dropped_stream_whose_read_fails_still_prints_the_stop_command() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Close]));
    // The ready wait reads it running; every read after the drop fails.
    let get = Sequence::new(vec![
        ResponseTemplate::new(200).set_body_json(sandbox(
            GPU_ID,
            "shell-aaaaaa",
            "active",
            running(),
        )),
        ResponseTemplate::new(500),
    ]);
    let server = api_with(&broker, 0, get, ResponseTemplate::new(500)).await;
    let out = on_pty(&Run::new(&server), |_| {});
    assert_eq!(out.code, 255, "{}", out.stderr);
    let line = class_line(&out.stderr).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Connection lost (18): Lost the connection to the sandbox"),
        "{line}"
    );
    assert!(
        line.contains(&format!("`hedra-cli sandbox stop --sandbox {GPU_ID}`")),
        "{line}"
    );
    assert!(line.contains("is still running"), "{line}");
    assert!(
        requests_to(&server, "GET", &format!("/v3/sandboxes/{GPU_ID}"))
            .await
            .len()
            >= 2
    );
    assert_eq!(stops(&server).await, 0);
}

#[tokio::test]
async fn a_panic_in_the_shell_restores_the_terminal_and_prints_the_stop_command() {
    for place in ["main", "task"] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let server = api(&broker, 0).await;
        let pty = Pty::open(24, 80);
        let mut args = ARGS.to_vec();
        args.push("2h");
        let run = Run::new(&server);
        let mut cmd = run.command(&args);
        cmd.env("HEDRA_CLI_TEST_PANIC", place);
        let child = pty.spawn(&cmd);
        let out = child.finish(WAIT);
        assert_eq!(out.code, 255, "{place}: {}", out.stderr);
        assert!(pty.cooked(), "{place}: the terminal was left raw");
        assert_eq!(
            out.stderr.matches("injected panic").count(),
            1,
            "{}",
            out.stderr
        );
        let line = class_line(&out.stderr).unwrap_or_default();
        assert!(
            line.starts_with("hedra-cli: Panic (255): ")
                && line.contains(&format!("hedra-cli sandbox stop --sandbox {GPU_ID}")),
            "{place}: {}",
            out.stderr
        );
        // No stop is sent.
        assert_eq!(stops(&server).await, 0, "{place}");
    }
}

#[tokio::test]
async fn keys_typed_during_the_stop_are_left_for_the_local_shell() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let slow = ResponseTemplate::new(200)
        .set_body_json(sandbox(
            GPU_ID,
            "shell-aaaaaa",
            "ended",
            json!([{"replica": 0, "status": "stopped"}]),
        ))
        .set_delay(Duration::from_secs(2));
    let server = api_stopping_with(&broker, 0, slow).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let mut args = ARGS.to_vec();
    args.push("2h");
    let mut child = pty.spawn(&run.command(&args));
    while stops(&server).await == 0 {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    std::thread::sleep(Duration::from_millis(500));
    pty.type_keys(b"left\n");
    let out = child.finish(WAIT);
    assert_eq!(out.code, 0, "{}", out.stderr);
    assert!(
        child.leader_exits_within(Duration::from_secs(5)),
        "hedra-cli read the line typed after the shell ended"
    );
}
