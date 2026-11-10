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

/// Text from the `about` of stream verbs, to find them where their names
/// may be common words.
const ABOUTS: [(&str, &str); 3] = [
    ("exec", "relaying its stdin"),
    ("cp", "in or out of a sandbox"),
    ("port-forward", "Forward a local port"),
];

/// The stream verbs are listed and run with no environment variable of their
/// own: `Run` clears the environment, and with only its variables set,
/// `sandbox --help`, the completion scripts and typo suggestions name the
/// verbs beside the generated commands, and `exec` reaches the API.
#[tokio::test]
async fn the_stream_verbs_are_listed_and_run_with_no_environment_variable() {
    let server = MockServer::start().await;
    let run = Run::new(&server);
    let help = stdout(&run.run(&["sandbox", "--help"]));
    for verb in ["exec", "shell", "logs", "attach", "cp", "port-forward"] {
        assert!(
            help.lines()
                .any(|l| l.trim_start().starts_with(&format!("{verb} "))),
            "{verb} is not listed: {help}"
        );
    }

    let script = stdout(&run.run(&["completion", "zsh"]));
    for (verb, about) in ABOUTS {
        assert!(script.contains(about), "zsh does not name {verb}");
    }
    let script = stdout(&run.run(&["completion", "bash"]));
    assert!(
        script.contains("port-forward"),
        "bash does not name port-forward"
    );
    assert!(
        script.contains("gpus"),
        "bash: the generated commands are missing"
    );
    for (typo, verb) in [
        ("exe", "exec"),
        ("cpp", "cp"),
        ("port-forwar", "port-forward"),
    ] {
        let out = run.run(&["sandbox", typo]);
        let text = format!("{}{}", stdout(&out), stderr(&out));
        assert!(
            text.contains(&format!("a similar subcommand exists: '{verb}'")),
            "{typo}: {text}"
        );
    }

    // No mock answers, so `exec` fails on its first request, after sending it.
    let _ = run.run(&["sandbox", "exec", "x", "--", "true"]);
    assert!(!server.received_requests().await.unwrap().is_empty());
}

/// Each stream verb's `--help` lists the exit codes, and says Ctrl-C ends
/// `logs -f`, `attach` and `port-forward` with 0, not 130.
#[tokio::test]
async fn each_stream_verbs_help_lists_the_exit_codes_and_ctrl_c_ending_with_0() {
    let server = MockServer::start().await;
    let run = Run::new(&server);
    for verb in ["exec", "shell", "logs", "attach", "cp", "port-forward"] {
        let out = run.run(&["sandbox", verb, "--help"]);
        let help = stdout(&out);
        assert!(help.contains("Exit codes:"), "{verb}: {help}");
        let interrupted = help
            .lines()
            .find(|l| l.trim_start().starts_with("130 "))
            .unwrap_or_default();
        for zero in ["logs -f", "attach", "port-forward"] {
            assert!(
                interrupted.contains(zero) && interrupted.contains("exit 0"),
                "{verb}: {interrupted}"
            );
        }
    }
    assert!(server.received_requests().await.unwrap().is_empty());
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

/// A mint refusal reads the sandbox once. A 404 says only that no replica
/// answered, and a 400 FAILED_PRECONDITION "The sandbox is not active." only
/// that it is no longer active, so the sandbox's own state is the line, then
/// the next step; an instance stopped under a still-active sandbox keeps the
/// route's message, then names the instance's failure and the command that
/// prints its output, or only that command when the instance has no failure
/// reason, since the route's message already says it stopped. A
/// still-active sandbox whose replica 0 reads as running names the command
/// that reads it, by the id the name resolved to.
#[tokio::test]
async fn a_mint_refusal_reads_the_sandbox_and_names_its_state_unless_still_active() {
    const STARTED: &str = "2026-11-03T12:01:00Z";
    let mut ended = sandbox(
        ID,
        "box",
        "ended",
        json!([{"replica": 0, "status": "ended"}]),
    );
    ended["end_reason"] = json!("ttl_expired");
    let failed_instance = sandbox(
        ID,
        "box",
        "ended",
        json!([{"replica": 0, "status": "ended", "failure_reason": "oom",
                "started_at": STARTED}]),
    );
    let still_active = sandbox(
        ID,
        "box",
        "active",
        json!([{"replica": 0, "status": "stopped", "failure_reason": "oom",
                "started_at": STARTED}]),
    );
    let mut unpaid = sandbox(
        ID,
        "box",
        "active",
        json!([{"replica": 0, "status": "stopped", "started_at": STARTED}]),
    );
    unpaid["end_reason"] = json!("unpaid");
    let running = sandbox(
        ID,
        "box",
        "active",
        json!([{"replica": 0, "status": "running", "started_at": STARTED}]),
    );
    for (target, answer, read, line) in [
        (
            ID,
            envelope(
                400,
                "FAILED_PRECONDITION",
                "The sandbox is not active.",
                None,
            ),
            ended,
            format!(
                "hedra-cli: Not ready or stopped (13): `{ID}` is ended (ttl_expired). \
                 Create another, or run `hedra-cli sandbox ls` for the running ones."
            ),
        ),
        (
            ID,
            envelope(
                400,
                "FAILED_PRECONDITION",
                "The instance at this replica has stopped.",
                None,
            ),
            still_active,
            format!(
                "hedra-cli: Not ready or stopped (13): The instance at this replica has stopped. \
                 `{ID}`'s instance stopped (oom). Run `hedra-cli sandbox logs {ID}` for its output."
            ),
        ),
        (
            ID,
            envelope(
                400,
                "FAILED_PRECONDITION",
                "The instance at this replica has stopped.",
                None,
            ),
            unpaid,
            format!(
                "hedra-cli: Not ready or stopped (13): The instance at this replica has stopped. \
                 Run `hedra-cli sandbox logs {ID}` for its output."
            ),
        ),
        (
            "box",
            envelope(404, "NOT_FOUND", "No replica is serving sessions.", None),
            running.clone(),
            format!(
                "hedra-cli: Not ready or stopped (13): `box` is active. \
                 Check on it with `hedra-cli sandbox get --sandbox {ID}`."
            ),
        ),
        (
            "box",
            envelope(
                400,
                "FAILED_PRECONDITION",
                "The instance at this replica has stopped.",
                None,
            ),
            running,
            format!(
                "hedra-cli: Not ready or stopped (13): The instance at this replica has stopped. \
                 Check on it with `hedra-cli sandbox get --sandbox {ID}`."
            ),
        ),
        (
            ID,
            envelope(404, "NOT_FOUND", "No replica is serving sessions.", None),
            failed_instance,
            format!(
                "hedra-cli: Not ready or stopped (13): `{ID}`'s instance stopped (oom). \
                 Run `hedra-cli sandbox logs {ID}` for its output."
            ),
        ),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(format!("/v3/sandboxes/{ID}/session")))
            .respond_with(answer)
            .mount(&server)
            .await;
        if target != ID {
            mount_sandbox(&server, target, read.clone()).await;
        }
        mount_sandbox(&server, ID, read).await;
        // `exec` exits 255 for any failure of hedra-cli's own; the line names
        // the class.
        let out = Run::new(&server).run(&["sandbox", "exec", target, "-n", "--", "x"]);
        assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
        assert_eq!(
            class_line(&stderr(&out)),
            Some(line.as_str()),
            "{}",
            stderr(&out)
        );
        assert_eq!(
            requests_to(&server, "GET", &format!("/v3/sandboxes/{ID}"))
                .await
                .len(),
            1
        );
    }
}

/// Once a sandbox ends its name is free for another, so a refusal's next
/// step names the sandbox by the id its name resolved to; the prose keeps the
/// name as typed.
#[tokio::test]
async fn a_refusal_by_name_names_the_id_in_its_next_step() {
    let mut ended = sandbox(
        ID,
        "box",
        "ended",
        json!([{"replica": 0, "status": "stopped", "started_at": "2026-11-03T12:01:00Z"}]),
    );
    ended["end_reason"] = json!("ttl_ended");
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{ID}/session")))
        .respond_with(envelope(
            400,
            "FAILED_PRECONDITION",
            "The sandbox is not active.",
            None,
        ))
        .mount(&server)
        .await;
    mount_sandbox(&server, ID, ended.clone()).await;
    mount_sandbox(&server, "box", ended).await;
    let out = Run::new(&server).run(&["sandbox", "exec", "box", "-n", "--", "x"]);
    assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
    assert_eq!(
        class_line(&stderr(&out)),
        Some(
            format!(
                "hedra-cli: Not ready or stopped (13): `box` is ended (ttl_ended). \
                 Run `hedra-cli sandbox logs {ID}` for its output."
            )
            .as_str()
        ),
        "{}",
        stderr(&out)
    );
}

/// A mint refused 400 FAILED_PRECONDITION or 404 whose sandbox read fails
/// keeps the route's message, then names the command that reads the
/// sandbox; a read that answers 404 itself is the line, with no step, since
/// that command would fail the same way.
#[tokio::test]
async fn a_mint_refusal_whose_read_fails_keeps_the_routes_message() {
    let get = format!(" Check on it with `hedra-cli sandbox get --sandbox {ID}`.");
    for (target, answer, read, line) in [
        (
            "box",
            envelope(
                400,
                "FAILED_PRECONDITION",
                "The sandbox is not active.",
                None,
            ),
            envelope(503, "UNAVAILABLE", "busy", Some(0)),
            format!("Not ready or stopped (13): The sandbox is not active.{get}"),
        ),
        (
            ID,
            envelope(404, "NOT_FOUND", "No replica is serving sessions.", None),
            envelope(503, "UNAVAILABLE", "busy", Some(0)),
            format!("Rejected request (19): No replica is serving sessions.{get}"),
        ),
        (
            ID,
            envelope(404, "NOT_FOUND", "Sandbox not found.", None),
            envelope(404, "NOT_FOUND", "Sandbox not found.", None),
            format!("Not found (11): No sandbox `{ID}`. Run `hedra-cli sandbox ls`."),
        ),
    ] {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(format!("/v3/sandboxes/{ID}/session")))
            .respond_with(answer)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(format!("/v3/sandboxes/{ID}")))
            .respond_with(read)
            .mount(&server)
            .await;
        // By name, the lookup reads the sandbox; the read after the mint
        // fails all the same, and the step still names the id.
        if target != ID {
            mount_sandbox(&server, target, sandbox(ID, "box", "active", running())).await;
        }
        // `exec` exits 255 for any failure of hedra-cli's own.
        let out = Run::new(&server).run(&["sandbox", "exec", target, "-n", "--", "x"]);
        assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
        assert_eq!(
            class_line(&stderr(&out)),
            Some(format!("hedra-cli: {line}").as_str()),
            "{}",
            stderr(&out)
        );
        assert_eq!(
            requests_to(&server, "GET", &format!("/v3/sandboxes/{ID}"))
                .await
                .len(),
            1
        );
    }
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
        stderr(&out).contains(
            "The live output ended; the rest is from the stored copy, which can miss the newest \
             output."
        ),
        "{}",
        stderr(&out)
    );
}

/// A follow whose wait ends with replica 0 stopped or failed reads the
/// sandbox again and ends as for a stopped sandbox: the stored copy, the
/// state line and exit 0, never the wait's own line, which would point back
/// at `logs`, nor its class.
#[tokio::test]
async fn logs_f_whose_wait_ends_stopped_prints_the_stored_copy_and_exits_0() {
    let creating = sandbox(ID, "box", "creating", json!(null));
    let exited = sandbox(
        ID,
        "box",
        "active",
        json!([{"replica": 0, "status": "stopped", "started_at": "2026-11-03T12:01:00Z"}]),
    );
    let full = sandbox(
        ID,
        "box",
        "active",
        json!([{"replica": 0, "status": "failed", "failure_reason": "no_capacity"}]),
    );
    let unpaid = |instance: serde_json::Value| {
        let mut s = sandbox(ID, "box", "ended", json!([instance]));
        s["end_reason"] = json!("unpaid");
        s
    };
    let funds = "Add funds to the workspace, then create another; `hedra-cli sandbox ls` lists \
                 the running ones.";
    let at = |b: &serde_json::Value| ResponseTemplate::new(200).set_body_json(b.clone());
    for (answers, line) in [
        // Stopped unpaid, started or not: funds, as every next step does.
        (
            vec![
                at(&creating),
                at(&unpaid(json!({"replica": 0, "status": "stopped"}))),
            ],
            format!("`{ID}` is ended (unpaid). {funds}"),
        ),
        (
            vec![
                at(&creating),
                at(&unpaid(json!({"replica": 0, "status": "stopped",
                                   "started_at": "2026-11-03T12:01:00Z"}))),
            ],
            format!("`{ID}` is ended (unpaid). {funds}"),
        ),
        (
            vec![at(&creating), at(&exited)],
            format!("`{ID}`'s instance stopped."),
        ),
        (
            vec![at(&creating), at(&full)],
            format!(
                "`{ID}`'s instance stopped (no_capacity). No `h100:1` was free. Run `hedra-cli \
                 sandbox gpus`, then create another."
            ),
        ),
        (
            vec![
                at(&creating),
                at(&sandbox(
                    ID,
                    "box",
                    "active",
                    json!([{"replica": 0, "status": "failed"}]),
                )),
            ],
            format!(
                "`{ID}`'s instance stopped. Create another, or run `hedra-cli sandbox ls` for the \
                 running ones."
            ),
        ),
    ] {
        let broker = FakeBroker::start();
        let server = MockServer::start().await;
        mount_session(&server, ID, &broker).await;
        Mock::given(method("GET"))
            .and(path(format!("/v3/sandboxes/{ID}")))
            .respond_with(Sequence::new(answers))
            .mount(&server)
            .await;
        mount_stored(&server, "stored\n").await;
        let out = Run::new(&server).run(&["sandbox", "logs", "-f", ID]);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(0), "{err}");
        assert_eq!(stdout(&out), "stored\n");
        assert!(err.ends_with(&format!("{line}\n")), "{err}");
        assert!(!err.contains("hedra-cli sandbox logs"), "{err}");
        assert!(!err.contains("already exited"), "{err}");
        assert!(broker.calls().is_empty());
    }
}

/// After a wait that ends with replica 0 failed, a read of the sandbox that
/// keeps failing for now is not retried without end: the follow goes on
/// from the wait's last read and exits 0, after a bounded number of reads.
#[tokio::test]
async fn logs_f_whose_wait_ends_failed_does_not_retry_a_failing_read_forever() {
    let creating = sandbox(ID, "box", "creating", json!(null));
    let full = sandbox(
        ID,
        "box",
        "active",
        json!([{"replica": 0, "status": "failed", "failure_reason": "no_capacity"}]),
    );
    let broker = FakeBroker::start();
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}")))
        .respond_with(Sequence::new(vec![
            ResponseTemplate::new(200).set_body_json(creating),
            ResponseTemplate::new(200).set_body_json(full),
            envelope(503, "UNAVAILABLE", "busy", Some(0)),
        ]))
        .mount(&server)
        .await;
    mount_stored(&server, "stored\n").await;
    let child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    let out = wait(child, WAIT);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(stdout(&out), "stored\n");
    assert!(
        err.ends_with(&format!(
            "`{ID}`'s instance stopped (no_capacity). No `h100:1` was free. Run `hedra-cli \
             sandbox gpus`, then create another.\n"
        )),
        "{err}"
    );
    assert!(!err.contains("retrying"), "{err}");
    // The first read, the wait's, and one that failed.
    let reads = requests_to(&server, "GET", &format!("/v3/sandboxes/{ID}")).await;
    assert_eq!(reads.len(), 3, "{err}");
    assert!(broker.calls().is_empty());
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

/// A follow whose stdout falls a spool behind stops reading, says so, and
/// once stdout drains reopens from the last chunk it printed, so every byte
/// the broker still holds arrives once.
#[tokio::test]
async fn logs_f_past_the_spool_stops_reading_then_reopens_from_its_last_chunk() {
    let broker = FakeBroker::start();
    fill_ring(&broker);
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let mut cmd = run.command(&["sandbox", "logs", "-f", ID]);
    cmd.env("HEDRA_CLI_TEST_LOG_SPOOL_BYTES", (256 * 1024).to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let mut child = cmd.spawn().unwrap();
    let err = collect(child.stderr.take().unwrap());
    wait_until("the spool's note", || {
        String::from_utf8_lossy(&err.lock().unwrap()).contains("faster than it is read")
    });
    let seen = collect(child.stdout.take().unwrap());
    wait_until("the reopen to replay the rest", || {
        seen.lock().unwrap().len() == 64 * 64 * 1024
    });
    assert!(broker.calls().len() >= 2, "{:?}", broker.calls().len());
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(seen.lock().unwrap().len(), 64 * 64 * 1024);
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
    assert!(
        stderr(&out).contains(&format!("Could not read `{ID}`'s state; retrying.")),
        "{}",
        stderr(&out)
    );
}

/// A follow whose sandbox is still starting when the wait's budget runs out
/// exits 13 with the wait's line.
#[tokio::test]
async fn logs_f_whose_sandbox_is_still_starting_after_the_budget_exits_13() {
    let broker = FakeBroker::start();
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    mount_sandbox(&server, ID, sandbox(ID, "box", "creating", json!(null))).await;
    let run = Run::new(&server);
    let mut cmd = run.command(&["sandbox", "logs", "-f", ID]);
    cmd.env("HEDRA_CLI_TEST_START_BUDGET_MS", "300");
    let child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let out = wait(child, WAIT);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(13), "{err}");
    assert!(
        class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Not ready or stopped (13): ")
            && l.contains("still starting after")),
        "{err}"
    );
    assert!(broker.calls().is_empty());
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

fn fell_behind() -> Status {
    broker_status(
        Code::Unavailable,
        "rate_limited",
        "client fell behind the sandbox's output; reconnect",
        &[],
    )
}

/// The `since` each `Logs` call asked for.
fn logs_since(broker: &FakeBroker) -> Vec<Option<(i64, i32)>> {
    broker
        .calls()
        .iter()
        .filter_map(|c| c.logs.as_ref())
        .map(|l| l.since.map(|t| (t.seconds, t.nanos)))
        .collect()
}

/// A plain `logs` whose live read the broker ends `rate_limited`, as when a
/// large ring buffer replays over a slower link, reopens after the last
/// chunk it printed, and ends with the reopen's replay, without the stored
/// copy.
#[tokio::test]
async fn plain_logs_ended_rate_limited_reopens_after_its_last_chunk() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.log(b"b\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())).then_append(b"c\n"));
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())).then_append(b"d\n"));
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "stored\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(stdout(&out), "a\nb\nc\nd\n");
    // Nothing marks a chunk the ring buffer dropped before a reopen, so any
    // read that reopened says output may be missing.
    assert_eq!(
        err,
        format!(
            "The live read of `{ID}` fell behind and reconnected 2 times; output written faster \
             than it was read may be missing.\n"
        ),
        "{err}"
    );
    assert!(stored_reads(&server).await.is_empty());
    // Each reopen resumes one nanosecond after the last chunk printed.
    let since = logs_since(&broker);
    assert_eq!(since.len(), 3, "{since:?}");
    assert_eq!(since[0], None);
    assert_eq!(since[1], Some((1_793_750_401, 500_000_002)));
    assert_eq!(since[2], Some((1_793_750_402, 500_000_003)));
    // One read of the sandbox: a reopen reuses it.
    assert_eq!(
        requests_to(&server, "GET", &format!("/v3/sandboxes/{ID}"))
            .await
            .len(),
        1
    );
}

/// A reopen whose `since` names a chunk the ring buffer has already
/// dropped replays only what is left, with no mark: the read prints what it
/// got and says output may be missing, whether it ends with OK trailers or
/// the buffer's eof marker.
#[tokio::test]
async fn plain_logs_whose_reopen_skipped_dropped_chunks_says_output_may_be_missing() {
    for end in [LogsEnd::Default, LogsEnd::Eof] {
        let broker = FakeBroker::start();
        broker.log(b"a\n");
        broker.log(b"b\n");
        // `c` is written and dropped before the reopen asks for it.
        broker.push_logs(
            LogsScript::admit(LogsEnd::Fail(fell_behind()))
                .then_append(b"c\n")
                .then_append(b"d\n")
                .then_evict(3),
        );
        broker.push_logs(LogsScript::admit(end.clone()));
        let server = running_sandbox(&broker).await;
        mount_stored(&server, "stored\n").await;
        let out = Run::new(&server).run(&["sandbox", "logs", ID]);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(0), "{end:?}: {err}");
        assert_eq!(stdout(&out), "a\nb\nd\n", "{end:?}");
        assert_eq!(
            err,
            format!(
                "The live read of `{ID}` fell behind and reconnected once; output written \
                 faster than it was read may be missing.\n"
            ),
            "{end:?}"
        );
        assert!(stored_reads(&server).await.is_empty());
        assert_eq!(broker.calls().len(), 2);
    }
}

/// Reopens stop at their cap even while each prints something, as when the
/// sandbox writes faster than the link carries: the rest comes from the
/// stored copy. Each reopen waits a moment first, so their replays are
/// spaced out.
#[tokio::test]
async fn plain_logs_stops_reopening_at_its_cap() {
    let broker = FakeBroker::start();
    broker.log(b"0\n");
    for i in 1..=21 {
        broker.push_logs(
            LogsScript::admit(LogsEnd::Fail(fell_behind()))
                .then_append(format!("{i}\n").as_bytes()),
        );
    }
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "stored\n").await;
    let started = std::time::Instant::now();
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    let took = started.elapsed();
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    // The first read and 20 reopens, then the stored copy.
    assert_eq!(broker.calls().len(), 21, "{err}");
    assert!(stdout(&out).starts_with("0\n1\n"));
    assert!(
        stdout(&out).ends_with("19\n20\nstored\n"),
        "{}",
        stdout(&out)
    );
    assert!(
        err.starts_with(&format!(
            "The live read of `{ID}` kept ending (rate_limited) after 20 reconnects; output written \
             faster than it was read may be missing.\n"
        )),
        "{err}"
    );
    assert_eq!(stored_reads(&server).await.len(), 1);
    // A quarter second before each of the 20 reopens.
    assert!(took >= Duration::from_secs(5), "{took:?}");
}

/// Reopens also stop once they have gone on for their time: at none, the
/// first read that falls behind goes to the stored copy, and with no reopen
/// nothing can have been skipped.
#[tokio::test]
async fn plain_logs_stops_reopening_after_its_time() {
    let broker = FakeBroker::start();
    broker.log(b"0\n");
    for i in 1..=3 {
        broker.push_logs(
            LogsScript::admit(LogsEnd::Fail(fell_behind()))
                .then_append(format!("{i}\n").as_bytes()),
        );
    }
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "stored\n").await;
    let run = Run::new(&server);
    let mut cmd = run.command(&["sandbox", "logs", ID]);
    cmd.env("HEDRA_CLI_TEST_CATCH_UP_MS", "0");
    let out = cmd.output().unwrap();
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(broker.calls().len(), 1, "{err}");
    assert_eq!(stdout(&out), "0\nstored\n");
    assert!(
        err.starts_with(&format!(
            "The live read of `{ID}` kept ending (rate_limited).\n"
        )),
        "{err}"
    );
    assert_eq!(stored_reads(&server).await.len(), 1);
}

/// The reopens' time counts from the first read that fell behind, not from
/// the latest: at 600 ms, with a quarter second before each reopen, a
/// read that keeps falling behind while printing stops after a few reopens,
/// well short of the cap.
#[tokio::test]
async fn plain_logs_reopening_time_counts_from_the_first_read_that_fell_behind() {
    let broker = FakeBroker::start();
    broker.log(b"0\n");
    for i in 1..=21 {
        broker.push_logs(
            LogsScript::admit(LogsEnd::Fail(fell_behind()))
                .then_append(format!("{i}\n").as_bytes()),
        );
    }
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "stored\n").await;
    let run = Run::new(&server);
    let mut cmd = run.command(&["sandbox", "logs", ID]);
    cmd.env("HEDRA_CLI_TEST_CATCH_UP_MS", "600");
    let out = cmd.output().unwrap();
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    // About three: the first read and the reopens that start inside 600 ms
    // of it; a time that restarted at each read would reach the cap's 21.
    let calls = broker.calls().len();
    assert!((2..=6).contains(&calls), "{calls}: {err}");
    assert!(stdout(&out).ends_with("stored\n"), "{}", stdout(&out));
    let reopened = match calls - 1 {
        1 => "1 reconnect".to_string(),
        n => format!("{n} reconnects"),
    };
    assert!(
        err.starts_with(&format!(
            "The live read of `{ID}` kept ending (rate_limited) after {reopened}; output \
             written faster than it was read may be missing.\n"
        )),
        "{err}"
    );
}

/// Reopens that keep ending `rate_limited` with nothing new end in the
/// stored copy after the last printed chunk, with one note naming why. No
/// reopen printed, so none moved past a chunk the stored copy then misses,
/// and the note does not say output may be missing.
#[tokio::test]
async fn plain_logs_whose_reopens_stall_finishes_from_the_stored_copy() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.log(b"b\n");
    for _ in 0..4 {
        broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())));
    }
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "c\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(stdout(&out), "a\nb\nc\n");
    assert_eq!(class_line(&err), None, "{err}");
    assert_eq!(
        err,
        format!(
            "The live read of `{ID}` kept ending (rate_limited) after 3 reconnects.\n\
             The live output ended; the rest is from the stored copy, which can miss the newest \
             output.\n"
        )
    );
    // The first read printed; three in a row after it printed nothing.
    assert_eq!(broker.calls().len(), 4);
    let reads = stored_reads(&server).await;
    assert_eq!(reads.len(), 1);
    let since = reads[0]
        .url
        .query_pairs()
        .find(|(k, _)| k == "since")
        .map(|(_, v)| v.into_owned());
    assert_eq!(since.as_deref(), Some("2026-11-04T00:00:01.500000002Z"));
}

/// Reopens that stall after one that printed end in the stored copy too,
/// but that one may have moved past chunks the ring buffer had dropped,
/// which the stored copy, read from after the last printed chunk, does not
/// bring back: the note says output may be missing.
#[tokio::test]
async fn plain_logs_whose_reopens_stall_after_one_printed_says_output_may_be_missing() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())).then_append(b"b\n"));
    for _ in 0..4 {
        broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())));
    }
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "c\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(stdout(&out), "a\nb\nc\n");
    assert_eq!(
        err,
        format!(
            "The live read of `{ID}` kept ending (rate_limited) after 4 reconnects; output written \
             faster than it was read may be missing.\n\
             The live output ended; the rest is from the stored copy, which can miss the newest \
             output.\n"
        )
    );
    // The first read, one reopen that printed, and three in a row that did
    // not.
    assert_eq!(broker.calls().len(), 5);
    assert_eq!(stored_reads(&server).await.len(), 1);
}

/// A catch-up reopen that is refused ends in the stored copy after the last
/// printed chunk, not in the refusal's exit code.
#[tokio::test]
async fn plain_logs_whose_reopen_is_refused_finishes_from_the_stored_copy() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())));
    broker.push_logs(LogsScript::refuse(Status::unavailable("broker restarting")));
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "b\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(stdout(&out), "a\nb\n");
    assert!(
        err.starts_with(&format!(
            "The live read of `{ID}` ended and could not reconnect.\n"
        )),
        "{err}"
    );
    assert_eq!(broker.calls().len(), 2);
    assert_eq!(stored_reads(&server).await.len(), 1);

    // After a reopen that did run, which may have skipped dropped chunks.
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())).then_append(b"b\n"));
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())));
    broker.push_logs(LogsScript::refuse(Status::unavailable("broker restarting")));
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "c\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(stdout(&out), "a\nb\nc\n");
    assert!(
        err.starts_with(&format!(
            "The live read of `{ID}` ended and could not reconnect; output written faster than \
             it was read may be missing.\n"
        )),
        "{err}"
    );
    assert_eq!(broker.calls().len(), 3);
}

/// A sandbox that stops between reopens ends the read in the stored copy
/// after the last printed chunk, which cannot bring back a chunk an earlier
/// reopen skipped, so the read says output may be missing however the
/// reopen learns of the stop: refused `sandbox_disconnected`, a mint refused
/// for a stopped sandbox, or the stream ended `sandbox_disconnected`.
#[tokio::test]
async fn plain_logs_whose_sandbox_stops_after_a_reopen_says_output_may_be_missing() {
    let stopped = || {
        envelope(
            400,
            "FAILED_PRECONDITION",
            "The sandbox is not active.",
            None,
        )
    };
    // The reopens that read: a refused one did not, a dropped one did.
    for (how, last, times) in [
        ("refused", Some(LogsScript::refuse(disconnected())), "once"),
        ("mint", None, "once"),
        (
            "dropped",
            Some(LogsScript::admit(LogsEnd::Fail(disconnected()))),
            "2 times",
        ),
    ] {
        let broker = FakeBroker::start();
        broker.log(b"a\n");
        broker.log(b"b\n");
        // `c` is dropped before the reopen asks for it.
        broker.push_logs(
            LogsScript::admit(LogsEnd::Fail(fell_behind()))
                .then_append(b"c\n")
                .then_append(b"d\n")
                .then_evict(3),
        );
        broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())));
        let server = MockServer::start().await;
        match last {
            Some(script) => {
                broker.push_logs(script);
                mount_session(&server, ID, &broker).await;
            }
            // Tokens that are already stale, so each open mints, and the
            // third mint is refused: the sandbox stopped.
            None => {
                let stale = ResponseTemplate::new(200).set_body_json(json!({
                    "token": TOKEN, "broker_url": broker.broker_url(),
                    "expires_at": "2000-01-01T00:00:00Z", "replica": 0,
                }));
                Mock::given(method("POST"))
                    .and(path(format!("/v3/sandboxes/{ID}/session")))
                    .respond_with(Sequence::new(vec![stale.clone(), stale, stopped()]))
                    .mount(&server)
                    .await;
            }
        }
        mount_sandbox(&server, ID, sandbox(ID, "box", "active", running())).await;
        mount_stored(&server, "e\n").await;
        let out = Run::new(&server).run(&["sandbox", "logs", ID]);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(0), "{how}: {err}");
        assert_eq!(stdout(&out), "a\nb\nd\ne\n", "{how}");
        assert!(
            err.starts_with(&format!(
                "The live read of `{ID}` fell behind and reconnected {times}; output written \
                 faster than it was read may be missing.\n"
            )),
            "{how}: {err}"
        );
    }
}

/// A read whose only reopen printed nothing ends in the stored copy from
/// the last chunk the first read printed, which misses nothing a reopen
/// skipped: however the read ends there, no note says output may be
/// missing.
#[tokio::test]
async fn plain_logs_whose_reopen_printed_nothing_says_nothing_is_missing_before_the_stored_copy() {
    let stopped = || {
        envelope(
            400,
            "FAILED_PRECONDITION",
            "The sandbox is not active.",
            None,
        )
    };
    for (how, last) in [
        ("refused", Some(LogsScript::refuse(disconnected()))),
        ("mint", None),
        (
            "dropped",
            Some(LogsScript::admit(LogsEnd::Fail(disconnected()))),
        ),
        (
            "unavailable",
            Some(LogsScript::refuse(Status::unavailable("broker restarting"))),
        ),
    ] {
        let broker = FakeBroker::start();
        broker.log(b"a\n");
        broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())));
        broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())));
        let server = MockServer::start().await;
        match last {
            Some(script) => {
                broker.push_logs(script);
                mount_session(&server, ID, &broker).await;
            }
            // Tokens that are already stale, so each open mints, and the
            // third mint is refused: the sandbox stopped.
            None => {
                let stale = ResponseTemplate::new(200).set_body_json(json!({
                    "token": TOKEN, "broker_url": broker.broker_url(),
                    "expires_at": "2000-01-01T00:00:00Z", "replica": 0,
                }));
                Mock::given(method("POST"))
                    .and(path(format!("/v3/sandboxes/{ID}/session")))
                    .respond_with(Sequence::new(vec![stale.clone(), stale, stopped()]))
                    .mount(&server)
                    .await;
            }
        }
        mount_sandbox(&server, ID, sandbox(ID, "box", "active", running())).await;
        mount_stored(&server, "b\n").await;
        let out = Run::new(&server).run(&["sandbox", "logs", ID]);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(0), "{how}: {err}");
        assert_eq!(stdout(&out), "a\nb\n", "{how}");
        assert!(!err.contains("may be missing"), "{how}: {err}");
        assert!(!err.contains("fell behind"), "{how}: {err}");
        assert_eq!(stored_reads(&server).await.len(), 1, "{how}");
    }
}

/// A reopen that then fails for another reason exits with that failure,
/// after the note that output may be missing.
#[tokio::test]
async fn plain_logs_whose_reopen_fails_says_output_may_be_missing_before_the_failure() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())).then_append(b"b\n"));
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(Status::unavailable(
        "broker restarting",
    ))));
    let server = running_sandbox(&broker).await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(18), "{err}");
    assert_eq!(stdout(&out), "a\nb\n");
    assert!(
        err.starts_with(&format!(
            "The live read of `{ID}` fell behind and reconnected once; output written faster \
             than it was read may be missing.\n"
        )),
        "{err}"
    );
    assert!(
        class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Connection lost (18): ")),
        "{err}"
    );
}

/// `rate_limited` before admission is the Limit class, as for every verb:
/// only an admitted read that falls behind reopens.
#[tokio::test]
async fn plain_logs_refused_rate_limited_before_admission_exits_16() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(LogsScript::refuse(broker_status(
        Code::Unavailable,
        "rate_limited",
        "too many sessions",
        &[],
    )));
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "stored\n").await;
    let out = Run::new(&server).run(&["sandbox", "logs", ID]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(16), "{err}");
    assert!(
        class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Limit (16): ")),
        "{err}"
    );
    assert_eq!(stdout(&out), "");
    assert!(stored_reads(&server).await.is_empty());
    assert_eq!(broker.calls().len(), 1);
}

/// `logs -f` ended `rate_limited` reopens as it does any dropped stream,
/// and keeps following: no stored copy.
#[tokio::test]
async fn logs_f_ended_rate_limited_reopens_and_keeps_following() {
    let broker = FakeBroker::start();
    broker.log(b"a\n");
    broker.push_logs(LogsScript::admit(LogsEnd::Fail(fell_behind())).then_append(b"b\n"));
    broker.push_logs(LogsScript::admit(LogsEnd::Hold));
    let server = running_sandbox(&broker).await;
    mount_stored(&server, "stored\n").await;
    let mut child = Run::new(&server).spawn(&["sandbox", "logs", "-f", ID]);
    let seen = collect(child.stdout.take().unwrap());
    wait_until("the chunk after the reopen", || {
        seen.lock().unwrap().ends_with(b"b\n")
    });
    broker.wait_for(WAIT, |c| c.len() == 2 && c[1].admitted);
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(String::from_utf8_lossy(&seen.lock().unwrap()), "a\nb\n");
    assert_eq!(broker.calls().len(), 2);
    assert!(broker
        .calls()
        .iter()
        .all(|c| c.logs.as_ref().unwrap().follow));
    assert!(stored_reads(&server).await.is_empty());
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
