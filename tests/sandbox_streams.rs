//! `exec`, `logs`, `logs -f` and `attach`, run as the built binary against a
//! mock v3 API and a fake broker.

#![cfg(unix)]

mod sandbox_stream_support;

use std::io::{Read, Write};
use std::os::unix::process::ExitStatusExt;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use sandbox_stream_support::*;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const WAIT: Duration = Duration::from_secs(20);

fn exec_args<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["sandbox", "exec", ID];
    args.extend_from_slice(extra);
    args
}

fn disconnected() -> Status {
    broker_status(
        Code::Unavailable,
        "sandbox_disconnected",
        "the sandbox's supervisor is not connected",
        &[],
    )
}

/// Every byte value, and then some, so nothing is decoded on the way.
fn binary(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i * 7 % 256) as u8).collect()
}

fn file_with(run: &Run, bytes: &[u8]) -> std::fs::File {
    let p = run.home.path().join("input.bin");
    std::fs::write(&p, bytes).unwrap();
    std::fs::File::open(p).unwrap()
}

/// Collects a child's stdout on a thread, so a test can wait on it while
/// the child runs.
fn collect(mut out: impl Read + Send + 'static) -> Arc<Mutex<Vec<u8>>> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let into = seen.clone();
    std::thread::spawn(move || {
        let mut buf = [0u8; 4096];
        while let Ok(n) = out.read(&mut buf) {
            if n == 0 {
                break;
            }
            into.lock().unwrap().extend_from_slice(&buf[..n]);
        }
    });
    seen
}

fn wait_until(what: &str, done: impl Fn() -> bool) {
    let deadline = std::time::Instant::now() + WAIT;
    while !done() {
        assert!(std::time::Instant::now() < deadline, "timed out: {what}");
        std::thread::sleep(Duration::from_millis(20));
    }
}

// ---------------------------------------------------------------- exec

#[tokio::test]
async fn exec_relays_output_byte_for_byte_and_passes_the_exit_code_through() {
    let broker = FakeBroker::start();
    let out_bytes = binary(70_000);
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(out_bytes.clone()),
        Step::Stderr(b"to stderr \xff\x00\n".to_vec()),
        Step::Stdout(b"tail".to_vec()),
        Step::Exit(7),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let out = run.run(&exec_args(&[
        "-n", "--env", "A=1", "--", "python", "-c", "x y",
    ]));
    assert_eq!(out.status.code(), Some(7), "{}", stderr(&out));
    let mut expected = out_bytes;
    expected.extend_from_slice(b"tail");
    assert!(out.stdout == expected, "stdout differs");
    assert_eq!(out.stderr, b"to stderr \xff\x00\n");

    let call = &broker.calls()[0];
    assert_eq!(
        call.authorization.as_deref(),
        Some("Bearer jwt.session.one")
    );
    let start = call.start.as_ref().unwrap();
    assert_eq!(start.argv, ["python", "-c", "x y"]);
    assert!(!start.tty);
    assert_eq!(start.env.get("A").map(String::as_str), Some("1"));
}

/// Without `HEDRA_CLI_SANDBOX_PREVIEW=1` the stream verbs are named nowhere,
/// in the `sandbox` group's help, a completion script, a typo suggestion or
/// `sandbox help <verb>`, and refuse to run, sending nothing: `exec` exits 255
/// under the 255 rule and `logs` exits 19. The generated commands beside them
/// are not gated.
#[tokio::test]
async fn without_the_preview_variable_the_stream_verbs_are_hidden_and_refuse() {
    let server = MockServer::start().await;
    let run = Run::new(&server);
    let help = run
        .command(&["sandbox", "--help"])
        .env_remove("HEDRA_CLI_SANDBOX_PREVIEW")
        .output()
        .unwrap();
    let help = stdout(&help);
    for verb in ["exec", "shell", "logs", "attach"] {
        assert!(
            !help
                .lines()
                .any(|l| l.trim_start().starts_with(&format!("{verb} "))),
            "{verb} is listed: {help}"
        );
    }
    assert!(
        help.lines().any(|l| l.trim_start().starts_with("get ")),
        "{help}"
    );

    for shell in ["zsh", "bash"] {
        let script = run
            .command(&["completion", shell])
            .env_remove("HEDRA_CLI_SANDBOX_PREVIEW")
            .output()
            .unwrap();
        let script = stdout(&script);
        assert!(
            script.contains("gpus"),
            "{shell}: the generated commands are missing"
        );
        assert!(!script.contains("relaying its stdin"), "{shell} names exec");
    }
    for args in [&["sandbox", "exe"][..], &["sandbox", "help", "exec"][..]] {
        let out = run
            .command(args)
            .env_remove("HEDRA_CLI_SANDBOX_PREVIEW")
            .output()
            .unwrap();
        let text = format!("{}{}", stdout(&out), stderr(&out));
        assert!(!text.contains("similar subcommand"), "{args:?}: {text}");
        assert!(!text.contains("relaying its stdin"), "{args:?}: {text}");
    }

    let out = run
        .command(&["sandbox", "exec", ID, "-n", "--", "true"])
        .env_remove("HEDRA_CLI_SANDBOX_PREVIEW")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("(exec, shell, logs, attach) are not released yet"),
        "{}",
        stderr(&out)
    );
    let out = run
        .command(&["sandbox", "logs", ID])
        .env_remove("HEDRA_CLI_SANDBOX_PREVIEW")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(19), "{}", stderr(&out));
    assert!(server.received_requests().await.unwrap().is_empty());

    let with_preview = stdout(&run.run(&["sandbox", "--help"]));
    assert!(
        with_preview
            .lines()
            .any(|l| l.trim_start().starts_with("exec ")),
        "{with_preview}"
    );
}

/// A name is resolved once, through `GET /sandboxes/{name}`, and the stream
/// uses the id it answers.
#[tokio::test]
async fn a_name_resolves_once_through_the_sandbox_route() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    mount_sandbox(&server, "box", sandbox(ID, "box", "active", running())).await;
    let out = Run::new(&server).run(&["sandbox", "exec", "box", "-n", "--", "true"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        requests_to(&server, "GET", "/v3/sandboxes/box").await.len(),
        1
    );
    assert_eq!(mints(&server, ID).await, 1);
}

/// The static segments under `/sandboxes` are refused as names before any
/// request, since `GET /sandboxes/gpus` is the GPU list.
#[tokio::test]
async fn a_reserved_name_is_refused_before_any_request() {
    let server = MockServer::start().await;
    let out = Run::new(&server).run(&["sandbox", "exec", "gpus", "-n", "--", "true"]);
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert!(
        class_line(&stderr(&out))
            .is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19): ")),
        "{}",
        stderr(&out)
    );
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn exec_n_sends_stdin_eof_at_once_and_reads_no_stdin() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::StdinEof,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    // A stdin that never ends: only -n lets the command see its end.
    let mut child = run
        .command(&exec_args(&["-n", "--", "true"]))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = child.stdin.take().unwrap();
    let out = wait(child, WAIT);
    drop(stdin);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let call = &broker.calls()[0];
    assert!(call.stdin_eof);
    assert!(call.stdin.is_empty());
}

#[tokio::test]
async fn stdin_ending_sends_stdin_eof_and_the_call_stays_open_until_the_exit_code() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Count,
        Step::Sleep(Duration::from_millis(300)),
        Step::PrintCount,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let input = binary(100_000);
    // exec -- wc -c < file
    let out = run
        .command(&exec_args(&["--", "wc", "-c"]))
        .stdin(file_with(&run, &input))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "100000\n");
    let call = &broker.calls()[0];
    assert!(call.stdin == input, "stdin differs");
    assert!(call.stdin_eof);
}

#[tokio::test]
async fn a_call_refused_before_admission_sends_nothing_and_the_retry_delivers_stdin() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(disconnected()));
    broker.push_exec(ExecScript::admit([Step::Start, Step::Echo, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let input = binary(200_000);
    let out = run
        .command(&exec_args(&["--", "cat"]))
        .stdin(file_with(&run, &input))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(out.stdout == input, "the echo differs from stdin");
    assert!(
        stderr(&out).contains("`sandbox_3f2504e0-4f89-41d3-9a0c-0305e82c3301`'s supervisor is not connected yet; retrying."),
        "{}",
        stderr(&out)
    );
    let calls = broker.calls();
    assert_eq!(calls.len(), 2);
    assert!(!calls[0].admitted);
    assert_eq!(calls[0].frames_before_admission, 0);
    assert!(calls[1].stdin == input, "the broker's stdin differs");
    // sandbox_disconnected makes the next call mint again.
    assert_eq!(mints(&server, ID).await, 2);
}

#[tokio::test]
async fn unavailable_without_error_info_is_retried_on_the_same_session() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(Status::unavailable("restarting")));
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "true"]));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("Could not reach the sandbox broker; retrying."),
        "{}",
        stderr(&out)
    );
    assert_eq!(broker.calls().len(), 2);
    assert_eq!(mints(&server, ID).await, 1);
}

#[tokio::test]
async fn ctrl_c_and_sigterm_without_t_cancel_the_call_and_exit_130_and_143() {
    for (sig, code) in [(libc::SIGINT, 130), (libc::SIGTERM, 143)] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        // stdin is a file: the cancel must not reach the command as its end.
        let child = run
            .command(&exec_args(&["--", "cat"]))
            .stdin(file_with(&run, b"x"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.stdin_eof));
        signal(&child, sig);
        let out = wait(child, WAIT);
        assert_eq!(out.status.code(), Some(code), "{}", stderr(&out));
        assert_eq!(class_line(&stderr(&out)), None, "{}", stderr(&out));
        let calls = broker.wait_for(WAIT, |c| c[0].reset.is_some() || c[0].half_closed);
        assert!(
            calls[0].reset.is_some() && !calls[0].half_closed,
            "the call was not reset: {:?}",
            calls[0]
        );
    }
}

#[tokio::test]
async fn sigterm_while_a_write_waits_on_an_unread_pipe_cancels_the_call_and_exits_143() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::StdoutForever(vec![b'z'; 64 * 1024], Duration::from_millis(1)),
    ]));
    let server = running_sandbox(&broker).await;
    // stdout is a pipe nobody reads, so a write soon waits on it.
    let child = Run::new(&server).spawn(&exec_args(&["-n", "--", "yes"]));
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
    std::thread::sleep(Duration::from_millis(500));
    signal(&child, libc::SIGTERM);
    let started = std::time::Instant::now();
    let mut child = child;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        assert!(started.elapsed() < WAIT, "exec did not exit on SIGTERM");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(status.code(), Some(143));
    broker.wait_for(WAIT, |c| c[0].client_gone);
}

/// The fake broker reads stdin only as fast as `cat` echoes it, so the client
/// must keep reading output while its stdin sends wait.
#[tokio::test]
async fn exec_cat_of_64_mib_completes_while_the_broker_holds_stdin_back() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Echo, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let input = binary(64 * 1024 * 1024);
    let child = run
        .command(&exec_args(&["--", "cat"]))
        .stdin(file_with(&run, &input))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let out = wait_with_output(child, Duration::from_secs(120));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(out.stdout == input, "the echo differs from stdin");
}

/// Waits for a child while reading its output, killing it on expiry.
fn wait_with_output(mut child: std::process::Child, timeout: Duration) -> std::process::Output {
    let out = collect(child.stdout.take().unwrap());
    let err = collect(child.stderr.take().unwrap());
    let deadline = std::time::Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if std::time::Instant::now() > deadline {
            let _ = child.kill();
            panic!(
                "hedra-cli did not exit within {timeout:?}; stderr: {}",
                String::from_utf8_lossy(&err.lock().unwrap())
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    // The readers finish once the pipes close.
    std::thread::sleep(Duration::from_millis(200));
    let stdout = out.lock().unwrap().clone();
    let stderr = err.lock().unwrap().clone();
    std::process::Output {
        status,
        stdout,
        stderr,
    }
}

#[tokio::test]
async fn signals_end_exec_while_the_broker_has_stopped_reading_stdin() {
    for (sig, code) in [(libc::SIGINT, 130), (libc::SIGTERM, 143)] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let child = run
            .command(&exec_args(&["--", "cat"]))
            .stdin(file_with(&run, &binary(64 * 1024 * 1024)))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
        // Long enough for the stdin sends to fill every window and wait.
        std::thread::sleep(Duration::from_millis(1500));
        signal(&child, sig);
        let out = wait(child, Duration::from_secs(5));
        assert_eq!(out.status.code(), Some(code), "{}", stderr(&out));
        broker.wait_for(WAIT, |c| c[0].client_gone);
    }
}

#[tokio::test]
async fn a_closed_stdout_pipe_ends_exec_by_sigpipe_and_cancels_the_call() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::StdoutForever(b"y\n".to_vec(), Duration::from_millis(10)),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    // exec … | head -c1
    let mut child = run.spawn(&exec_args(&["-n", "--", "yes"]));
    let mut first = [0u8; 1];
    child
        .stdout
        .as_mut()
        .unwrap()
        .read_exact(&mut first)
        .unwrap();
    drop(child.stdout.take());
    let out = wait(child, WAIT);
    assert_eq!(out.status.signal(), Some(libc::SIGPIPE), "{}", stderr(&out));
    assert!(stderr(&out).is_empty(), "{}", stderr(&out));
    broker.wait_for(WAIT, |c| c[0].client_gone);
}

#[tokio::test]
async fn a_stream_lost_after_start_is_connection_lost_and_exits_255() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"partial\n".to_vec()),
        Step::Sleep(Duration::from_millis(100)),
        Step::Close,
    ]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "train"]));
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert_eq!(stdout(&out), "partial\n");
    let err = stderr(&out);
    let line = class_line(&err).unwrap_or_else(|| panic!("no class line: {err}"));
    assert!(
        line.starts_with("hedra-cli: Connection lost (18): Lost the connection to the sandbox"),
        "{line}"
    );
    assert!(line.contains("The sandbox is still running."), "{line}");
    // A lost stream is never replayed.
    assert_eq!(broker.calls().len(), 1);
}

#[tokio::test]
async fn statuses_after_start_without_a_table_reason_are_connection_lost_with_the_brokers_message()
{
    for status in [
        broker_status(
            Code::Unavailable,
            "rate_limited",
            "client fell behind the sandbox's output; reconnect",
            &[],
        ),
        disconnected(),
        Status::internal("stream reset"),
    ] {
        let message = status.message().to_string();
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Fail(status)]));
        let server = running_sandbox(&broker).await;
        let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
        assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
        let err = stderr(&out);
        let line = class_line(&err).unwrap_or_default();
        assert!(
            line.starts_with(&format!(
                "hedra-cli: Connection lost (18): Lost the connection to the sandbox ({message})."
            )),
            "{line}"
        );
    }
}

#[tokio::test]
async fn each_broker_reason_maps_to_its_class_line_and_exec_exits_255() {
    let cases = [
        (
            Code::PermissionDenied,
            "permission_denied",
            "Missing scope (10): This key lacks `sandboxes:exec`.",
        ),
        (Code::ResourceExhausted, "quota_exceeded", "Limit (16): "),
        (Code::Unavailable, "rate_limited", "Limit (16): "),
        (Code::NotFound, "not_found", "Not ready or stopped (13): "),
        (Code::Internal, "internal", "Internal (1): "),
        (
            Code::InvalidArgument,
            "invalid_argument",
            "Rejected request (19): ",
        ),
    ];
    for (code, reason, expected) in cases {
        let broker = FakeBroker::start();
        // The real broker names the scope only in its message.
        let message = if reason == "permission_denied" {
            "token lacks the sandbox:exec scope"
        } else {
            "the broker says no"
        };
        broker.push_exec(ExecScript::refuse(broker_status(
            code,
            reason,
            message,
            &[],
        )));
        let server = running_sandbox(&broker).await;
        let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
        assert_eq!(out.status.code(), Some(255), "{reason}: {}", stderr(&out));
        let err = stderr(&out);
        let line = class_line(&err).unwrap_or_default();
        assert!(
            line.starts_with(&format!("hedra-cli: {expected}")),
            "{reason}: {line}"
        );
        assert!(!line.contains("`sandbox:exec`"), "{line}");
        // A known reason is not retried.
        assert_eq!(broker.calls().len(), 1, "{reason}");
    }
}

#[tokio::test]
async fn invalid_argument_after_admission_is_rejected_request() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Fail(broker_status(
            Code::InvalidArgument,
            "invalid_argument",
            "the first frame must be start",
            &[],
        )),
    ]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert_eq!(
        class_line(&stderr(&out)),
        Some("hedra-cli: Rejected request (19): the first frame must be start")
    );
}

/// The full 60 s retry before admission, then Not ready or stopped.
#[tokio::test]
async fn sandbox_disconnected_past_the_retry_is_not_ready() {
    let broker = FakeBroker::start();
    for _ in 0..40 {
        broker.push_exec(ExecScript::refuse(disconnected()));
    }
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    let err = stderr(&out);
    assert!(
        class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Not ready or stopped (13): ")),
        "{err}"
    );
    assert!(broker.calls().len() > 2);
}

#[tokio::test]
async fn unauthenticated_mints_once_more_and_a_second_is_unavailable() {
    let unauthenticated = || {
        broker_status(
            Code::Unauthenticated,
            "unauthenticated",
            "unknown signing key",
            &[],
        )
    };
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(unauthenticated()));
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(mints(&server, ID).await, 2);

    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(unauthenticated()));
    broker.push_exec(ExecScript::refuse(unauthenticated()));
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert!(
        class_line(&stderr(&out)).is_some_and(|l| l.starts_with("hedra-cli: Unavailable (17): ")),
        "{}",
        stderr(&out)
    );
    assert_eq!(mints(&server, ID).await, 2);
    assert_eq!(broker.calls().len(), 2);
}

#[tokio::test]
async fn a_brokers_message_is_printed_with_its_controls_escaped() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(broker_status(
        Code::Internal,
        "internal",
        "audit write failed \x1b]52;c;aGk=\x07 done",
        &[],
    )));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
    let err = stderr(&out);
    assert!(!err.contains('\x1b'), "{err:?}");
    assert!(!err.contains('\x07'), "{err:?}");
    assert!(err.contains(r"\u{1b}]52;c;aGk=\u{7} done"), "{err}");
}

// ---------------------------------------------------------------- session

#[tokio::test]
async fn a_mint_answered_503_is_retried() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{ID}/session")))
        .respond_with(Sequence::new(vec![
            envelope(
                503,
                "UNAVAILABLE",
                "Sandbox sessions are starting.",
                Some(0),
            ),
            session_answer(&broker),
        ]))
        .mount(&server)
        .await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(mints(&server, ID).await, 2);
    assert_eq!(broker.calls().len(), 1);
}

/// The verbs' client runs with the executor's retries off: a lookup, which
/// the verb attempts once, is sent once however the route answers.
#[tokio::test]
async fn a_name_lookup_answered_503_is_sent_once() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/sandboxes/box"))
        .respond_with(envelope(503, "UNAVAILABLE", "starting", Some(0)))
        .mount(&server)
        .await;
    let out = Run::new(&server).run(&["sandbox", "exec", "box", "-n", "--", "true"]);
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert_eq!(
        requests_to(&server, "GET", "/v3/sandboxes/box").await.len(),
        1
    );
}

/// A lookup's 404s exit with the class codes the help documents.
#[tokio::test]
async fn a_lookups_404s_exit_not_found_and_not_enabled() {
    for (message, code, line) in [
        (
            "Sandbox not found.",
            11,
            "hedra-cli: Not found (11): No sandbox `box`. Run `hedra-cli sandbox ls`.",
        ),
        (
            "Sandboxes are not available.",
            12,
            "hedra-cli: Not enabled (12): Sandboxes are not available to this workspace yet. \
             Ask Hedra to enable them.",
        ),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v3/sandboxes/box"))
            .respond_with(envelope(404, "NOT_FOUND", message, None))
            .mount(&server)
            .await;
        let out = Run::new(&server).run(&["sandbox", "logs", "box"]);
        assert_eq!(out.status.code(), Some(code), "{}", stderr(&out));
        assert_eq!(class_line(&stderr(&out)), Some(line));
    }
}

/// A broker that closes the connection after reading the request has not
/// admitted the call, so the open is retried.
#[tokio::test]
async fn a_connection_dropped_before_admission_is_retried() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::drop_connection());
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"second".to_vec()),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&exec_args(&["-n", "--", "x"]));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "second");
    assert_eq!(broker.calls().len(), 2);
}

// ---------------------------------------------------------------- transport

fn write_dotenv(run: &Run, proxy: &Proxy) {
    std::fs::write(
        run.home.path().join(".env"),
        format!(
            "https_proxy={0}\nHTTPS_PROXY={0}\nHEDRA_CLI_INSECURE=1\n",
            proxy.url()
        ),
    )
    .unwrap();
}

#[tokio::test]
async fn a_dotenv_cannot_set_the_proxy_or_turn_off_verification() {
    let proxy = Proxy::start();
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    write_dotenv(&run, &proxy);

    // Without the CA bundle, the fake broker's certificate is not trusted.
    let child = run
        .command(&exec_args(&["-n", "--", "x"]))
        .env_remove("HEDRA_CLI_CA_BUNDLE")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    wait_until("a refused handshake", || broker.failed_handshakes() >= 1);
    signal(&child, libc::SIGKILL);
    let _ = wait(child, WAIT);
    assert!(broker.calls().is_empty());

    // With it, the call goes straight to the broker.
    let out = run.run(&exec_args(&["-n", "--", "x"]));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(proxy.requests().is_empty(), "{:?}", proxy.requests());
}

/// An untrusted broker certificate is not retried for the admission budget:
/// the open fails at once and names the variables that add a CA.
#[tokio::test]
async fn an_untrusted_broker_certificate_fails_at_once_naming_the_fix() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let started = std::time::Instant::now();
    let out = Run::new(&server)
        .command(&exec_args(&["-n", "--", "x"]))
        .env_remove("HEDRA_CLI_CA_BUNDLE")
        .output()
        .unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    let err = stderr(&out);
    let line = class_line(&err).unwrap_or_default();
    assert!(
        line.starts_with(
            "hedra-cli: Rejected request (19): Could not connect to the sandbox broker: \
             the broker's TLS certificate is not trusted"
        ),
        "{err}"
    );
    assert!(line.contains("HEDRA_CLI_CA_BUNDLE"), "{err}");
    assert_eq!(broker.failed_handshakes(), 1);
}

/// A proxy answering the CONNECT with 407 fails the open at once, with the
/// proxy's answer.
#[tokio::test]
async fn a_proxy_refusing_the_tunnel_fails_at_once_with_its_answer() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let tunnels = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = tunnels.clone();
    std::thread::spawn(move || {
        for conn in listener.incoming() {
            let Ok(mut client) = conn else { continue };
            counted.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let mut head = Vec::new();
            let mut b = [0u8; 1];
            while !head.ends_with(b"\r\n\r\n") {
                if client.read(&mut b).unwrap_or(0) == 0 {
                    break;
                }
                head.push(b[0]);
            }
            let _ = client.write_all(b"HTTP/1.1 407 Proxy Authentication Required\r\n\r\n");
        }
    });
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let started = std::time::Instant::now();
    let out = Run::new(&server)
        .command(&exec_args(&["-n", "--", "x"]))
        .env("HTTPS_PROXY", format!("http://127.0.0.1:{port}"))
        .output()
        .unwrap();
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "{:?}",
        started.elapsed()
    );
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert_eq!(
        class_line(&stderr(&out)),
        Some(
            "hedra-cli: Rejected request (19): Could not connect to the sandbox broker: the \
             proxy refused the tunnel to the broker: HTTP/1.1 407 Proxy Authentication Required."
        )
    );
    assert_eq!(tunnels.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(broker.connections(), 0);
}

/// A proxy answering the CONNECT 503 cannot reach the broker just now: the
/// open retries through it, as it does an unreachable broker, and succeeds
/// once the proxy relays again.
#[tokio::test]
async fn a_proxy_answering_503_is_retried_until_it_relays() {
    let proxy = Proxy::start();
    proxy.unavailable_for(2);
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"through".to_vec()),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server)
        .command(&exec_args(&["-n", "--", "x"]))
        .env("HTTPS_PROXY", proxy.url())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "through");
    assert_eq!(proxy.requests().len(), 3, "{:?}", proxy.requests());
    assert!(
        stderr(&out).contains("Could not reach the sandbox broker through the proxy"),
        "{}",
        stderr(&out)
    );
}

/// A malformed `HEDRA_CLI_PROXY` fails before any request, naming the
/// variable and never its value, which may hold a password.
#[tokio::test]
async fn a_malformed_hedra_cli_proxy_is_refused_without_its_password() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    for (verb, code) in [
        (&["sandbox", "exec", ID, "-n", "--", "x"][..], 255),
        (&["sandbox", "logs", ID][..], 19),
    ] {
        let out = Run::new(&server)
            .command(verb)
            .env("HEDRA_CLI_PROXY", "http://user:s3cret@127.0.0.1:notaport")
            .output()
            .unwrap();
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(code), "{verb:?}: {err}");
        assert!(!err.contains("s3cret"), "{err}");
        assert!(!err.contains("panicked"), "{err}");
        let line = class_line(&err).unwrap_or_default();
        assert!(
            line.starts_with("hedra-cli: Rejected request (19): "),
            "{err}"
        );
        assert!(line.contains("HEDRA_CLI_PROXY"), "{err}");
    }
    assert!(server.received_requests().await.unwrap().is_empty());
    assert_eq!(broker.connections(), 0);
}

#[tokio::test]
async fn hedra_cli_insecure_from_the_environment_accepts_the_broker() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server)
        .command(&exec_args(&["-n", "--", "x"]))
        .env_remove("HEDRA_CLI_CA_BUNDLE")
        .env("HEDRA_CLI_INSECURE", "1")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(broker.failed_handshakes(), 0);
}

#[tokio::test]
async fn the_broker_is_reached_through_a_connect_tunnel() {
    for var in ["HTTPS_PROXY", "https_proxy"] {
        let proxy = Proxy::start();
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([
            Step::Start,
            Step::Stdout(b"tunnelled".to_vec()),
            Step::Exit(0),
        ]));
        let server = running_sandbox(&broker).await;
        let out = Run::new(&server)
            .command(&exec_args(&["-n", "--", "x"]))
            .env(var, proxy.url())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "{var}: {}", stderr(&out));
        assert_eq!(stdout(&out), "tunnelled");
        assert_eq!(
            proxy.requests(),
            [format!("CONNECT 127.0.0.1:{} HTTP/1.1", broker.port())],
            "{var}"
        );
    }
}

#[tokio::test]
async fn a_socks_proxy_is_refused_before_any_request() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server)
        .command(&exec_args(&["-n", "--", "x"]))
        .env("HTTPS_PROXY", "socks5://127.0.0.1:1080")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert_eq!(
        class_line(&stderr(&out)),
        Some(
            "hedra-cli: Rejected request (19): a SOCKS proxy is not supported for sandbox \
             sessions; use an http:// proxy (set by HTTPS_PROXY)"
        )
    );
    assert!(server.received_requests().await.unwrap().is_empty());
    assert_eq!(broker.connections(), 0);
}

// ---------------------------------------------------------------- logs

fn ended() -> serde_json::Value {
    let mut s = sandbox(
        ID,
        "box",
        "ended",
        json!([{"replica": 0, "status": "stopped"}]),
    );
    s["end_reason"] = json!("exited");
    s
}

async fn mount_stored(server: &MockServer, body: &str) {
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}/logs")))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"content": body, "truncated": false})),
        )
        .mount(server)
        .await;
}

async fn stored_reads(server: &MockServer) -> Vec<wiremock::Request> {
    requests_to(server, "GET", &format!("/v3/sandboxes/{ID}/logs")).await
}

#[tokio::test]
async fn logs_reads_the_broker_while_running_and_the_stored_copy_once_stopped() {
    let broker = FakeBroker::start();
    broker.log(b"one\n");
    broker.log(b"two\n");
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "stored\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "one\ntwo\n");
    let request = broker.calls()[0].logs.unwrap();
    assert!(!request.follow);
    assert_eq!(request.since, None);
    assert!(stored_reads(&server).await.is_empty());

    let broker = FakeBroker::start();
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    mount_sandbox(&server, ID, ended()).await;
    mount_stored(&server, "stored\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "stored\n");
    assert!(broker.calls().is_empty());
    assert_eq!(mints(&server, ID).await, 0);
}

#[tokio::test]
async fn logs_falls_back_to_the_stored_copy_on_sandbox_disconnected() {
    let broker = FakeBroker::start();
    broker.push_logs(LogsScript::refuse(disconnected()));
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "stored\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "stored\n");
}

/// `logs -f` follows replica 0: replica 1 failing while the sandbox starts
/// does not end the follow.
#[tokio::test]
async fn logs_f_follows_replica_zero_when_another_replica_fails() {
    let broker = FakeBroker::start();
    broker.log(b"zero\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    let mut started = sandbox(ID, "box", "creating", json!(null));
    started["replicas"] = json!(2);
    let mut split = sandbox(
        ID,
        "box",
        "active",
        json!([
            {"replica": 0, "status": "running"},
            {"replica": 1, "status": "failed", "failure_reason": "instance_failed"},
        ]),
    );
    split["replicas"] = json!(2);
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}")))
        .respond_with(Sequence::new(vec![
            ResponseTemplate::new(200).set_body_json(started),
            ResponseTemplate::new(200).set_body_json(split),
        ]))
        .mount(&server)
        .await;
    let mut child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    let seen = collect(child.stdout.take().unwrap());
    wait_until("replica 0's output", || {
        seen.lock().unwrap().ends_with(b"zero\n")
    });
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(broker.calls().len(), 1);
}

/// A wait that failed while the sandbox started, re-read once replica 0 is
/// running, goes on to follow it instead of ending.
#[tokio::test]
async fn logs_f_follows_once_replica_zero_runs_after_a_failed_wait() {
    let broker = FakeBroker::start();
    broker.log(b"zero\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    let at = |status: &str, instances: serde_json::Value| {
        ResponseTemplate::new(200).set_body_json(sandbox(ID, "box", status, instances))
    };
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}")))
        .respond_with(Sequence::new(vec![
            at("creating", json!(null)),
            at(
                "active",
                json!([{"replica": 0, "status": "failed", "failure_reason": "instance_failed"}]),
            ),
            at("active", running()),
        ]))
        .mount(&server)
        .await;
    let mut child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    let seen = collect(child.stdout.take().unwrap());
    wait_until("replica 0's output", || {
        seen.lock().unwrap().ends_with(b"zero\n")
    });
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(broker.calls().len(), 1);
}

#[tokio::test]
async fn logs_f_resumes_a_nanosecond_after_the_last_chunk_on_one_mint() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    let b_at = broker.log(b"b\n");
    broker.push_logs(
        LogsScript::admit(LogsEnd::Fail(Status::unavailable("broker restarting")))
            .then_append(b"c\n"),
    );
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    let server = running_sandbox(&broker).await;
    let mut child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    let seen = collect(child.stdout.take().unwrap());
    wait_until("the resumed chunk", || {
        seen.lock().unwrap().ends_with(b"c\n")
    });
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(String::from_utf8_lossy(&seen.lock().unwrap()), "a\nb\nc\n");

    let calls = broker.calls();
    assert_eq!(calls.len(), 2);
    let resumed = calls[1].logs.unwrap();
    assert!(resumed.follow);
    let since = resumed.since.unwrap();
    assert_eq!((since.seconds, since.nanos), (b_at.seconds, b_at.nanos + 1));
    // One mint serves both streams.
    assert_eq!(mints(&server, ID).await, 1);
    assert!(calls
        .iter()
        .all(|c| c.authorization.as_deref() == Some("Bearer jwt.session.one")));
    assert!(calls[1].client_gone);
}

#[tokio::test]
async fn logs_f_prints_the_received_chunks_then_the_stored_tail_when_the_sandbox_stops() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.log(b"b\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(disconnected())));
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}")))
        .respond_with(Sequence::new(vec![
            ResponseTemplate::new(200).set_body_json(sandbox(ID, "box", "active", running())),
            ResponseTemplate::new(200).set_body_json(ended()),
        ]))
        .mount(&server)
        .await;
    mount_stored(&server, "c\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", "-f", ID]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "a\nb\nc\n");
    let reads = stored_reads(&server).await;
    assert_eq!(reads.len(), 1);
    let since = reads[0]
        .url
        .query_pairs()
        .find(|(k, _)| k == "since")
        .map(|(_, v)| v.into_owned());
    assert_eq!(since.as_deref(), Some("2026-11-04T00:00:01.500000002Z"));
    assert!(
        stderr(&out).contains("the rest is from the stored copy"),
        "{}",
        stderr(&out)
    );
}

#[tokio::test]
async fn logs_and_logs_f_of_a_failed_sandbox_read_no_stored_copy() {
    for follow in [false, true] {
        let broker = FakeBroker::start();
        let server = MockServer::start().await;
        mount_session(&server, ID, &broker).await;
        mount_sandbox(&server, ID, sandbox(ID, "box", "failed", json!(null))).await;
        mount_stored(&server, "never\n").await;
        let args: &[&str] = if follow {
            &["sandbox", "logs", "-f", ID]
        } else {
            &["sandbox", "logs", ID]
        };
        let out = Run::new(&server).run(args);
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
        assert!(stderr(&out).contains("never started"), "{}", stderr(&out));
        assert!(stored_reads(&server).await.is_empty());
        assert_eq!(mints(&server, ID).await, 0);
    }
}

/// A ring buffer far larger than a pipe holds, so the writer soon waits on a
/// stdout nobody reads.
fn fill_ring(broker: &FakeBroker) {
    for _ in 0..64 {
        broker.log(&[b'z'; 64 * 1024]);
    }
}

#[tokio::test]
async fn signals_end_logs_while_stdout_is_stalled() {
    let cases: [(&[&str], i32, i32); 5] = [
        (&["logs", "-f"], libc::SIGTERM, 143),
        (&["logs", "-f"], libc::SIGHUP, 129),
        (&["logs", "-f"], libc::SIGINT, 0),
        (&["attach"], libc::SIGINT, 0),
        (&["logs"], libc::SIGINT, 130),
    ];
    for (verb, sig, code) in cases {
        let broker = FakeBroker::start();
        fill_ring(&broker);
        broker.push_logs(LogsScript::admit(LogsEnd::Default));
        let server = running_sandbox(&broker).await;
        let mut args = vec!["sandbox"];
        args.extend_from_slice(verb);
        args.push(ID);
        // stdout is a pipe nobody reads.
        let child = Run::new(&server).spawn(&args);
        broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.admitted));
        std::thread::sleep(Duration::from_millis(1000));
        signal(&child, sig);
        let out = wait(child, Duration::from_secs(5));
        assert_eq!(out.status.code(), Some(code), "{verb:?}: {}", stderr(&out));
    }
}

/// The first open of `logs -f` gives up after its 60 s retry before
/// admission, by the class of the last refusal.
async fn logs_f_first_open_gives_up_as(status: Status, class: &str, code: i32) {
    let broker = FakeBroker::start();
    for _ in 0..60 {
        broker.push_logs(LogsScript::refuse(status.clone()));
    }
    let server = running_sandbox(&broker).await;
    let child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    let out = wait(child, Duration::from_secs(90));
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(code), "{class}: {err}");
    assert!(
        class_line(&err).is_some_and(|l| l.starts_with(&format!("hedra-cli: {class}: "))),
        "{err}"
    );
    assert!(broker.calls().len() > 2, "{class}: not retried");
}

#[tokio::test]
async fn logs_f_first_open_refused_sandbox_disconnected_is_not_ready() {
    logs_f_first_open_gives_up_as(disconnected(), "Not ready or stopped (13)", 13).await;
}

#[tokio::test]
async fn logs_f_first_open_refused_unavailable_with_an_unknown_reason_is_unavailable() {
    let status = broker_status(
        Code::Unavailable,
        "a_new_reason",
        "unknown to this client",
        &[],
    );
    logs_f_first_open_gives_up_as(status, "Unavailable (17)", 17).await;
}

#[tokio::test]
async fn logs_f_with_data_loss_before_admission_is_internal() {
    let broker = FakeBroker::start();
    broker.push_logs(LogsScript::refuse(Status::data_loss("lost")));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&["sandbox", "logs", "-f", ID]);
    assert_eq!(out.status.code(), Some(1), "{}", stderr(&out));
    assert!(
        class_line(&stderr(&out)).is_some_and(|l| l.starts_with("hedra-cli: Internal (1): ")),
        "{}",
        stderr(&out)
    );
    assert_eq!(broker.calls().len(), 1);
}

#[tokio::test]
async fn a_logs_f_reopen_refused_before_admission_says_what_it_waits_on() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(Status::unavailable(
        "broker restarting",
    ))));
    broker.push_logs(LogsScript::refuse(disconnected()));
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    let server = running_sandbox(&broker).await;
    let child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    broker.wait_for(WAIT, |c| c.len() == 3 && c[2].admitted);
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(
        stderr(&out).contains("supervisor is not connected yet; retrying."),
        "{}",
        stderr(&out)
    );
}

#[tokio::test]
async fn ctrl_c_ends_logs_f_and_attach_with_0_and_cancels_the_call() {
    for verb in [&["logs", "-f"][..], &["attach"][..]] {
        let broker = FakeBroker::start();
        broker.log(b"x\n");
        broker.push_logs(LogsScript::admit(LogsEnd::Hold));
        let server = running_sandbox(&broker).await;
        let mut args = vec!["sandbox"];
        args.extend_from_slice(verb);
        args.push(ID);
        let child = Run::new(&server).spawn(&args);
        broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.admitted));
        signal(&child, libc::SIGINT);
        let out = wait(child, WAIT);
        assert_eq!(out.status.code(), Some(0), "{verb:?}: {}", stderr(&out));
        broker.wait_for(WAIT, |c| c[0].client_gone);
    }
}

#[tokio::test]
async fn logs_f_rides_out_a_failed_read_between_reopens() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(
        LogsScript::admit(LogsEnd::Fail(Status::unavailable("broker restarting")))
            .then_append(b"b\n"),
    );
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    let active =
        || ResponseTemplate::new(200).set_body_json(sandbox(ID, "box", "active", running()));
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}")))
        .respond_with(Sequence::new(vec![
            active(),
            envelope(503, "UNAVAILABLE", "Try again.", None),
            active(),
        ]))
        .mount(&server)
        .await;
    let mut child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    let seen = collect(child.stdout.take().unwrap());
    wait_until("the chunk after the reopen", || {
        seen.lock().unwrap().ends_with(b"b\n")
    });
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(String::from_utf8_lossy(&seen.lock().unwrap()), "a\nb\n");
    assert_eq!(broker.calls().len(), 2);
}

#[tokio::test]
async fn logs_f_still_ends_on_a_first_read_refused() {
    let broker = FakeBroker::start();
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}")))
        .respond_with(envelope(503, "UNAVAILABLE", "Try again.", None))
        .mount(&server)
        .await;
    let child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(17), "{}", stderr(&out));
    assert!(broker.calls().is_empty());
}

#[tokio::test]
async fn plain_logs_losing_its_stream_says_so_with_the_sandboxs_state() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(Status::unavailable(
        "broker restarting",
    ))));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    assert_eq!(out.status.code(), Some(18), "{}", stderr(&out));
    assert_eq!(
        class_line(&stderr(&out)),
        Some(
            "hedra-cli: Connection lost (18): Lost the connection to the sandbox (broker \
             restarting). The sandbox is still running."
        ),
        "{}",
        stderr(&out)
    );
}

/// A pipe already full, so the next write to it blocks.
fn full_pipe() -> (std::fs::File, std::fs::File) {
    use std::os::fd::FromRawFd;
    let mut fds = [0; 2];
    // SAFETY: pipe fills two descriptors this test then owns.
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    // SAFETY: both are valid and owned from here on.
    let (read, write) = unsafe {
        (
            std::fs::File::from_raw_fd(fds[0]),
            std::fs::File::from_raw_fd(fds[1]),
        )
    };
    // SAFETY: flag changes on a descriptor this test owns.
    unsafe {
        let flags = libc::fcntl(fds[1], libc::F_GETFL);
        libc::fcntl(fds[1], libc::F_SETFL, flags | libc::O_NONBLOCK);
        while libc::write(fds[1], [0u8; 4096].as_ptr().cast(), 4096) > 0 {}
        // Blocking again, as a stderr nobody reads is.
        libc::fcntl(fds[1], libc::F_SETFL, flags);
    }
    (read, write)
}

#[tokio::test]
async fn sigterm_ends_logs_f_while_its_own_stderr_lines_are_stalled() {
    let broker = FakeBroker::start();
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    // A starting sandbox: the wait writes its status to stderr.
    mount_sandbox(&server, ID, sandbox(ID, "box", "creating", json!([]))).await;
    let (_read, write) = full_pipe();
    let child = Run::new(&server)
        .command(&["sandbox", "logs", "-f", ID])
        .stdout(Stdio::null())
        .stderr(Stdio::from(write))
        .spawn()
        .unwrap();
    while requests_to(&server, "GET", &format!("/v3/sandboxes/{ID}"))
        .await
        .is_empty()
    {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    std::thread::sleep(Duration::from_millis(500));
    signal(&child, libc::SIGTERM);
    let out = wait(child, Duration::from_secs(5));
    assert_eq!(out.status.code(), Some(143));
}

/// A session answer whose `Date` puts the server `skew` seconds from this
/// clock, with a token living `life` seconds from that `Date`.
fn session_dated(broker: &FakeBroker, skew: i64, life: i64) -> ResponseTemplate {
    use hedra_cli_sdk::api::prelude::{DateTime, Utc};
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let server_now = now + skew;
    let date = std::time::UNIX_EPOCH + Duration::from_secs(server_now as u64);
    let expires = DateTime::<Utc>::from_timestamp(server_now + life, 0).unwrap();
    ResponseTemplate::new(200)
        .insert_header("date", httpdate::fmt_http_date(date).as_str())
        .set_body_json(json!({
            "token": TOKEN, "broker_url": broker.broker_url(),
            "expires_at": expires.to_rfc3339(), "replica": 0,
        }))
}

#[tokio::test]
async fn a_tokens_life_is_measured_from_the_mint_answers_date() {
    // (server clock skew, life from its Date, mints for two streams)
    for (skew, life, want) in [
        // Behind: this clock sees the token expired, the Date does not.
        (-3_600, 300, 1),
        // Ahead: this clock sees an hour left, the Date under a minute.
        (3_600, 30, 2),
    ] {
        let broker = FakeBroker::start();
        broker.log(b"a\n");
        broker.push_logs(
            LogsScript::admit(LogsEnd::Fail(Status::unavailable("broker restarting")))
                .then_append(b"b\n"),
        );
        broker.push_logs(LogsScript::admit(LogsEnd::Hold));
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(format!("/v3/sandboxes/{ID}/session")))
            .respond_with(session_dated(&broker, skew, life))
            .mount(&server)
            .await;
        mount_sandbox(&server, ID, sandbox(ID, "box", "active", running())).await;
        let mut child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
        let seen = collect(child.stdout.take().unwrap());
        wait_until("the chunk after the reopen", || {
            seen.lock().unwrap().ends_with(b"b\n")
        });
        signal(&child, libc::SIGINT);
        let out = wait(child, WAIT);
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
        assert_eq!(mints(&server, ID).await, want, "skew {skew}");
    }
}

// ---------------------------------------------------------------- transport

#[tokio::test]
async fn a_reset_broker_connection_is_connection_lost_not_sigpipe() {
    let proxy = Proxy::start();
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Count]));
    let server = running_sandbox(&broker).await;
    let mut child = Run::new(&server)
        .command(&exec_args(&["--", "wc", "-c"]))
        .env("HTTPS_PROXY", proxy.url())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // stdin that never stops, so the client keeps writing to the socket.
    let mut stdin = child.stdin.take().unwrap();
    std::thread::spawn(move || while stdin.write_all(&[b's'; 32 * 1024]).is_ok() {});
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.stdin.len() > 1 << 20));
    proxy.reset();
    let out = wait(child, WAIT);
    assert_eq!(out.status.signal(), None, "{}", stderr(&out));
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert!(
        class_line(&stderr(&out))
            .is_some_and(|l| l.starts_with("hedra-cli: Connection lost (18): ")),
        "{}",
        stderr(&out)
    );
}

#[tokio::test]
async fn a_dead_network_ends_the_stream_by_keepalive_within_about_40_s() {
    let proxy = Proxy::start();
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let server = running_sandbox(&broker).await;
    let child = Run::new(&server)
        .command(&exec_args(&["-n", "--", "sleep", "inf"]))
        .env("HTTPS_PROXY", proxy.url())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.stdin_eof));
    proxy.freeze();
    let frozen = std::time::Instant::now();
    let out = wait(child, Duration::from_secs(60));
    let took = frozen.elapsed();
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert!(
        class_line(&stderr(&out))
            .is_some_and(|l| l.starts_with("hedra-cli: Connection lost (18): ")),
        "{}",
        stderr(&out)
    );
    assert!(took < Duration::from_secs(45), "took {took:?}");
}

/// Output files whose writes fail other than `EPIPE`: a descriptor open only
/// for reading everywhere, and a full disk where the platform has one.
fn unwritable(run: &Run) -> Vec<(&'static str, std::fs::File)> {
    let p = run.home.path().join("read-only");
    std::fs::write(&p, b"").unwrap();
    let mut files = vec![("a read-only descriptor", std::fs::File::open(&p).unwrap())];
    if let Ok(full) = std::fs::OpenOptions::new().write(true).open("/dev/full") {
        files.push(("a full disk", full));
    }
    files
}

#[tokio::test]
async fn logs_f_into_an_unwritable_output_cancels_its_call_and_exits_255() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    for (what, file) in unwritable(&run) {
        let calls = broker.calls().len();
        broker.log(b"some output\n");
        broker.push_logs(LogsScript::admit(LogsEnd::Hold));
        let child = run
            .command(&["sandbox", "logs", "-f", ID])
            .stdout(file)
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let out = wait(child, WAIT);
        assert_eq!(out.status.code(), Some(255), "{what}: {}", stderr(&out));
        assert!(
            class_line(&stderr(&out))
                .is_some_and(|l| l
                    .starts_with("hedra-cli: Output failed (255): Could not write the output: ")),
            "{what}: {}",
            stderr(&out)
        );
        broker.wait_for(WAIT, |c| c.len() > calls && c[calls].client_gone);
    }
}

#[tokio::test]
async fn a_slow_reader_slows_an_exec_without_t_instead_of_ending_it() {
    const TOTAL: usize = 16 << 20;
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::StdoutBulk(TOTAL, 32 * 1024),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let mut child = Run::new(&server)
        .command(&exec_args(&["-n", "--", "cat", "big"]))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // A reader taking about 1 MiB a second.
    let mut stdout = child.stdout.take().unwrap();
    let read = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = read.clone();
    let reader = std::thread::spawn(move || {
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            match stdout.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => {
                    counted.fetch_add(n, std::sync::atomic::Ordering::SeqCst);
                    std::thread::sleep(Duration::from_millis(62));
                }
            }
        }
    });
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
    std::thread::sleep(Duration::from_secs(3));
    // The command is held back near the reader, not run to its end.
    let sent = broker.calls()[0].sent;
    assert!(
        sent < TOTAL,
        "the command was not slowed: {sent} bytes sent"
    );
    let out = wait(child, Duration::from_secs(60));
    reader.join().unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(read.load(std::sync::atomic::Ordering::SeqCst), TOTAL);
    let call = &broker.calls()[0];
    assert!(call.finished && !call.client_gone, "{call:?}");
}

#[tokio::test]
async fn debug_on_exec_shows_the_broker_and_the_admission_and_never_the_token() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(Status::unavailable("restarting")));
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&["sandbox", "exec", "--debug", ID, "-n", "--", "true"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert!(
        err.contains(&format!("> exec https://{}", broker.broker_url())),
        "{err}"
    );
    assert!(err.contains("< exec refused: code Unavailable"), "{err}");
    assert!(err.contains("< exec admitted"), "{err}");
    assert!(err.contains("authorization: Bearer [REDACTED]"), "{err}");
    assert!(!err.contains(TOKEN), "{err}");
    assert!(!err.contains("jwt."), "{err}");
}

#[tokio::test]
async fn a_panic_in_the_logs_writer_exits_255_with_its_message() {
    let broker = FakeBroker::start();
    broker.log(b"some output\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    let server = running_sandbox(&broker).await;
    let child = Run::new(&server)
        .command(&["sandbox", "logs", "-f", ID])
        .env("HEDRA_CLI_TEST_PANIC", "task")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert!(
        class_line(&stderr(&out)).is_some_and(
            |l| l.starts_with("hedra-cli: Panic (255): ") && l.contains("injected panic")
        ),
        "{}",
        stderr(&out)
    );
}

// ---------------------------------------------------------------- O_NONBLOCK

/// A pipe whose read end (`read_side`) or write end is `O_NONBLOCK`, as a
/// process sharing the descriptor may leave it.
fn nonblocking_pipe(read_side: bool) -> (std::fs::File, std::fs::File) {
    use std::os::fd::FromRawFd;
    let mut fds = [0; 2];
    // SAFETY: pipe fills two descriptors this test then owns.
    assert_eq!(unsafe { libc::pipe(fds.as_mut_ptr()) }, 0);
    let fd = if read_side { fds[0] } else { fds[1] };
    // SAFETY: flag changes on descriptors this test owns; neither is
    // inherited past the one a child is given.
    unsafe {
        libc::fcntl(fds[0], libc::F_SETFD, libc::FD_CLOEXEC);
        libc::fcntl(fds[1], libc::F_SETFD, libc::FD_CLOEXEC);
        let flags = libc::fcntl(fd, libc::F_GETFL);
        libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
    }
    // SAFETY: both are valid and owned from here on.
    unsafe {
        (
            std::fs::File::from_raw_fd(fds[0]),
            std::fs::File::from_raw_fd(fds[1]),
        )
    }
}

#[tokio::test]
async fn a_nonblocking_stdin_is_waited_on_rather_than_read_as_its_end() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Count,
        Step::PrintCount,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let (read, mut write) = nonblocking_pipe(true);
    let child = Run::new(&server)
        .command(&exec_args(&["--", "wc", "-c"]))
        .stdin(read)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::thread::sleep(Duration::from_secs(3));
    write.write_all(b"hello world\n").unwrap();
    drop(write);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(stdout(&out), "12\n");
    let call = &broker.calls()[0];
    assert_eq!(call.stdin, b"hello world\n");
    assert!(call.stdin_eof);
}

#[tokio::test]
async fn a_nonblocking_stdout_read_slowly_gets_every_byte() {
    const TOTAL: usize = 4 << 20;
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::StdoutBulk(TOTAL, 64 * 1024),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let (mut read, write) = nonblocking_pipe(false);
    let child = Run::new(&server)
        .command(&exec_args(&["-n", "--", "cat", "big"]))
        .stdout(write)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let reader = std::thread::spawn(move || {
        let mut buf = vec![0u8; 16 * 1024];
        let mut total = 0;
        // The read end blocks; only the write end is non-blocking.
        while let Ok(n) = read.read(&mut buf) {
            if n == 0 {
                break;
            }
            total += n;
            std::thread::sleep(Duration::from_millis(2));
        }
        total
    });
    let out = wait(child, Duration::from_secs(60));
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(reader.join().unwrap(), TOTAL);
}

#[tokio::test]
async fn a_stdin_that_cannot_be_read_ends_its_input_with_a_line() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::StdinEof,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    // Reading a directory fails with EISDIR.
    let dir = std::fs::File::open(run.home.path()).unwrap();
    let out = run
        .command(&exec_args(&["--", "cat"]))
        .stdin(dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(broker.calls()[0].stdin_eof);
    let err = stderr(&out);
    assert_eq!(err.matches("Could not read stdin (").count(), 1, "{err}");
    assert!(err.contains("the command's input ends here."), "{err}");
}

// ---------------------------------------------------------------- NO_PROXY

#[tokio::test]
async fn a_broker_no_proxy_bypasses_is_not_said_to_be_reached_through_the_proxy() {
    // exec's retry line.
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(Status::unavailable("restarting")));
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server)
        .command(&exec_args(&["-n", "--", "true"]))
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "127.0.0.1")
        .output()
        .unwrap();
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert!(
        err.contains("Could not reach the sandbox broker; retrying."),
        "{err}"
    );
    assert!(!err.contains("proxy"), "{err}");

    // logs' failure line.
    let broker = FakeBroker::start();
    broker.push_logs(LogsScript::refuse(Status::unavailable("restarting")));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server)
        .command(&["sandbox", "logs", ID])
        .env("HTTPS_PROXY", "http://127.0.0.1:9")
        .env("NO_PROXY", "127.0.0.1")
        .output()
        .unwrap();
    let err = stderr(&out);
    assert!(
        class_line(&err).is_some_and(|l| l.contains("per-network connection limit")),
        "{err}"
    );
    assert!(!err.contains("proxy"), "{err}");
}
