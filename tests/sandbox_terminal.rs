//! `exec -t` and `shell <name>` on a pseudo-terminal, against a fake broker.

#![cfg(unix)]

mod sandbox_stream_support;

use std::time::Duration;

use sandbox_stream_support::*;

const WAIT: Duration = Duration::from_secs(20);

#[tokio::test]
async fn sigterm_under_t_restores_the_terminal_and_term_is_normalized() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let pty = Pty::open(30, 100);
    assert!(pty.cooked());
    let mut cmd = run.command(&["sandbox", "exec", "-t", ID, "--", "bash"]);
    cmd.env("TERM", "xterm-kitty");
    let child = pty.spawn(&cmd);
    let calls = broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
    assert!(
        !pty.cooked(),
        "the terminal is not raw while the call is open"
    );

    let start = calls[0].start.clone().unwrap();
    assert!(start.tty);
    assert_eq!(start.argv, ["bash"]);
    assert_eq!(
        start.env.get("TERM").map(String::as_str),
        Some("xterm-256color")
    );
    assert_eq!((start.rows, start.cols), (30, 100));

    child.signal(libc::SIGTERM);
    let out = child.finish(WAIT);
    assert_eq!(out.code, 143, "{}", out.stderr);
    assert!(pty.cooked(), "the terminal was left raw");
    broker.wait_for(WAIT, |c| c[0].client_gone);
}

#[tokio::test]
async fn term_passes_a_common_name_through_and_term_overrides() {
    for (local, flag, sent) in [
        ("tmux-256color", None, "tmux-256color"),
        ("xterm-ghostty", None, "xterm-256color"),
        ("xterm-ghostty", Some("xterm-ghostty"), "xterm-ghostty"),
    ] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let pty = Pty::open(24, 80);
        let mut args = vec!["sandbox", "exec", "-t", ID];
        if let Some(t) = flag {
            args.extend(["--term", t]);
        }
        args.extend(["--", "top"]);
        let mut cmd = run.command(&args);
        cmd.env("TERM", local);
        let out = pty.spawn(&cmd).finish(WAIT);
        assert_eq!(out.code, 0, "{}", out.stderr);
        let start = broker.calls()[0].start.clone().unwrap();
        assert_eq!(start.env.get("TERM").map(String::as_str), Some(sent));
    }
}

#[tokio::test]
async fn an_env_term_is_kept_and_term_still_wins() {
    for (flag, sent) in [(None, "xterm-kitty"), (Some("vt100"), "vt100")] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let pty = Pty::open(24, 80);
        let mut args = vec!["sandbox", "exec", "-t", "--env", "TERM=xterm-kitty", ID];
        if let Some(t) = flag {
            args.extend(["--term", t]);
        }
        args.extend(["--", "top"]);
        let mut cmd = run.command(&args);
        cmd.env("TERM", "screen");
        let out = pty.spawn(&cmd).finish(WAIT);
        assert_eq!(out.code, 0, "{}", out.stderr);
        let start = broker.calls()[0].start.clone().unwrap();
        assert_eq!(start.env.get("TERM").map(String::as_str), Some(sent));
    }
}

#[tokio::test]
async fn shell_sends_the_non_login_argv_on_a_terminal() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(4)]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let child = pty.spawn(&run.command(&["sandbox", "shell", ID]));
    let out = child.finish(WAIT);
    assert_eq!(out.code, 4, "{}", out.stderr);
    assert!(pty.cooked());
    let start = broker.calls()[0].start.clone().unwrap();
    assert!(start.tty);
    assert_eq!(
        start.argv,
        [
            "sh",
            "-c",
            "exec \"${SHELL:-$(command -v bash || echo sh)}\""
        ]
    );
}

/// Each refusal offers what runs without a terminal: `exec` without `-t`,
/// and for `shell`, a command through `exec`.
#[tokio::test]
async fn t_without_a_terminal_is_refused_before_any_request() {
    for (args, alternative) in [
        (
            &["sandbox", "exec", "-t", ID, "--", "bash"][..],
            "run it in a terminal, or drop -t.",
        ),
        (
            &["sandbox", "shell", ID][..],
            "run it in a terminal, or run a command without one with \
             `hedra-cli sandbox exec <sandbox> -- <command>`.",
        ),
    ] {
        let broker = FakeBroker::start();
        let server = running_sandbox(&broker).await;
        let out = Run::new(&server).run(args);
        assert_eq!(out.status.code(), Some(255), "{}", stderr(&out));
        assert_eq!(
            class_line(&stderr(&out)),
            Some(
                format!(
                    "hedra-cli: Rejected request (19): This needs a terminal on stdin and stdout: \
                     {alternative}"
                )
                .as_str()
            ),
            "{}",
            stderr(&out)
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }
}

/// A background job has the terminal but cannot take it.
#[tokio::test]
async fn t_in_a_background_job_is_refused_before_any_request() {
    for (args, alternative) in [
        (
            &["sandbox", "exec", "-t", ID, "--", "bash"][..],
            ", or drop -t.",
        ),
        (
            &["sandbox", "shell", ID][..],
            ", or run a command without one with `hedra-cli sandbox exec <sandbox> -- <command>`.",
        ),
    ] {
        let broker = FakeBroker::start();
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let pty = Pty::open(24, 80);
        let out = pty.spawn(&in_background(&run.command(args))).finish(WAIT);
        assert_eq!(out.code, 255, "{}", out.stderr);
        assert_eq!(
            class_line(&out.stderr),
            Some(
                format!(
                    "hedra-cli: Rejected request (19): This needs the terminal: run it in the \
                     foreground{alternative}"
                )
                .as_str()
            ),
            "{}",
            out.stderr
        );
        assert!(server.received_requests().await.unwrap().is_empty());
    }
}

#[tokio::test]
async fn under_nohup_an_exec_stays_open_after_its_terminal_closes() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let child = pty.spawn_nohup(&run.command(&["sandbox", "exec", ID, "--", "train"]));
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
    pty.close();
    child.signal(libc::SIGHUP);
    std::thread::sleep(Duration::from_millis(1500));
    let calls = broker.calls();
    assert!(!calls[0].client_gone, "the call ended: {:?}", calls[0]);
    child.signal(libc::SIGTERM);
    let out = child.finish(WAIT);
    assert_eq!(out.code, 143, "{}", out.stderr);
    broker.wait_for(WAIT, |c| c[0].client_gone);
}

#[tokio::test]
async fn under_t_stdin_ending_with_the_terminal_cancels_the_call_and_exits_129() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    // SIGHUP ignored, so only stdin ending can end it.
    let child = pty.spawn_nohup(&run.command(&["sandbox", "exec", "-t", ID, "--", "bash"]));
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
    pty.close();
    let out = child.finish(WAIT);
    assert_eq!(out.code, 129, "{}", out.stderr);
    assert_eq!(class_line(&out.stderr), None, "{}", out.stderr);
    broker.wait_for(WAIT, |c| c[0].client_gone);
}

#[tokio::test]
async fn a_resized_terminal_sends_its_new_size() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let child = pty.spawn(&run.command(&["sandbox", "exec", "-t", ID, "--", "top"]));
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
    // The size change itself signals the terminal's foreground group.
    pty.resize(40, 132);
    broker.wait_for(WAIT, |c| c[0].resizes.last() == Some(&(40, 132)));
    child.signal(libc::SIGTERM);
    let out = child.finish(WAIT);
    assert_eq!(out.code, 143, "{}", out.stderr);
}

#[tokio::test]
async fn a_panic_under_raw_mode_restores_the_terminal_then_prints_once() {
    for place in ["main", "task"] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let pty = Pty::open(24, 80);
        let mut cmd = run.command(&["sandbox", "exec", "-t", ID, "--", "bash"]);
        cmd.env("HEDRA_CLI_TEST_PANIC", place);
        let child = pty.spawn(&cmd);
        let out = child.finish(WAIT);
        assert_eq!(out.code, 255, "{place}: {}", out.stderr);
        assert!(pty.cooked(), "{place}: the terminal was left raw");
        assert_eq!(
            out.stderr.matches("injected panic").count(),
            1,
            "{place}: {}",
            out.stderr
        );
        assert!(
            class_line(&out.stderr).is_some_and(|l| l.starts_with("hedra-cli: Panic (255): ")),
            "{place}: {}",
            out.stderr
        );
        broker.wait_for(WAIT, |c| c[0].client_gone);
    }
}

#[tokio::test]
async fn a_reset_broker_connection_under_t_is_connection_lost_not_sigpipe() {
    let proxy = Proxy::start();
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Echo]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let mut cmd = run.command(&["sandbox", "exec", "-t", ID, "--", "cat"]);
    cmd.env("HTTPS_PROXY", proxy.url());
    let child = pty.spawn(&cmd);
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
    proxy.reset();
    // Keys keep the client writing to the reset connection.
    let typing = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    let out = std::thread::scope(|s| {
        s.spawn(|| {
            while typing.load(std::sync::atomic::Ordering::SeqCst) {
                pty.type_keys(&[b'k'; 512]);
                std::thread::sleep(Duration::from_millis(5));
            }
        });
        let out = child.finish(WAIT);
        typing.store(false, std::sync::atomic::Ordering::SeqCst);
        out
    });
    assert_eq!(out.code, 255, "{}", out.stderr);
    assert!(
        class_line(&out.stderr).is_some_and(|l| l.starts_with("hedra-cli: Connection lost (18): ")),
        "{}",
        out.stderr
    );
    assert!(pty.cooked(), "the terminal was left raw");
}

#[tokio::test]
async fn under_t_a_nonblocking_terminal_is_waited_on_rather_than_hung_up() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(vec![b'o'; 256 * 1024]),
        Step::Hold,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    pty.set_nonblocking(true);
    let child = pty.spawn(&run.command(&["sandbox", "exec", "-t", ID, "--", "bash"]));
    broker.wait_for(WAIT, |c| c.first().is_some_and(|c| c.start.is_some()));
    std::thread::sleep(Duration::from_millis(1500));
    pty.type_keys(b"abc");
    broker.wait_for(WAIT, |c| c[0].stdin == b"abc");
    // Blocking again, so the `sh` leader's own read waits.
    pty.set_nonblocking(false);
    child.signal(libc::SIGTERM);
    let out = child.finish(WAIT);
    assert_eq!(out.code, 143, "{}", out.stderr);
    assert_eq!(class_line(&out.stderr), None, "{}", out.stderr);
    assert!(pty.cooked(), "the terminal was left raw");
}

#[tokio::test]
async fn a_debug_line_written_under_raw_mode_returns_the_carriage() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Fail(Status::unavailable("going away")),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let child = pty.spawn(&run.command(&["sandbox", "exec", "--debug", "-t", ID, "--", "bash"]));
    let out = child.finish(WAIT);
    assert_eq!(out.code, 255, "{}", out.stderr);
    let at = out.stderr.find("< exec ended: ").expect(&out.stderr);
    let line = out.stderr[at..].split_inclusive('\n').next().unwrap();
    assert!(line.ends_with("\r\n"), "{line:?}");
}

/// `shell` on a sandbox whose replica's instance stopped while the sandbox
/// still reads active exits 255 with the route's message, then that the
/// command already exited and how to read its output.
#[tokio::test]
async fn shell_on_a_stopped_instance_names_the_logs_command() {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer};
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{ID}/session")))
        .respond_with(envelope(
            400,
            "FAILED_PRECONDITION",
            "The instance at this replica has stopped.",
            None,
        ))
        .mount(&server)
        .await;
    let exited = sandbox(
        ID,
        "box",
        "active",
        serde_json::json!([{"replica": 0, "status": "stopped"}]),
    );
    mount_sandbox(&server, "box", exited.clone()).await;
    mount_sandbox(&server, ID, exited).await;
    let run = Run::new(&server);
    let pty = Pty::open(24, 80);
    let out = pty
        .spawn(&run.command(&["sandbox", "shell", "box"]))
        .finish(WAIT);
    assert_eq!(out.code, 255, "{}", out.stderr);
    assert_eq!(
        class_line(&out.stderr),
        Some(
            "hedra-cli: Not ready or stopped (13): The instance at this replica has stopped. \
             `box`'s command already exited. Run `hedra-cli sandbox logs box` for its output."
        ),
        "{}",
        out.stderr
    );
}
