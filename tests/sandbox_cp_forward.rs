//! `cp` and `port-forward`, run as the built binary against a mock v3 API
//! and a fake broker.

#![cfg(unix)]

mod sandbox_stream_support;

use std::io::Read;
use std::net::TcpStream;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sandbox_stream_support::*;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const WAIT: Duration = Duration::from_secs(20);

async fn request_count(server: &MockServer) -> usize {
    server.received_requests().await.unwrap_or_default().len()
}

/// A ustar header, its name written as given.
fn header(name: &str, kind: u8, mode: u32, size: u64) -> Vec<u8> {
    let mut h = [0u8; 512];
    h[..name.len()].copy_from_slice(name.as_bytes());
    let mode = format!("{mode:07o}\0");
    let size = format!("{size:011o}\0");
    for (at, field) in [
        (100, mode.as_bytes()),
        (108, b"0000000\0"),
        (116, b"0000000\0"),
        (124, size.as_bytes()),
        (136, b"00000000000\0"),
    ] {
        h[at..at + field.len()].copy_from_slice(field);
    }
    h[156] = kind;
    h[257..265].copy_from_slice(b"ustar\x0000");
    h[148..156].fill(b' ');
    let sum: u32 = h.iter().map(|b| u32::from(*b)).sum();
    h[148..155].copy_from_slice(format!("{sum:06o}\0").as_bytes());
    h.to_vec()
}

/// A ustar stream of one one-byte file, its name written as given.
fn one_entry(name: &str) -> Vec<u8> {
    let mut out = header(name, b'0', 0o644, 1);
    out.push(b'x');
    out.resize(10240, 0);
    out
}

#[tokio::test(flavor = "multi_thread")]
async fn cp_refusing_an_entry_exits_255_and_escapes_the_sandbox_stderr() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(one_entry("/etc/evil")),
        Step::Stderr(b"tar: \x1b]52;c;evil\x07\n".to_vec()),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cp_src = format!("{ID}:/work/tree");
    let out = run.run(&["sandbox", "cp", &cp_src, "out"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    let line = class_line(&err).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Rejected request (19):"),
        "{err}"
    );
    assert!(line.contains("absolute"), "{err}");
    assert!(
        !err.contains('\x1b') && err.contains("]52;c;evil"),
        "{err:?}"
    );
    assert!(
        !run.home.path().join("out").exists()
            || std::fs::read_dir(run.home.path().join("out"))
                .unwrap()
                .next()
                .is_none()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn the_preview_gate_sends_nothing() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cp_src = format!("{ID}:/work/tree");
    // `cp` runs a command, so it exits 255 under the 255 rule;
    // `port-forward` exits with the Rejected request code.
    for (args, code) in [
        (vec!["sandbox", "cp", cp_src.as_str(), "out"], 255),
        (vec!["sandbox", "port-forward", ID, "0:80"], 19),
    ] {
        let out = run
            .command(&args)
            .env_remove("HEDRA_CLI_SANDBOX_PREVIEW")
            .output()
            .unwrap();
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(code), "{args:?}: {err}");
        assert!(err.contains("not released yet"), "{err}");
    }
    assert_eq!(request_count(&server).await, 0);
    assert!(broker.calls().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn bad_cp_arguments_exit_255_with_no_request() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let ro = run.home.path().join("ro");
    std::fs::create_dir(&ro).unwrap();
    std::fs::set_permissions(&ro, std::os::unix::fs::PermissionsExt::from_mode(0o555)).unwrap();
    let src = format!("{ID}:/work/tree");
    let climbs = format!("{ID}:a/..");
    for args in [
        vec!["sandbox", "cp", "a", "b"],
        vec!["sandbox", "cp", climbs.as_str(), "out"],
        vec!["sandbox", "cp", "--dry-run", "a", "b"],
        vec!["sandbox", "cp", "--dry-run", climbs.as_str(), "out"],
        // Not a sandbox name, so neither side names a sandbox.
        vec!["sandbox", "cp", "Not_A_Name:/work", "out"],
        // A destination that cannot be written costs no request.
        vec!["sandbox", "cp", src.as_str(), "ro/out"],
    ] {
        let out = run.run(&args);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(255), "{args:?}: {err}");
        assert!(
            class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19):")),
            "{args:?}: {err}"
        );
    }
    // A good one under --dry-run passes and writes nothing.
    let out = run.run(&["sandbox", "cp", "--dry-run", &src, "new/out"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert!(!run.home.path().join("new").exists());
    assert_eq!(request_count(&server).await, 0);
    assert!(broker.calls().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn port_forward_bad_ports_and_a_taken_port_exit_19_with_no_request() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let held = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let taken = format!("{}:80", held.local_addr().unwrap().port());
    for args in [
        vec!["sandbox", "port-forward", ID, "+80"],
        vec!["sandbox", "port-forward", "--dry-run", ID, "+80"],
        vec!["sandbox", "port-forward", "--dry-run", ID, "80:0"],
        vec!["sandbox", "port-forward", "Not_A_Name", "80"],
        // By name, which costs a request once resolved.
        vec!["sandbox", "port-forward", "box", taken.as_str()],
    ] {
        let out = run.run(&args);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(19), "{args:?}: {err}");
        assert!(
            class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19):")),
            "{args:?}: {err}"
        );
    }
    assert_eq!(request_count(&server).await, 0);
    assert!(broker.calls().is_empty());
}

/// A running port-forward, its stderr collected as it comes.
struct Forward {
    child: Child,
    err: Arc<Mutex<String>>,
    reader: Option<std::thread::JoinHandle<()>>,
}

impl Forward {
    fn start(run: &Run) -> Self {
        Self::start_with(run, &["sandbox", "port-forward", ID, "0:8888"])
    }

    fn start_with(run: &Run, args: &[&str]) -> Self {
        Self::from_command(run.command(args))
    }

    fn from_command(mut cmd: std::process::Command) -> Self {
        let mut child = cmd
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let mut pipe = child.stderr.take().unwrap();
        let err = Arc::new(Mutex::new(String::new()));
        let into = err.clone();
        let reader = std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = pipe.read(&mut buf) {
                if n == 0 {
                    break;
                }
                into.lock()
                    .unwrap()
                    .push_str(&String::from_utf8_lossy(&buf[..n]));
            }
        });
        Self {
            child,
            err,
            reader: Some(reader),
        }
    }

    fn stderr(&self) -> String {
        self.err.lock().unwrap().clone()
    }

    fn wait_for(&self, what: &str) -> String {
        self.wait_for_within(what, WAIT)
    }

    fn wait_for_within(&self, what: &str, within: Duration) -> String {
        let deadline = Instant::now() + within;
        loop {
            let err = self.stderr();
            if err.contains(what) {
                return err;
            }
            assert!(Instant::now() < deadline, "no {what:?} in: {err}");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// The local port from the "Forwarding" line.
    fn port(&self) -> u16 {
        let err = self.wait_for("Forwarding ");
        // After the start line's own text: `--debug` prints the mock API's
        // 127.0.0.1 address before it.
        let rest = err
            .split("Forwarding 127.0.0.1:")
            .nth(1)
            .expect("an IPv4 address");
        rest.split(|c: char| !c.is_ascii_digit())
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }

    /// Waits for the exit code, then for stderr to close.
    fn finish(mut self) -> (i32, String) {
        let deadline = Instant::now() + WAIT;
        let code = loop {
            if let Some(s) = self.child.try_wait().unwrap() {
                break s.code().unwrap_or(-1);
            }
            if Instant::now() > deadline {
                let _ = self.child.kill();
                panic!("port-forward did not exit; stderr: {}", self.stderr());
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        // The process has exited, so its stderr reaches its end.
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
        let err = self.err.lock().unwrap().clone();
        (code, err)
    }
}

/// Kills a port-forward a failed assertion left running, so a failing test
/// does not leave it listening after the test binary exits.
impl Drop for Forward {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_permanent_refusal_ends_the_port_forward_with_its_class() {
    let refused = |status: u16, code: &str, message: &str| envelope(status, code, message, None);
    let cases: Vec<(&str, Option<ResponseTemplate>, i32, &str)> = vec![
        (
            "a revoked key",
            Some(refused(401, "UNAUTHENTICATED", "Invalid API key.")),
            2,
            "Not signed in (2)",
        ),
        (
            "a sandbox no longer active",
            Some(refused(
                400,
                "FAILED_PRECONDITION",
                "The sandbox is not active.",
            )),
            13,
            "Not ready or stopped (13)",
        ),
        (
            "a key that lost its workspace",
            Some(refused(
                403,
                "PERMISSION_DENIED",
                "This key no longer has access to its workspace.",
            )),
            19,
            "Rejected request (19)",
        ),
        (
            "a session without sandbox:write",
            None,
            10,
            "Missing scope (10)",
        ),
    ];
    for (case, mint, want, class) in cases {
        let broker = FakeBroker::start();
        let server = MockServer::start().await;
        mount_sandbox(&server, ID, sandbox(ID, "box", "active", running())).await;
        let session = Mock::given(method("POST")).and(path(format!("/v3/sandboxes/{ID}/session")));
        match mint {
            Some(answer) => session.respond_with(answer).mount(&server).await,
            None => {
                session
                    .respond_with(session_answer(&broker))
                    .mount(&server)
                    .await;
                broker.push_port(PortScript::refuse(broker_status(
                    Code::PermissionDenied,
                    "permission_denied",
                    "the session lacks sandbox:write",
                    &[],
                )));
            }
        }
        let run = Run::new(&server);
        let fwd = Forward::start(&run);
        let port = fwd.port();
        let _ = TcpStream::connect(("127.0.0.1", port));
        // The listener closes as the refusal is said, before the process
        // exits.
        fwd.wait_for(class);
        assert!(
            TcpStream::connect(("127.0.0.1", port)).is_err(),
            "{case}: still listening"
        );
        let (code, err) = fwd.finish();
        assert_eq!(code, want, "{case}: {err}");
        assert!(
            class_line(&err).is_some_and(|l| l.contains(class)),
            "{case}: {err}"
        );
    }
}

/// A mint refused 400 `FAILED_PRECONDITION` because the replica's instance
/// stopped, while the sandbox still reads active, keeps the route's message
/// and says the command already exited and how to read its output: `exec`
/// and both directions of `cp` exit 255 with the Not ready or stopped (13)
/// line. `port-forward` reads the sandbox before its start line, so it exits
/// 13 with the same next step, before it starts and before any mint.
/// `shell`, which needs a terminal, is in `sandbox_terminal.rs`.
#[tokio::test(flavor = "multi_thread")]
async fn a_stopped_instance_on_an_active_sandbox_names_the_logs_command() {
    let line = "hedra-cli: Not ready or stopped (13): The instance at this replica has stopped. \
                `box`'s command already exited. Run `hedra-cli sandbox logs box` for its output.";
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
    std::fs::write(run.home.path().join("a.txt"), b"hello").unwrap();
    let verbs: [&[&str]; 3] = [
        &["sandbox", "exec", "box", "-n", "--", "true"],
        &["sandbox", "cp", "a.txt", "box:/work/a.txt"],
        &["sandbox", "cp", "box:/work/a.txt", "out.txt"],
    ];
    for args in verbs {
        let out = run.run(args);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(255), "{args:?}: {err}");
        assert_eq!(class_line(&err), Some(line), "{args:?}: {err}");
    }

    let mints_before = mints(&server, ID).await;
    let out = wait(run.spawn(&["sandbox", "port-forward", "box", "0:8888"]), WAIT);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(13), "{err}");
    assert_eq!(
        class_line(&err),
        Some(
            "hedra-cli: Not ready or stopped (13): `box`'s command already exited. \
             Run `hedra-cli sandbox logs box` for its output."
        ),
        "{err}"
    );
    assert!(!err.contains("Forwarding"), "{err}");
    assert_eq!(mints(&server, ID).await, mints_before);
}

/// `port-forward` on a sandbox that is stopping, ended, or stopping because
/// its lease was refused exits 13 before its start line, with the state and the next step,
/// and mints nothing; a name and an id both read the sandbox once. The bind
/// still comes first, so a taken port costs no request.
#[tokio::test(flavor = "multi_thread")]
async fn port_forward_on_a_stopped_sandbox_exits_13_before_it_starts() {
    let stopped = |status: &str, end_reason: &str| {
        let mut s = sandbox(ID, "box", status, serde_json::json!([{"replica": 0, "status": "stopped"}]));
        s["end_reason"] = end_reason.into();
        s
    };
    for (read, target, line) in [
        (
            stopped("stopping", "caller_stopped"),
            "box",
            "`box` is stopping (caller_stopped). Create another, or run `hedra-cli sandbox ls` \
             for the running ones.",
        ),
        (
            stopped("ended", "exited"),
            ID,
            "`sandbox_3f2504e0-4f89-41d3-9a0c-0305e82c3301` is ended (exited). Run \
             `hedra-cli sandbox logs sandbox_3f2504e0-4f89-41d3-9a0c-0305e82c3301` for its output.",
        ),
        (
            stopped("stopping", "unpaid"),
            "box",
            "`box` is stopping (unpaid). Add funds to the workspace, then create another; \
             `hedra-cli sandbox ls` lists the running ones.",
        ),
    ] {
        let server = MockServer::start().await;
        mount_sandbox(&server, "box", read.clone()).await;
        mount_sandbox(&server, ID, read).await;
        let out = wait(
            Run::new(&server).spawn(&["sandbox", "port-forward", target, "0:8888"]),
            WAIT,
        );
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(13), "{target}: {err}");
        assert_eq!(
            class_line(&err),
            Some(format!("hedra-cli: Not ready or stopped (13): {line}").as_str()),
            "{err}"
        );
        assert!(!err.contains("Forwarding"), "{err}");
        assert_eq!(mints(&server, ID).await, 0);
        assert_eq!(
            requests_to(&server, "GET", &format!("/v3/sandboxes/{target}"))
                .await
                .len(),
            1
        );
    }
}

/// An id whose first read fails with a status a retry may get past still
/// starts: each connection's mint finds out.
#[tokio::test(flavor = "multi_thread")]
async fn port_forward_starts_when_its_first_read_is_unavailable() {
    let broker = FakeBroker::start();
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}")))
        .respond_with(envelope(503, "UNAVAILABLE", "busy", Some(0)))
        .mount(&server)
        .await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let _ = fwd.port();
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

/// A sandbox that may yet run starts the port-forward, and Ctrl-C ends it
/// with 0: one still being created, and an active one whose lease was
/// refused (`unpaid`), which runs until the refusal's grace ends and keeps
/// running once a lease is paid.
#[tokio::test(flavor = "multi_thread")]
async fn port_forward_starts_on_a_sandbox_that_may_yet_run() {
    let mut unpaid = sandbox(
        ID,
        "box",
        "active",
        serde_json::json!([{"replica": 0, "status": "running"}]),
    );
    unpaid["end_reason"] = "unpaid".into();
    for read in [sandbox(ID, "box", "creating", serde_json::json!([])), unpaid] {
        let server = MockServer::start().await;
        mount_sandbox(&server, ID, read.clone()).await;
        let run = Run::new(&server);
        let fwd = Forward::start(&run);
        let _ = fwd.port();
        signal(&fwd.child, libc::SIGINT);
        let (code, err) = fwd.finish();
        assert_eq!(code, 0, "{read}: {err}");
        assert!(class_line(&err).is_none(), "{read}: {err}");
    }
}

/// A call that fails for want of a scope after its admission, as when the
/// key loses `sandboxes:write` mid-session, ends the port-forward with
/// Missing scope (10) and closes the listener, as a refusal at open does.
/// With `--debug`, the broker's status is printed as `exec` prints it.
#[tokio::test(flavor = "multi_thread")]
async fn a_scope_lost_after_admission_ends_the_port_forward() {
    for debug in [false, true] {
        let broker = FakeBroker::start();
        broker.push_port(PortScript::admit([
            PortStep::AwaitOpen,
            PortStep::Fail(denied_naming_no_scope()),
        ]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let mut args = vec!["sandbox", "port-forward", ID, "0:8888"];
        if debug {
            args.push("--debug");
        }
        let fwd = Forward::start_with(&run, &args);
        let port = fwd.port();
        let _conn = TcpStream::connect(("127.0.0.1", port)).unwrap();
        fwd.wait_for("Missing scope (10)");
        assert!(
            TcpStream::connect(("127.0.0.1", port)).is_err(),
            "still listening"
        );
        let (code, err) = fwd.finish();
        assert_eq!(code, 10, "{err}");
        let line = class_line(&err).unwrap_or_default();
        assert!(line.contains("`sandboxes:write`"), "{err}");
        let ended = "< port_forward ended: code PermissionDenied, reason permission_denied";
        assert_eq!(err.contains(ended), debug, "debug {debug}: {err}");
    }
}

/// A forwarded connection the broker ends `rate_limited` says, after the
/// broker's message, that port-forward cannot slow the sandbox's server.
#[tokio::test(flavor = "multi_thread")]
async fn a_rate_limited_connection_says_to_copy_large_files_with_cp() {
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Fail(broker_status(
            Code::Unavailable,
            "rate_limited",
            "client fell behind the sandbox's output; reconnect",
            &[],
        )),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let _conn = TcpStream::connect(("127.0.0.1", fwd.port())).unwrap();
    let err = fwd.wait_for("with `hedra-cli sandbox cp`.");
    assert!(
        err.contains(
            "a forwarded connection was reset: client fell behind the sandbox's output; \
             reconnect. `port-forward` cannot slow the sandbox's server; copy large files with \
             `hedra-cli sandbox cp`."
        ),
        "{err}"
    );
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

/// A local client that connects and closes having sent nothing, as `nc -z`
/// or a browser's preconnect does, says nothing about the sandbox's server.
#[tokio::test(flavor = "multi_thread")]
async fn a_probe_that_connects_and_closes_says_nothing() {
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([PortStep::AwaitEof, PortStep::Eof]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let conn = TcpStream::connect(("127.0.0.1", fwd.port())).unwrap();
    conn.shutdown(std::net::Shutdown::Write).unwrap();
    broker.wait_for(WAIT, |c| c.len() == 1 && c[0].port_eof && c[0].finished);
    std::thread::sleep(Duration::from_millis(300));
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(!err.contains("nothing is listening"), "{err}");
}

/// The refusal is printed when the listener closes, so a Ctrl-C while an
/// open connection finishes still says why.
#[tokio::test(flavor = "multi_thread")]
async fn a_permanent_refusal_is_said_while_open_connections_finish() {
    let broker = FakeBroker::start();
    broker.push_port(PortScript::hold());
    broker.push_port(PortScript::refuse(broker_status(
        Code::PermissionDenied,
        "permission_denied",
        "the session lacks sandbox:write",
        &[],
    )));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let _open = TcpStream::connect(("127.0.0.1", port)).unwrap();
    broker.wait_for(WAIT, |c| c.len() == 1 && c[0].admitted);
    let _ = TcpStream::connect(("127.0.0.1", port));
    fwd.wait_for("Missing scope (10)");
    assert!(
        TcpStream::connect(("127.0.0.1", port)).is_err(),
        "still listening"
    );
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 10, "{err}");
    assert_eq!(err.matches("hedra-cli: ").count(), 1, "{err}");
}

#[tokio::test(flavor = "multi_thread")]
async fn ctrl_c_ends_a_port_forward_with_0() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let err = fwd.stderr();
    assert!(
        err.contains(&format!("Forwarding 127.0.0.1:{port}")),
        "{err}"
    );
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(class_line(&err).is_none(), "{err}");
}

/// A local file that cannot be read cancels the copy in: the call is reset,
/// so tar in the sandbox never reads a clean end of its input.
#[tokio::test(flavor = "multi_thread")]
async fn a_local_read_error_resets_the_copy_in() {
    use std::os::unix::fs::PermissionsExt;
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"dir\n".to_vec()),
        Step::Count,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let tree = run.home.path().join("tree");
    std::fs::create_dir(&tree).unwrap();
    std::fs::write(tree.join("a"), b"readable").unwrap();
    std::fs::write(tree.join("secret"), b"x").unwrap();
    std::fs::set_permissions(tree.join("secret"), std::fs::Permissions::from_mode(0o000)).unwrap();
    let dst = format!("{ID}:/work");
    let out = run.run(&["sandbox", "cp", "tree", &dst]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    assert!(
        class_line(&err).is_some_and(
            |l| l.starts_with("hedra-cli: Rejected request (19):") && l.contains("secret")
        ),
        "{err}"
    );
    let calls = broker.wait_for(WAIT, |c| c[0].reset.is_some() || c[0].half_closed);
    assert!(
        calls[0].reset.is_some() && !calls[0].half_closed && !calls[0].stdin_eof,
        "the copy in was not reset: {:?}",
        calls[0]
    );
}

/// A walk failing just after a whole chunk's worth of entries sends the
/// sandbox no end blocks and no `stdin_eof`, and resets the call.
#[tokio::test(flavor = "multi_thread")]
async fn a_walk_failing_at_a_chunk_boundary_sends_no_end_of_archive() {
    use std::os::unix::fs::PermissionsExt;
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"dir\n".to_vec()),
        Step::Count,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let tree = run.home.path().join("tree");
    std::fs::create_dir(&tree).unwrap();
    // With `tree/`'s and `a`'s headers and two end blocks: 32 KiB, one frame.
    std::fs::write(tree.join("a"), vec![7u8; 30720]).unwrap();
    std::fs::write(tree.join("b"), b"x").unwrap();
    std::fs::set_permissions(tree.join("b"), std::fs::Permissions::from_mode(0o000)).unwrap();
    let dst = format!("{ID}:/work");
    let out = run.run(&["sandbox", "cp", "tree", &dst]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    let calls = broker.wait_for(WAIT, |c| c[0].reset.is_some() || c[0].half_closed);
    let c = &calls[0];
    assert!(
        c.reset.is_some() && !c.half_closed && !c.stdin_eof,
        "the copy in was not reset: {c:?}"
    );
    assert!(
        c.stdin.len() < 1024 || c.stdin[c.stdin.len() - 1024..].iter().any(|b| *b != 0),
        "the sandbox got the end blocks ({} bytes)",
        c.stdin.len()
    );
}

/// Ctrl-C during a copy out removes the parents it created.
#[tokio::test(flavor = "multi_thread")]
async fn ctrl_c_during_a_copy_out_removes_the_parents_it_made() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let src = format!("{ID}:/work/tree");
    let child = run.spawn(&["sandbox", "cp", &src, "a/b/out"]);
    broker.wait_for(WAIT, |c| c.len() == 1 && c[0].start.is_some());
    assert!(run.home.path().join("a/b").is_dir());
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(130), "{}", stderr(&out));
    assert!(!run.home.path().join("a").exists());
    assert!(!stderr(&out).contains("partial"), "{}", stderr(&out));
}

/// A second signal while `cp` waits for a stalled extractor that wrote
/// nothing removes the parents it made, and says the modes it opened up may
/// not have been put back.
#[tokio::test(flavor = "multi_thread")]
async fn a_second_signal_before_any_write_still_says_modes_may_stay_open() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let src = format!("{ID}:/work/tree");
    let child = run
        .command(&["sandbox", "cp", &src, "a/b/out"])
        .env("HEDRA_CLI_TEST_CP_STALL_EXTRACTOR", "1")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    broker.wait_for(WAIT, |c| c.len() == 1 && c[0].start.is_some());
    signal(&child, libc::SIGINT);
    std::thread::sleep(Duration::from_millis(500));
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(130), "{err}");
    assert!(!run.home.path().join("a").exists(), "{err}");
    assert!(
        err.contains("The modes of directories the copy opened up may not have been put back."),
        "{err}"
    );
    assert!(!err.contains("partial"), "{err}");
}

/// The first entry's header and body, without the end of the archive.
fn first_entry_only(name: &str) -> Vec<u8> {
    one_entry(name)[..1024].to_vec()
}

/// Ctrl-C once a copy out has written something leaves it, with the parents
/// made for it, and says where.
#[tokio::test(flavor = "multi_thread")]
async fn ctrl_c_after_a_copy_out_wrote_something_leaves_it_and_says_so() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(first_entry_only("a.txt")),
        Step::Hold,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let src = format!("{ID}:/work/a.txt");
    let child = run.spawn(&["sandbox", "cp", &src, "a/b/out"]);
    let written = run.home.path().join("a/b/out");
    let deadline = Instant::now() + WAIT;
    while std::fs::read(&written).ok().as_deref() != Some(b"x") {
        assert!(Instant::now() < deadline, "the entry was never written");
        std::thread::sleep(Duration::from_millis(20));
    }
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(130), "{err}");
    assert!(err.contains("A partial copy was left at a/b/out."), "{err}");
    assert!(!err.contains("modes"), "{err}");
    assert_eq!(std::fs::read(&written).unwrap(), b"x");
}

/// A copy out that fails once it has written something leaves it and names
/// it in the failure line; one whose remote `tar` fails is hedra-cli's
/// failure, 255, not tar's code.
#[tokio::test(flavor = "multi_thread")]
async fn a_copy_out_failing_part_way_leaves_what_it_wrote_and_names_it() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(first_entry_only("a.txt")),
        Step::Stderr(b"tar: a.txt: file changed as we read it\n".to_vec()),
        Step::Exit(1),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let src = format!("{ID}:/work/a.txt");
    let out = run.run(&["sandbox", "cp", &src, "a/b/out"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    let line = class_line(&err).unwrap_or_default();
    assert!(
        line.starts_with("hedra-cli: Rejected request (19): ")
            && line.contains("tar in the sandbox exited 1")
            && line.ends_with("; a partial copy was left at a/b/out"),
        "{err}"
    );
    assert!(err.contains("file changed as we read it"), "{err}");
    assert_eq!(
        std::fs::read(run.home.path().join("a/b/out")).unwrap(),
        b"x"
    );
}

/// `cp` takes a sandbox by name, as its help's examples do.
#[tokio::test(flavor = "multi_thread")]
async fn cp_takes_a_sandbox_by_name() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"new\n".to_vec()),
        Step::Count,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    mount_sandbox(&server, "box", sandbox(ID, "box", "active", running())).await;
    let run = Run::new(&server);
    std::fs::write(run.home.path().join("a.txt"), b"hello").unwrap();
    let out = run.run(&["sandbox", "cp", "a.txt", "box:/work/a.txt"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(
        requests_to(&server, "GET", "/v3/sandboxes/box").await.len(),
        1
    );
    let calls = broker.calls();
    assert!(calls[0].stdin_eof, "{calls:#?}");
    assert_eq!(&calls[0].stdin[..5], b"a.txt");
    assert_eq!(&calls[0].stdin[512..517], b"hello");
}

/// A copy out into a directory that was already there names it as the
/// directory part of the copy was written into, not as a partial copy: on
/// Ctrl-C, and on a failure part way, where the user's own file stays; also
/// when the destination is a symlink to the user's directory.
#[tokio::test(flavor = "multi_thread")]
async fn a_copy_out_into_an_existing_directory_says_part_was_written_into_it() {
    for via_link in [false, true] {
        for ctrl_c in [true, false] {
            let broker = FakeBroker::start();
            let last = if ctrl_c { Step::Hold } else { Step::Exit(2) };
            broker.push_exec(ExecScript::admit([
                Step::Start,
                Step::Stdout(first_entry_only("./a.txt")),
                last,
            ]));
            let server = running_sandbox(&broker).await;
            let run = Run::new(&server);
            let proj = run.home.path().join("proj");
            std::fs::create_dir(&proj).unwrap();
            std::fs::write(proj.join("notes.txt"), b"mine").unwrap();
            let dst = if via_link {
                std::os::unix::fs::symlink("proj", run.home.path().join("link")).unwrap();
                "link"
            } else {
                "proj"
            };
            let src = format!("{ID}:/work/tree/.");
            let child = run.spawn(&["sandbox", "cp", &src, dst]);
            let written = proj.join("a.txt");
            let deadline = Instant::now() + WAIT;
            while ctrl_c && std::fs::read(&written).ok().as_deref() != Some(b"x") {
                assert!(Instant::now() < deadline, "the entry was never written");
                std::thread::sleep(Duration::from_millis(20));
            }
            if ctrl_c {
                signal(&child, libc::SIGINT);
            }
            let out = wait(child, WAIT);
            let err = stderr(&out);
            let at = format!("link {via_link}, ctrl-c {ctrl_c}: {err}");
            assert!(!err.contains("partial"), "{at}");
            if ctrl_c {
                assert_eq!(out.status.code(), Some(130), "{at}");
                let into =
                    format!("Part of the copy was written into {dst}, which already existed.");
                assert!(err.contains(&into), "{at}");
            } else {
                assert_eq!(out.status.code(), Some(255), "{at}");
                let line = class_line(&err).unwrap_or_default();
                let into =
                    format!("; part of the copy was written into {dst}, which already existed");
                assert!(
                    line.contains("tar in the sandbox exited 2") && line.ends_with(&into),
                    "{at}"
                );
            }
            assert_eq!(std::fs::read(&written).unwrap(), b"x");
            assert_eq!(std::fs::read(proj.join("notes.txt")).unwrap(), b"mine");
        }
    }
}

/// A copy out that fails after writing into the user's read-only directory
/// puts its mode back: `cp box:/work/tree .` over an existing `./tree/ro`
/// at 0555, with an entry refused after `tree/ro/`.
#[tokio::test(flavor = "multi_thread")]
async fn a_failed_copy_out_puts_back_a_read_only_directory_it_wrote_into() {
    use std::os::unix::fs::PermissionsExt;
    let broker = FakeBroker::start();
    let mut stream = header("tree/", b'5', 0o755, 0);
    stream.extend(header("tree/ro/", b'5', 0o555, 0));
    stream.extend(&one_entry("tree/ro/f.txt")[..1024]);
    stream.extend(one_entry("/etc/evil"));
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(stream),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let ro = run.home.path().join("tree/ro");
    std::fs::create_dir_all(&ro).unwrap();
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();
    let src = format!("{ID}:/work/tree");
    let out = run.run(&["sandbox", "cp", &src, "."]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    assert!(
        class_line(&err).is_some_and(|l| l.contains("absolute")
            && l.ends_with("; part of the copy was written into ./tree, which already existed")),
        "{err}"
    );
    let mode = std::fs::metadata(&ro).unwrap().permissions().mode() & 0o7777;
    assert_eq!(mode, 0o555, "{err}");
    assert_eq!(std::fs::read(ro.join("f.txt")).unwrap(), b"x");
    let _ = std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o755));
}

/// A copy over more read-only directories than the default limit on open
/// files lands: opening each one up for the copy holds no descriptor.
#[tokio::test(flavor = "multi_thread")]
async fn a_copy_over_many_read_only_directories_holds_no_descriptor_for_each() {
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::process::CommandExt;
    const DIRS: usize = 400;
    let mut stream = header("tree/", b'5', 0o755, 0);
    for i in 0..DIRS {
        stream.extend(header(&format!("tree/d{i}/"), b'5', 0o555, 0));
        stream.extend(&one_entry(&format!("tree/d{i}/f"))[..1024]);
    }
    stream.resize(stream.len().div_ceil(10240) * 10240 + 10240, 0);
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(stream),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    for i in 0..DIRS {
        let d = run.home.path().join(format!("tree/d{i}"));
        std::fs::create_dir_all(&d).unwrap();
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o555)).unwrap();
    }
    let src = format!("{ID}:/work/tree");
    let mut cmd = run.command(&["sandbox", "cp", &src, "."]);
    // SAFETY: only async-signal-safe calls between fork and exec.
    unsafe {
        cmd.pre_exec(|| {
            // Only the soft limit: an unprivileged process cannot raise
            // the hard one, which Linux runners keep finite.
            let mut limit = libc::rlimit {
                rlim_cur: 0,
                rlim_max: 0,
            };
            if libc::getrlimit(libc::RLIMIT_NOFILE, &mut limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            limit.rlim_cur = 256;
            if libc::setrlimit(libc::RLIMIT_NOFILE, &limit) != 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let out = cmd.output().unwrap();
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    for i in 0..DIRS {
        let d = run.home.path().join(format!("tree/d{i}"));
        assert_eq!(std::fs::read(d.join("f")).unwrap(), b"x", "d{i}");
        let mode = std::fs::metadata(&d).unwrap().permissions().mode() & 0o7777;
        assert_eq!(mode, 0o555, "d{i}");
        let _ = std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o755));
    }
}

/// A copy out cut part way over a file of the user's names what it left as
/// a partial copy: the file was replaced, so nothing of the user's is in it.
#[tokio::test(flavor = "multi_thread")]
async fn a_copy_out_cut_over_an_existing_file_says_a_partial_copy_was_left() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(first_entry_only("a.txt")),
        Step::Exit(2),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    std::fs::write(run.home.path().join("out"), b"mine").unwrap();
    let src = format!("{ID}:/work/a.txt");
    let out = run.run(&["sandbox", "cp", &src, "out"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    let line = class_line(&err).unwrap_or_default();
    assert!(
        line.contains("tar in the sandbox exited 2")
            && line.ends_with("; a partial copy was left at ./out"),
        "{err}"
    );
    assert!(!err.contains("already existed"), "{err}");
}

/// Ctrl-C while a copy out writes a file into the user's read-only
/// directory stops the write, puts the directory's mode back before `cp`
/// exits 130, and names the directory part of the copy was written into.
#[tokio::test(flavor = "multi_thread")]
async fn ctrl_c_mid_file_puts_back_a_read_only_directory_before_exiting() {
    use std::os::unix::fs::PermissionsExt;
    const BIG: u64 = 256 << 20;
    let broker = FakeBroker::start();
    let mut stream = header("tree/", b'5', 0o755, 0);
    stream.extend(header("tree/ro/", b'5', 0o555, 0));
    stream.extend(header("tree/ro/big", b'0', 0o644, BIG));
    let chunk = vec![7u8; 1 << 20];
    let mut steps = vec![Step::Start, Step::Stdout(stream)];
    for _ in 0..64 {
        steps.push(Step::Stdout(chunk.clone()));
    }
    steps.push(Step::Hold);
    broker.push_exec(ExecScript::admit(steps));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let ro = run.home.path().join("tree/ro");
    std::fs::create_dir_all(&ro).unwrap();
    std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o555)).unwrap();
    let src = format!("{ID}:/work/tree");
    let child = run.spawn(&["sandbox", "cp", &src, "."]);
    let big = ro.join("big");
    let deadline = Instant::now() + WAIT;
    while std::fs::metadata(&big).map_or(0, |m| m.len()) == 0 {
        assert!(Instant::now() < deadline, "the file was never begun");
        std::thread::sleep(Duration::from_millis(5));
    }
    signal(&child, libc::SIGINT);
    let signalled = Instant::now();
    let out = wait(child, WAIT);
    // Well inside the 5 s the exit waits for the extractor at most: it
    // stops at once and says so.
    let took = signalled.elapsed();
    let err = stderr(&out);
    let mode = std::fs::metadata(&ro).unwrap().permissions().mode() & 0o7777;
    let _ = std::fs::set_permissions(&ro, std::fs::Permissions::from_mode(0o755));
    assert_eq!(out.status.code(), Some(130), "{err}");
    assert_eq!(mode, 0o555, "{err}");
    assert!(took < Duration::from_secs(2), "{took:?}: {err}");
    assert!(
        err.contains("Part of the copy was written into ./tree, which already existed."),
        "{err}"
    );
    assert!(!err.contains("modes"), "{err}");
}

/// A second signal while `cp` waits for an extractor held up by a stalled
/// destination ends the wait at once: `cp` exits by the first signal and
/// says the modes it opened up may not have been put back.
#[tokio::test(flavor = "multi_thread")]
async fn a_second_signal_ends_the_wait_for_a_stalled_extractor() {
    for (first, second, code) in [
        (libc::SIGINT, libc::SIGINT, 130),
        (libc::SIGTERM, libc::SIGINT, 143),
    ] {
        let broker = FakeBroker::start();
        let mut stream = header("tree/", b'5', 0o755, 0);
        stream.extend(first_entry_only("tree/a.txt"));
        broker.push_exec(ExecScript::admit([
            Step::Start,
            Step::Stdout(stream),
            Step::Hold,
        ]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        std::fs::create_dir(run.home.path().join("tree")).unwrap();
        let src = format!("{ID}:/work/tree");
        let child = run
            .command(&["sandbox", "cp", &src, "."])
            .env("HEDRA_CLI_TEST_CP_STALL_EXTRACTOR", "1")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let written = run.home.path().join("tree/a.txt");
        let deadline = Instant::now() + WAIT;
        while std::fs::read(&written).ok().as_deref() != Some(b"x") {
            assert!(Instant::now() < deadline, "the entry was never written");
            std::thread::sleep(Duration::from_millis(20));
        }
        signal(&child, first);
        // Into the wait, which would last the 5 s grace.
        std::thread::sleep(Duration::from_millis(500));
        signal(&child, second);
        let signalled = Instant::now();
        let out = wait(child, WAIT);
        let took = signalled.elapsed();
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(code), "{err}");
        assert!(took < Duration::from_secs(2), "{took:?}: {err}");
        assert!(
            err.contains(
                "Part of the copy was written into ./tree, which already existed; the modes of \
                 directories the copy opened up may not have been put back."
            ),
            "{err}"
        );
    }
}

/// Ctrl-C after a copy out let a write through that never landed, here a
/// top directory refused over a file of the user's, says nothing was left:
/// the file is the user's, untouched.
#[tokio::test(flavor = "multi_thread")]
async fn ctrl_c_after_a_write_that_never_landed_names_nothing() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(header("tree/", b'5', 0o755, 0)),
        Step::Hold,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    std::fs::write(run.home.path().join("tree"), b"mine").unwrap();
    let src = format!("{ID}:/work/tree");
    let child = run.spawn(&["sandbox", "cp", &src, "."]);
    broker.wait_for(WAIT, |c| c.len() == 1 && c[0].start.is_some());
    // Time for the extractor to meet the header and refuse it.
    std::thread::sleep(Duration::from_millis(500));
    signal(&child, libc::SIGINT);
    let out = wait(child, WAIT);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(130), "{err}");
    assert!(
        !err.contains("partial") && !err.contains("Part of"),
        "{err}"
    );
    assert_eq!(
        std::fs::read(run.home.path().join("tree")).unwrap(),
        b"mine"
    );
}

/// A copy out over a directory of the user's that its owner can read but
/// not search, 0444, writes into it and gives it the archive's mode.
#[tokio::test(flavor = "multi_thread")]
async fn a_copy_out_writes_into_a_directory_its_owner_cannot_search() {
    use std::os::unix::fs::PermissionsExt;
    let broker = FakeBroker::start();
    let mut stream = header("tree/", b'5', 0o755, 0);
    stream.extend(header("tree/a/", b'5', 0o755, 0));
    stream.extend(one_entry("tree/a/b"));
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(stream),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let a = run.home.path().join("tree/a");
    std::fs::create_dir_all(&a).unwrap();
    std::fs::set_permissions(&a, std::fs::Permissions::from_mode(0o444)).unwrap();
    let src = format!("{ID}:/work/tree");
    let out = run.run(&["sandbox", "cp", &src, "."]);
    let err = stderr(&out);
    let mode = std::fs::metadata(&a).unwrap().permissions().mode() & 0o7777;
    let _ = std::fs::set_permissions(&a, std::fs::Permissions::from_mode(0o755));
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(mode, 0o755, "{err}");
    assert_eq!(std::fs::read(a.join("b")).unwrap(), b"x");
}

/// GNU tar exiting 1 on a whole stream whose totals match what arrived
/// still fails the copy with 255, but says every entry arrived and why tar
/// failed, and names no partial copy.
#[tokio::test(flavor = "multi_thread")]
async fn tar_exiting_1_on_a_whole_stream_says_every_entry_arrived() {
    // With tar's message relayed above the line, and with none to point at.
    for said in [true, false] {
        let broker = FakeBroker::start();
        let stream = one_entry("a.txt");
        let message = if said {
            "tar: a.txt: file changed as we read it\n"
        } else {
            ""
        };
        let tar_err = format!(
            "{message}[hedra-cp] totals\nTotal bytes written: {} (10KiB, 1.0MiB/s)\n",
            stream.len()
        );
        broker.push_exec(ExecScript::admit([
            Step::Start,
            Step::Stdout(stream),
            Step::Stderr(tar_err.into_bytes()),
            Step::Exit(1),
        ]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let src = format!("{ID}:/work/a.txt");
        let out = run.run(&["sandbox", "cp", &src, "out"]);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(255), "{err}");
        let line = class_line(&err).unwrap_or_default();
        let above = if said { " (its message is above)" } else { "" };
        assert!(
            line.ends_with(&format!(
                "tar in the sandbox exited 1{above}; every entry it sent arrived at ./out"
            )),
            "{err}"
        );
        assert!(!err.contains("partial"), "{err}");
        assert_eq!(err.contains("file changed as we read it"), said, "{err}");
        assert_eq!(std::fs::read(run.home.path().join("out")).unwrap(), b"x");
    }
}

/// A copy out to a sandbox that cannot be found removes the parents it
/// created for the copy.
#[tokio::test(flavor = "multi_thread")]
async fn a_copy_out_from_a_missing_sandbox_removes_the_parents_it_made() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    Mock::given(method("GET"))
        .and(path("/v3/sandboxes/gone"))
        .respond_with(envelope(404, "NOT_FOUND", "Sandbox not found.", None))
        .mount(&server)
        .await;
    let run = Run::new(&server);
    let out = run.run(&["sandbox", "cp", "gone:/x", "a/b/out"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    assert!(class_line(&err).is_some(), "{err}");
    assert!(!run.home.path().join("a").exists(), "{err}");
    assert!(broker.calls().is_empty());
}

/// A mint answered 503 refuses the one connection it was for; the listener
/// serves the next.
#[tokio::test(flavor = "multi_thread")]
async fn a_mint_failing_with_503_refuses_only_its_connection() {
    let broker = FakeBroker::start();
    broker.push_port(PortScript::hold());
    let server = MockServer::start().await;
    mount_sandbox(&server, ID, sandbox(ID, "box", "active", running())).await;
    // Retried after 30 s, then given up on: the mint's minute is spent.
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{ID}/session")))
        .respond_with(envelope(503, "UNAVAILABLE", "Try again.", Some(30)))
        .up_to_n_times(2)
        .mount(&server)
        .await;
    mount_session(&server, ID, &broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let _first = TcpStream::connect(("127.0.0.1", port)).unwrap();
    fwd.wait_for_within(
        "a forwarded connection was refused",
        Duration::from_secs(90),
    );
    let _second = TcpStream::connect(("127.0.0.1", port)).expect("the listener closed");
    broker.wait_for(WAIT, |c| c.len() == 1 && c[0].admitted);
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(class_line(&err).is_none(), "{err}");
}

/// SIGHUP and SIGTERM end a port-forward with their codes, and cancel the
/// open calls.
#[tokio::test(flavor = "multi_thread")]
async fn hangup_and_terminate_cancel_the_calls_and_exit_with_their_codes() {
    for (sig, want) in [(libc::SIGHUP, 129), (libc::SIGTERM, 143)] {
        let broker = FakeBroker::start();
        broker.push_port(PortScript::hold());
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let fwd = Forward::start(&run);
        let port = fwd.port();
        let _open = TcpStream::connect(("127.0.0.1", port)).unwrap();
        broker.wait_for(WAIT, |c| c.len() == 1 && c[0].admitted);
        signal(&fwd.child, sig);
        let (code, err) = fwd.finish();
        assert_eq!(code, want, "{sig}: {err}");
        let calls = broker.wait_for(WAIT, |c| c[0].reset.is_some() || c[0].half_closed);
        assert!(
            calls[0].reset.is_some() && !calls[0].half_closed,
            "{sig}: the call was not cancelled: {:?}",
            calls[0]
        );
    }
}

/// Resetting one forwarded connection cancels only its call: the others on
/// the same broker connection keep carrying bytes, and a new one opens on it.
#[tokio::test(flavor = "multi_thread")]
async fn resetting_one_connection_leaves_the_others_on_the_channel_working() {
    use std::io::Write;
    let broker = FakeBroker::start();
    for _ in 0..3 {
        broker.push_port(PortScript::hold());
    }
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let first = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    broker.wait_for(WAIT, |c| c.len() == 1 && c[0].frames >= 1);
    let mut b = TcpStream::connect(("127.0.0.1", port)).unwrap();
    broker.wait_for(WAIT, |c| c.len() == 2 && c[1].frames >= 1);
    first.set_zero_linger().unwrap();
    drop(first);
    broker.wait_for(WAIT, |c| c[0].reset.is_some());
    // Past the wait after which a call whose reset did not go out has its
    // connection closed.
    std::thread::sleep(Duration::from_millis(600));
    b.write_all(b"still here").unwrap();
    broker.wait_for(WAIT, |c| c[1].frames >= 2);
    let mut c = TcpStream::connect(("127.0.0.1", port)).unwrap();
    broker.wait_for(WAIT, |c| c.len() == 3 && c[2].frames >= 1);
    c.write_all(b"new").unwrap();
    let calls = broker.wait_for(WAIT, |c| c[2].frames >= 2);
    assert_eq!(broker.connections(), 1);
    assert!(
        calls[1].reset.is_none() && !calls[1].client_gone,
        "{:?}",
        calls[1]
    );
    assert!(calls[2].reset.is_none(), "{:?}", calls[2]);
    assert!(!fwd.stderr().contains("hedra-cli: "), "{}", fwd.stderr());
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_socks_or_https_proxy_is_refused_before_any_request() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cp_src = format!("{ID}:/work/tree");
    for (proxy, line) in [
        ("socks5://127.0.0.1:1080", "a SOCKS proxy"),
        ("https://127.0.0.1:3128", "an https:// proxy"),
    ] {
        for (args, code) in [
            (vec!["sandbox", "cp", cp_src.as_str(), "out"], 255),
            (vec!["sandbox", "port-forward", ID, "0:80"], 19),
        ] {
            let out = run
                .command(&args)
                .env("HTTPS_PROXY", proxy)
                .output()
                .unwrap();
            let err = stderr(&out);
            assert_eq!(out.status.code(), Some(code), "{proxy} {args:?}: {err}");
            let want = format!("hedra-cli: Rejected request (19): {line}");
            assert!(
                class_line(&err).is_some_and(|l| l.starts_with(&want)),
                "{proxy} {args:?}: {err}"
            );
        }
    }
    assert_eq!(request_count(&server).await, 0);
    assert_eq!(broker.connections(), 0);
}

/// A proxy that answers every `CONNECT` 407, counting them.
fn refusing_proxy() -> (u16, Arc<std::sync::atomic::AtomicUsize>) {
    use std::io::Write;
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
    (port, tunnels)
}

/// A connection no retry can fix, here a proxy refusing the tunnel, closes
/// the listener at once and ends the port-forward Rejected request (19).
#[tokio::test(flavor = "multi_thread")]
async fn a_proxy_refusing_the_tunnel_ends_the_port_forward_with_19() {
    let (proxy, tunnels) = refusing_proxy();
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let mut cmd = run.command(&["sandbox", "port-forward", ID, "0:8888"]);
    cmd.env("HTTPS_PROXY", format!("http://127.0.0.1:{proxy}"));
    let fwd = Forward::from_command(cmd);
    let port = fwd.port();
    let _ = TcpStream::connect(("127.0.0.1", port));
    let (code, err) = fwd.finish();
    assert_eq!(code, 19, "{err}");
    assert_eq!(
        class_line(&err),
        Some(
            "hedra-cli: Rejected request (19): Could not connect to the sandbox broker: the \
             proxy refused the tunnel to the broker: HTTP/1.1 407 Proxy Authentication Required."
        ),
        "{err}"
    );
    assert!(
        TcpStream::connect(("127.0.0.1", port)).is_err(),
        "still listening"
    );
    assert_eq!(tunnels.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(broker.connections(), 0);
}

/// A broker that `NO_PROXY` bypasses is not said to be reached through the
/// configured proxy, by `cp` or by `port-forward`.
#[tokio::test(flavor = "multi_thread")]
async fn a_bypassed_proxy_is_not_named_in_a_retry_line() {
    use std::io::Write;
    let with_bypass = |cmd: &mut std::process::Command| {
        cmd.env("HTTPS_PROXY", "http://127.0.0.1:9")
            .env("NO_PROXY", "127.0.0.1");
    };

    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(Status::unavailable("restarting")));
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(one_entry("tree")),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let src = format!("{ID}:/work/tree");
    let mut cmd = run.command(&["sandbox", "cp", &src, "out"]);
    with_bypass(&mut cmd);
    let out = cmd.output().unwrap();
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert!(
        err.contains("Could not reach the sandbox broker; retrying."),
        "{err}"
    );
    assert!(!err.contains("proxy"), "{err}");

    let broker = FakeBroker::start();
    broker.push_port(PortScript::refuse(Status::unavailable("restarting")));
    broker.push_port(PortScript::admit([
        PortStep::AwaitBytes(2),
        PortStep::Data(b"ok".to_vec()),
        PortStep::Eof,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let mut cmd = run.command(&["sandbox", "port-forward", ID, "0:8888"]);
    with_bypass(&mut cmd);
    let fwd = Forward::from_command(cmd);
    let mut conn = TcpStream::connect(("127.0.0.1", fwd.port())).unwrap();
    conn.write_all(b"hi").unwrap();
    assert_eq!(read_all(&mut conn).unwrap(), b"ok");
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(
        err.contains("Could not reach the sandbox broker; retrying."),
        "{err}"
    );
    assert!(!err.contains("proxy"), "{err}");
}

/// The sandbox read that explains a failed open, or a copy cut off, still
/// answers a signal: a stalled read never holds `cp` past Ctrl-C or SIGTERM.
#[tokio::test(flavor = "multi_thread")]
async fn cp_answers_a_signal_while_it_reads_the_sandbox_after_a_failure() {
    let stalled_read = || {
        Mock::given(method("GET"))
            .and(path(format!("/v3/sandboxes/{ID}")))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(sandbox(ID, "box", "active", running()))
                    .set_delay(Duration::from_secs(120)),
            )
    };
    let src = format!("{ID}:/work/tree");

    // A mint refused 404 reads the sandbox to say why.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{ID}/session")))
        .respond_with(envelope(404, "NOT_FOUND", "Not Found", None))
        .mount(&server)
        .await;
    stalled_read().mount(&server).await;
    let run = Run::new(&server);
    let child = run.spawn(&["sandbox", "cp", &src, "out"]);
    wait_for_read(&server).await;
    signal(&child, libc::SIGINT);
    let started = Instant::now();
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(130), "{}", stderr(&out));
    assert!(started.elapsed() < Duration::from_secs(10));

    // A call cut off part way reads the sandbox to say whether it stopped.
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Close]));
    let server = MockServer::start().await;
    mount_session(&server, ID, &broker).await;
    stalled_read().mount(&server).await;
    let run = Run::new(&server);
    let child = run.spawn(&["sandbox", "cp", &src, "out"]);
    wait_for_read(&server).await;
    signal(&child, libc::SIGTERM);
    let started = Instant::now();
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(143), "{}", stderr(&out));
    assert!(started.elapsed() < Duration::from_secs(10));
}

/// Waits until `server` has seen a read of sandbox `ID`.
async fn wait_for_read(server: &MockServer) {
    let deadline = Instant::now() + WAIT;
    while requests_to(server, "GET", &format!("/v3/sandboxes/{ID}"))
        .await
        .is_empty()
    {
        assert!(Instant::now() < deadline, "the sandbox was never read");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_copy_refused_sandbox_disconnected_is_retried_and_lands() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(broker_status(
        Code::Unavailable,
        "sandbox_disconnected",
        "the sandbox's supervisor is not connected",
        &[],
    )));
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(one_entry("tree")),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cp_src = format!("{ID}:/work/tree");
    let out = run.run(&["sandbox", "cp", &cp_src, "out"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    assert_eq!(std::fs::read(run.home.path().join("out")).unwrap(), b"x");
    assert_eq!(broker.calls().len(), 2, "{err}");
}

// ------------------------------------------------- forwarded connections

/// Reads `conn` to its end, or to the error that ends it, within `WAIT`.
fn read_all(conn: &mut TcpStream) -> std::io::Result<Vec<u8>> {
    // macOS refuses the option on a socket already reset, and the read
    // then fails at once.
    let _ = conn.set_read_timeout(Some(WAIT));
    let mut got = Vec::new();
    conn.read_to_end(&mut got).map(|_| got)
}

/// A local port nothing listens on, for a forward whose two ports match.
fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

/// `port-forward <name> <port>` resolves the name and forwards that port to
/// the same port in the sandbox, carrying bytes both ways; the sandbox
/// server's `eof` ends the call and closes the local connection.
#[tokio::test(flavor = "multi_thread")]
async fn a_named_sandbox_and_one_port_forward_that_port_to_the_same_port() {
    use std::io::Write;
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([
        PortStep::AwaitBytes(4),
        PortStep::Data(b"pong".to_vec()),
        PortStep::Eof,
    ]));
    let server = running_sandbox(&broker).await;
    mount_sandbox(&server, "box", sandbox(ID, "box", "active", running())).await;
    let run = Run::new(&server);
    let port = free_port();
    let spec = port.to_string();
    let fwd = Forward::start_with(&run, &["sandbox", "port-forward", "box", &spec]);
    assert_eq!(fwd.port(), port);
    let err = fwd.stderr();
    assert!(err.contains(&format!("to port {port} in `box`")), "{err}");
    let mut conn = TcpStream::connect(("127.0.0.1", port)).unwrap();
    conn.write_all(b"ping").unwrap();
    assert_eq!(read_all(&mut conn).unwrap(), b"pong");
    let calls = broker.wait_for(WAIT, |c| c.len() == 1 && c[0].port_data == b"ping");
    assert_eq!(calls[0].port_open, Some(u32::from(port)));
    assert_eq!(
        requests_to(&server, "GET", "/v3/sandboxes/box").await.len(),
        1
    );
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

/// A client reaching the forward over IPv6, as `localhost` resolving to
/// `::1` does, is forwarded like one over IPv4.
#[tokio::test(flavor = "multi_thread")]
async fn a_client_connecting_over_ipv6_is_forwarded() {
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Data(b"v6".to_vec()),
        PortStep::Eof,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    if !fwd.stderr().contains(&format!("[::1]:{port}")) {
        // No IPv6 loopback here; the bind warning tests cover its absence.
        signal(&fwd.child, libc::SIGINT);
        let _ = fwd.finish();
        return;
    }
    let mut conn = TcpStream::connect(("::1", port)).unwrap();
    assert_eq!(read_all(&mut conn).unwrap(), b"v6");
    assert_eq!(broker.calls()[0].port_open, Some(8888));
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

/// At most 90 forwarded connections have a call open; the 91st waits for
/// one of them to end, then gets its own.
#[tokio::test(flavor = "multi_thread")]
async fn the_91st_connection_waits_for_one_to_end() {
    let broker = FakeBroker::start();
    for _ in 0..91 {
        broker.push_port(PortScript::hold());
    }
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let first = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .unwrap();
    let mut open = Vec::new();
    for _ in 1..90 {
        open.push(TcpStream::connect(("127.0.0.1", port)).unwrap());
    }
    broker.wait_for(WAIT, |c| {
        c.len() == 90 && c.iter().all(|c| c.port_open.is_some())
    });
    let _waiting = TcpStream::connect(("127.0.0.1", port)).unwrap();
    std::thread::sleep(Duration::from_millis(500));
    assert_eq!(broker.calls().len(), 90, "a call opened past the cap");
    first.set_zero_linger().unwrap();
    drop(first);
    broker.wait_for(WAIT, |c| c.len() == 91 && c[90].port_open.is_some());
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

/// A server that speaks first, as MySQL or SMTP do, is heard before the
/// client writes anything: `open` goes out on admission, not on the first
/// local byte.
#[tokio::test(flavor = "multi_thread")]
async fn a_server_that_speaks_first_is_heard_before_the_client_writes() {
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Data(b"220 ready\r\n".to_vec()),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let mut conn = TcpStream::connect(("127.0.0.1", fwd.port())).unwrap();
    conn.set_read_timeout(Some(WAIT)).unwrap();
    let mut greeting = [0u8; 11];
    conn.read_exact(&mut greeting).unwrap();
    assert_eq!(&greeting, b"220 ready\r\n");
    let calls = broker.calls();
    assert_eq!(calls[0].port_open, Some(8888));
    assert!(calls[0].port_data.is_empty(), "{:?}", calls[0]);
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

/// A local half-close sends `eof` and leaves the call open, so the reply
/// the server sends after reading its request still arrives.
#[tokio::test(flavor = "multi_thread")]
async fn a_local_half_close_sends_eof_and_the_reply_still_arrives() {
    use std::io::Write;
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([
        PortStep::AwaitEof,
        PortStep::Data(b"reply".to_vec()),
        PortStep::Eof,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let mut conn = TcpStream::connect(("127.0.0.1", fwd.port())).unwrap();
    conn.write_all(b"request").unwrap();
    conn.shutdown(std::net::Shutdown::Write).unwrap();
    assert_eq!(read_all(&mut conn).unwrap(), b"reply");
    let calls = broker.calls();
    assert!(
        calls[0].port_eof && calls[0].port_data == b"request" && calls[0].reset.is_none(),
        "{:?}",
        calls[0]
    );
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(class_line(&err).is_none(), "{err}");
}

/// A connection the sandbox closes before sending a byte says that nothing
/// may be listening on the sandbox port, and the listener carries on.
#[tokio::test(flavor = "multi_thread")]
async fn a_connection_closed_before_any_byte_says_nothing_is_listening() {
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([PortStep::AwaitOpen, PortStep::Eof]));
    broker.push_port(PortScript::hold());
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let mut conn = TcpStream::connect(("127.0.0.1", port)).unwrap();
    assert_eq!(read_all(&mut conn).unwrap(), b"");
    fwd.wait_for(
        "nothing is listening on 127.0.0.1:8888 in the sandbox (a server bound only to ::1 or \
         another address is not reached), or it closed the connection",
    );
    let _next = TcpStream::connect(("127.0.0.1", port)).expect("the listener closed");
    broker.wait_for(WAIT, |c| c.len() == 2 && c[1].admitted);
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(class_line(&err).is_none(), "{err}");
}

/// A connection whose local reader stalls is reset once its 8 MiB buffer
/// fills, over the real transport with its adaptive window, while another
/// connection on the same broker connection keeps receiving at the rate
/// its sandbox server sends.
/// Neither leaves 256 frames unread at the broker, which would end its
/// session `rate_limited`: port-forward never stops reading a stream.
#[tokio::test(flavor = "multi_thread")]
async fn a_stalled_connection_is_reset_while_another_keeps_its_throughput() {
    // Slow enough for a debug build on a shared runner to keep up with both
    // streams, so only a client that stops reading meets the broker's rule.
    const RATE: usize = 8 << 20;
    const LIVE: usize = 24 << 20;
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Flood {
            bytes: 512 << 20,
            per_sec: RATE,
        },
    ]));
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Flood {
            bytes: LIVE,
            per_sec: RATE,
        },
        PortStep::Eof,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let mut stalled = TcpStream::connect(("127.0.0.1", port)).unwrap();
    broker.wait_for(WAIT, |c| c.len() == 1 && c[0].port_open.is_some());
    let mut live = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let started = Instant::now();
    let got = read_all(&mut live).unwrap();
    let took = started.elapsed();
    assert_eq!(got.len(), LIVE);
    fwd.wait_for("the forwarded connection fell behind and was reset");
    // The stalled connection's local side sees the reset once it reads.
    let read = read_all(&mut stalled).map(|b| b.len());
    assert!(
        read.as_ref()
            .is_err_and(|e| e.kind() == std::io::ErrorKind::ConnectionReset),
        "the stalled connection was not reset: {read:?}"
    );
    // The live call ran its script to the end before the client let it go:
    // the client may reset it once it has read the end, which the fake
    // records as `reset` after `finished`, so `reset` alone says nothing.
    let calls = broker.wait_for(WAIT, |c| {
        (c[0].reset.is_some() || c[0].client_gone) && (c[1].finished || c[1].client_gone)
    });
    assert!(calls[1].finished && !calls[1].client_gone, "{:?}", calls[1]);
    assert!(
        !calls[0].rate_limited && !calls[1].rate_limited,
        "the broker ended a session rate_limited"
    );
    assert_eq!(broker.connections(), 1);
    // The stalled connection did not hold the live one back.
    assert!(took < Duration::from_secs(10), "{took:?}");
    assert_eq!(
        fwd.stderr().matches("fell behind").count(),
        1,
        "{}",
        fwd.stderr()
    );
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

/// `quota_exceeded` refuses only its own connection: the limit is said
/// once however many it refuses, the broker's message is printed escaped,
/// and the next connection is served.
#[tokio::test(flavor = "multi_thread")]
async fn quota_exceeded_refuses_only_its_connection_and_is_said_once() {
    let broker = FakeBroker::start();
    for _ in 0..2 {
        broker.push_port(PortScript::refuse(broker_status(
            Code::ResourceExhausted,
            "quota_exceeded",
            "\x1b]52;c;evil\x07too many sessions",
            &[],
        )));
    }
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Data(b"ok".to_vec()),
        PortStep::Eof,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    for n in 1..=2 {
        let mut refused = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let _ = read_all(&mut refused);
        broker.wait_for(WAIT, |c| c.len() == n);
    }
    let mut served = TcpStream::connect(("127.0.0.1", port)).expect("the listener closed");
    assert_eq!(read_all(&mut served).unwrap(), b"ok");
    let err = fwd.wait_for("too many sessions");
    signal(&fwd.child, libc::SIGINT);
    let (code, err_after) = fwd.finish();
    assert_eq!(code, 0, "{err_after}");
    assert_eq!(
        err_after
            .matches("The workspace is at its limit of open sandbox sessions")
            .count(),
        1,
        "{err_after}"
    );
    assert!(class_line(&err_after).is_none(), "{err_after}");
    assert!(
        !err.contains('\x1b') && err.contains("]52;c;evil"),
        "{err:?}"
    );
}

fn disconnected() -> Status {
    broker_status(
        Code::Unavailable,
        "sandbox_disconnected",
        "the sandbox's supervisor is not connected",
        &[],
    )
}

/// A forwarded connection refused `sandbox_disconnected`, then `UNAVAILABLE`
/// without an `ErrorInfo`, is retried until admitted, and sends `open` only
/// once admitted.
#[tokio::test(flavor = "multi_thread")]
async fn a_forwarded_connection_refused_before_admission_is_retried() {
    use std::io::Write;
    let broker = FakeBroker::start();
    broker.push_port(PortScript::refuse(disconnected()));
    broker.push_port(PortScript::refuse(Status::unavailable("restarting")));
    broker.push_port(PortScript::admit([
        PortStep::AwaitBytes(2),
        PortStep::Data(b"ok".to_vec()),
        PortStep::Eof,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let mut conn = TcpStream::connect(("127.0.0.1", fwd.port())).unwrap();
    conn.write_all(b"hi").unwrap();
    assert_eq!(read_all(&mut conn).unwrap(), b"ok");
    let calls = broker.calls();
    assert_eq!(calls.len(), 3, "{calls:#?}");
    assert!(
        calls[..2].iter().all(|c| c.frames_before_admission == 0),
        "open went out before admission: {calls:#?}"
    );
    assert_eq!(calls[2].port_open, Some(8888));
    assert_eq!(calls[2].port_data, b"hi");
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(class_line(&err).is_none(), "{err}");
}

/// A forwarded connection still refused `sandbox_disconnected` after the
/// minute of retries is refused on its own; the listener serves the next.
#[tokio::test(flavor = "multi_thread")]
async fn a_forwarded_connection_past_the_admission_retry_is_refused_alone() {
    let broker = FakeBroker::start();
    for _ in 0..40 {
        broker.push_port(PortScript::refuse(disconnected()));
    }
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let started = Instant::now();
    let mut conn = TcpStream::connect(("127.0.0.1", port)).unwrap();
    fwd.wait_for_within(
        "a forwarded connection was refused",
        Duration::from_secs(90),
    );
    // Retried for the minute the design gives a supervisor to connect.
    assert!(
        started.elapsed() >= Duration::from_secs(55),
        "{:?}",
        started.elapsed()
    );
    let _ = read_all(&mut conn);
    let tried = broker.calls().len();
    assert!(tried > 2, "{tried}");
    let _next = TcpStream::connect(("127.0.0.1", port)).expect("the listener closed");
    broker.wait_for(WAIT, |c| c.len() > tried);
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(class_line(&err).is_none(), "{err}");
}

/// A session answer with `life` seconds left by its own `Date`.
fn session_with_life(broker: &FakeBroker, life: i64) -> ResponseTemplate {
    use hedra_cli_sdk::api::prelude::{DateTime, Utc};
    let now = std::time::SystemTime::now();
    let secs = now.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64;
    let expires = DateTime::<Utc>::from_timestamp(secs + life, 0).unwrap();
    ResponseTemplate::new(200)
        .insert_header("date", httpdate::fmt_http_date(now).as_str())
        .set_body_json(serde_json::json!({
            "token": TOKEN, "broker_url": broker.broker_url(),
            "expires_at": expires.to_rfc3339(), "replica": 0,
        }))
}

/// A connection opened in its session's last minute mints a new session
/// first; one opened earlier reuses it.
#[tokio::test(flavor = "multi_thread")]
async fn a_connection_in_the_sessions_last_minute_mints_first() {
    // (seconds the session lives, mints for two connections)
    for (life, want) in [(600, 1), (30, 2)] {
        let broker = FakeBroker::start();
        for _ in 0..2 {
            broker.push_port(PortScript::admit([
                PortStep::AwaitOpen,
                PortStep::Data(b"ok".to_vec()),
                PortStep::Eof,
            ]));
        }
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(format!("/v3/sandboxes/{ID}/session")))
            .respond_with(session_with_life(&broker, life))
            .mount(&server)
            .await;
        mount_sandbox(&server, ID, sandbox(ID, "box", "active", running())).await;
        let run = Run::new(&server);
        let fwd = Forward::start(&run);
        let port = fwd.port();
        for _ in 0..2 {
            let mut conn = TcpStream::connect(("127.0.0.1", port)).unwrap();
            assert_eq!(read_all(&mut conn).unwrap(), b"ok");
        }
        assert_eq!(mints(&server, ID).await, want, "a {life} s session");
        assert_eq!(broker.calls().len(), 2);
        signal(&fwd.child, libc::SIGINT);
        let (code, err) = fwd.finish();
        assert_eq!(code, 0, "{err}");
    }
}

// ------------------------------------------------------------ cp, more

/// A copy in refused `UNAVAILABLE` without an `ErrorInfo`, then
/// `sandbox_disconnected`, is retried, sends no stdin before admission, and
/// delivers the file byte for byte once admitted.
#[tokio::test(flavor = "multi_thread")]
async fn a_copy_in_refused_before_admission_is_retried_and_delivers_every_byte() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(Status::unavailable("restarting")));
    broker.push_exec(ExecScript::refuse(disconnected()));
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"dir\n".to_vec()),
        Step::Count,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let body: Vec<u8> = (0..200_000u32).map(|i| (i * 7 % 251) as u8).collect();
    std::fs::write(run.home.path().join("data.bin"), &body).unwrap();
    let dst = format!("{ID}:/work");
    let out = run.run(&["sandbox", "cp", "data.bin", &dst]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    let calls = broker.calls();
    assert_eq!(calls.len(), 3, "{err}");
    assert!(
        calls[..2].iter().all(|c| c.frames_before_admission == 0),
        "stdin went out before admission: {calls:#?}"
    );
    let stdin = &calls[2].stdin;
    assert!(calls[2].stdin_eof);
    assert_eq!(stdin.len() % 10240, 0, "{}", stdin.len());
    // One header, the body, then padding.
    assert_eq!(&stdin[..8], b"data.bin");
    assert_eq!(&stdin[512..512 + body.len()], &body[..]);
}

/// A local path that is not UTF-8 reaches `cp` as bytes, past clap and the
/// verb's own reading of argv: a missing one is refused with 255 (as a line,
/// or as JSON when asked), and where the filesystem takes such a name, the
/// file is copied in under its own bytes.
#[tokio::test(flavor = "multi_thread")]
async fn a_local_path_that_is_not_utf8_reaches_cp_without_a_panic() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let local = OsStr::from_bytes(b"./caf\xe9");
    let dst = format!("{ID}:/x");
    for extra in [&[][..], &["--dry-run"][..], &["--format", "json"][..]] {
        let out = run
            .command(&["sandbox", "cp"])
            .args(extra)
            .arg(local)
            .arg(&dst)
            .output()
            .unwrap();
        let err = stderr(&out);
        assert!(!err.contains("panicked"), "{extra:?}: {err}");
        assert_eq!(out.status.code(), Some(255), "{extra:?}: {err}");
        if extra.contains(&"json") {
            assert_eq!(json_error(&err)["error"]["exit_code"], 19, "{err}");
        } else {
            assert!(
                class_line(&err)
                    .is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19):")),
                "{extra:?}: {err}"
            );
        }
    }
    assert!(broker.calls().is_empty());

    // macOS refuses such a name; Linux takes any bytes.
    if std::fs::write(run.home.path().join(local), b"body").is_err() {
        return;
    }
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"dir\n".to_vec()),
        Step::Count,
        Step::Exit(0),
    ]));
    let out = run
        .command(&["sandbox", "cp"])
        .arg(local)
        .arg(&dst)
        .output()
        .unwrap();
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(0), "{err}");
    let calls = broker.calls();
    assert_eq!(calls.len(), 1, "{err}");
    assert_eq!(&calls[0].stdin[..5], b"caf\xe9\0");
}

/// A copy still refused `sandbox_disconnected` after the minute of retries
/// exits 255 with the Not ready line.
#[tokio::test(flavor = "multi_thread")]
async fn a_copy_past_the_admission_retry_is_not_ready() {
    let broker = FakeBroker::start();
    for _ in 0..40 {
        broker.push_exec(ExecScript::refuse(disconnected()));
    }
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let src = format!("{ID}:/work/tree");
    let started = Instant::now();
    let out = run.run(&["sandbox", "cp", &src, "out"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    // Retried for the minute the design gives a supervisor to connect.
    assert!(
        started.elapsed() >= Duration::from_secs(55),
        "{:?}",
        started.elapsed()
    );
    assert!(
        class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Not ready or stopped (13): ")),
        "{err}"
    );
    assert!(broker.calls().len() > 2);
    assert!(!run.home.path().join("out").exists());
}

/// An image without `tar` fails a copy either way with exit 255 and a line
/// that says so.
#[tokio::test(flavor = "multi_thread")]
async fn an_image_without_tar_fails_the_copy_both_ways() {
    let no_tar = b"[hedra-cp] no tar\n".to_vec();
    for into in [true, false] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([
            Step::Start,
            Step::Stderr(no_tar.clone()),
            Step::Exit(127),
        ]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        std::fs::write(run.home.path().join("a.txt"), b"a").unwrap();
        let remote = format!("{ID}:/work/a.txt");
        let args = if into {
            ["sandbox", "cp", "a.txt", remote.as_str()]
        } else {
            ["sandbox", "cp", remote.as_str(), "b.txt"]
        };
        let out = run.run(&args);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(255), "{args:?}: {err}");
        assert!(
            class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19):")
                && l.contains("has no `tar`")),
            "{args:?}: {err}"
        );
        assert!(!err.contains("[hedra-cp]"), "{args:?}: {err}");
    }
}

/// An image without `sh`, which the supervisor reports with its line that
/// the command could not start and exit 2, fails the copy naming `sh`, not
/// with 2, which reads as Not signed in.
#[tokio::test(flavor = "multi_thread")]
async fn an_image_without_sh_fails_the_copy_naming_sh() {
    let line = b"[hedra] the command could not start: exec: \"sh\": executable file not found in $PATH\r\n";
    for into in [true, false] {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([
            Step::Start,
            Step::Stderr(line.to_vec()),
            Step::Exit(2),
        ]));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        std::fs::write(run.home.path().join("a.txt"), b"a").unwrap();
        let remote = format!("{ID}:/work/a.txt");
        let args = if into {
            ["sandbox", "cp", "a.txt", remote.as_str()]
        } else {
            ["sandbox", "cp", remote.as_str(), "b.txt"]
        };
        let out = run.run(&args);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(255), "{args:?}: {err}");
        assert!(
            class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19):")
                && l.contains(
                    "could not start `sh` (exec: \"sh\": executable file not found in $PATH)"
                )),
            "{args:?}: {err}"
        );
        assert!(!err.contains("[hedra] the command"), "{args:?}: {err}");
    }
}

/// A copy-out entry name holding a terminal escape is refused, and the
/// name is printed with its control characters escaped.
#[tokio::test(flavor = "multi_thread")]
async fn a_refused_entry_name_is_printed_escaped() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(one_entry("tree\x1b]52;c;evil\x07")),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let src = format!("{ID}:/work/tree");
    let out = run.run(&["sandbox", "cp", &src, "out"]);
    let err = stderr(&out);
    assert_eq!(out.status.code(), Some(255), "{err}");
    assert!(
        class_line(&err).is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19):")),
        "{err}"
    );
    assert!(
        !err.contains('\x1b') && err.contains("]52;c;evil"),
        "{err:?}"
    );
}

/// The stderr line holding a JSON error object, parsed.
fn json_error(err: &str) -> serde_json::Value {
    err.lines()
        .find_map(|l| serde_json::from_str::<serde_json::Value>(l).ok())
        .unwrap_or_else(|| panic!("no JSON error in: {err}"))
}

fn denied_naming_no_scope() -> Status {
    broker_status(
        Code::PermissionDenied,
        "permission_denied",
        "permission denied",
        &[],
    )
}

/// A broker `permission_denied` naming no scope names the one each verb
/// needs: `sandboxes:exec` for `cp`, which runs tar through `Exec`, and
/// `sandboxes:write` for `port-forward`. With a machine format asked for,
/// the error is a JSON object on stderr, and stdout stays empty.
#[tokio::test(flavor = "multi_thread")]
async fn a_missing_scope_names_the_scope_each_verb_needs_as_a_line_and_as_json() {
    let cp_src = format!("{ID}:/work/tree");
    for json in [false, true] {
        let format: &[&str] = if json { &["--format", "json"] } else { &[] };

        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::refuse(denied_naming_no_scope()));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let mut args = vec!["sandbox", "cp", cp_src.as_str(), "out"];
        args.extend_from_slice(format);
        let out = run.run(&args);
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(255), "cp, json {json}: {err}");
        assert!(stdout(&out).is_empty(), "{}", stdout(&out));
        check_missing_scope(&err, json, "sandboxes:exec");

        let broker = FakeBroker::start();
        broker.push_port(PortScript::refuse(denied_naming_no_scope()));
        let server = running_sandbox(&broker).await;
        let run = Run::new(&server);
        let mut args = vec!["sandbox", "port-forward", ID, "0:8888"];
        args.extend_from_slice(format);
        let fwd = Forward::start_with(&run, &args);
        let port = fwd.port();
        let _ = TcpStream::connect(("127.0.0.1", port));
        let (code, err) = fwd.finish();
        assert_eq!(code, 10, "port-forward, json {json}: {err}");
        check_missing_scope(&err, json, "sandboxes:write");
    }
}

fn check_missing_scope(err: &str, json: bool, scope: &str) {
    let lacks = format!("This key lacks `{scope}`.");
    if !json {
        assert!(
            class_line(err).is_some_and(
                |l| l.starts_with("hedra-cli: Missing scope (10): ") && l.contains(&lacks)
            ),
            "{err}"
        );
        return;
    }
    assert!(class_line(err).is_none(), "{err}");
    let e = &json_error(err)["error"];
    assert_eq!(e["class"], "Missing scope", "{err}");
    assert_eq!(e["exit_code"], 10, "{err}");
    assert_eq!(e["reason"], "permission_denied", "{err}");
    assert!(
        e["message"].as_str().is_some_and(|m| m.contains(&lacks)),
        "{err}"
    );
}

/// The sandbox's tar stderr goes through cp's own stderr handle, as a
/// command's stderr goes through exec's: a closed stderr pipe ends cp by
/// SIGPIPE, not with a panic, and the copy itself has landed.
#[tokio::test(flavor = "multi_thread")]
async fn a_closed_stderr_ends_cp_by_sigpipe_when_it_relays_tar_stderr() {
    use std::os::unix::process::ExitStatusExt;
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(one_entry("tree")),
        Step::Stderr(b"tar: a note from the sandbox\n".to_vec()),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cp_src = format!("{ID}:/work/tree");
    let mut child = run.spawn(&["sandbox", "cp", &cp_src, "out"]);
    drop(child.stderr.take());
    let out = wait(child, WAIT);
    assert_eq!(out.status.signal(), Some(libc::SIGPIPE), "{:?}", out.status);
    assert_eq!(std::fs::read(run.home.path().join("out")).unwrap(), b"x");
}

/// The listener survives a dropped broker connection: the connection
/// forwarded over it ends, and the next one reconnects the channel and is
/// served.
#[tokio::test(flavor = "multi_thread")]
async fn the_listener_survives_a_dropped_broker_connection_and_the_next_reconnects() {
    first_call_lost_then_served(
        PortScript::admit([PortStep::AwaitOpen, PortStep::Close]),
        true,
    )
    .await;
}

/// A call that loses the race with the close of a channel left idle is
/// dropped mid-open, before any header, and is retried before admission:
/// the same forwarded connection is served on a new broker connection.
#[tokio::test(flavor = "multi_thread")]
async fn a_forwarded_connection_dropped_mid_open_is_retried() {
    first_call_lost_then_served(PortScript::drop_connection(), false).await;
}

/// A port-forward whose first `PortForward` call follows `first`, losing
/// its broker connection, and whose next is admitted and answers "pong" to
/// "ping". With `after_open`, the first local connection had its call and
/// ends with it, so a second one is made; otherwise the first is retried.
async fn first_call_lost_then_served(first: PortScript, after_open: bool) {
    use std::io::Write;
    let broker = FakeBroker::start();
    broker.push_port(first);
    broker.push_port(PortScript::admit([
        PortStep::AwaitBytes(4),
        PortStep::Data(b"pong".to_vec()),
        PortStep::Eof,
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let mut conn = TcpStream::connect(("127.0.0.1", port)).unwrap();
    conn.set_read_timeout(Some(WAIT)).unwrap();
    if after_open {
        broker.wait_for(WAIT, |c| {
            c.len() == 1 && (c[0].client_gone || c[0].reset.is_some())
        });
        let _ = read_all(&mut conn);
        conn = TcpStream::connect(("127.0.0.1", port)).unwrap();
        conn.set_read_timeout(Some(WAIT)).unwrap();
    }
    conn.write_all(b"ping").unwrap();
    assert_eq!(read_all(&mut conn).unwrap(), b"pong");
    let calls = broker.wait_for(WAIT, |c| c.len() == 2 && c[1].port_data == b"ping");
    assert_eq!(calls[0].admitted, after_open, "{:?}", calls[0]);
    assert_eq!(calls[1].port_open, Some(8888));
    assert_eq!(broker.connections(), 2);
    assert!(class_line(&fwd.stderr()).is_none(), "{}", fwd.stderr());
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
}

/// `sandbox stop` under a running port-forward, as on staging: the
/// connection open across the stop is reset when its call ends
/// `sandbox_disconnected`, and the listener carries on; the next connection
/// is refused `sandbox_disconnected`, says the supervisor is not connected,
/// mints again, and the mint's 400 `FAILED_PRECONDITION` refuses it, closes
/// the listener and ends the port-forward with Not ready or stopped (13),
/// naming the sandbox's state and the next step.
#[tokio::test(flavor = "multi_thread")]
async fn a_stop_under_a_port_forward_resets_its_connection_then_exits_13() {
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Fail(disconnected()),
    ]));
    broker.push_port(PortScript::refuse(disconnected()));
    let server = MockServer::start().await;
    let session = || Mock::given(method("POST")).and(path(format!("/v3/sandboxes/{ID}/session")));
    session()
        .respond_with(session_answer(&broker))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    session()
        .respond_with(envelope(
            400,
            "FAILED_PRECONDITION",
            "The sandbox is not active.",
            None,
        ))
        .mount(&server)
        .await;
    // The first read, before the start line, finds it running; the next,
    // the refusal's, finds it ended.
    let mut ended = sandbox(ID, "box", "ended", serde_json::json!([]));
    ended["end_reason"] = "caller_stopped".into();
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{ID}")))
        .respond_with(Sequence::new(vec![
            ResponseTemplate::new(200).set_body_json(sandbox(ID, "box", "active", running())),
            ResponseTemplate::new(200).set_body_json(ended),
        ]))
        .mount(&server)
        .await;

    let run = Run::new(&server);
    let fwd = Forward::start(&run);
    let port = fwd.port();
    let mut open = TcpStream::connect(("127.0.0.1", port)).unwrap();
    open.set_read_timeout(Some(WAIT)).unwrap();
    fwd.wait_for("a forwarded connection was reset: ");
    let _ = read_all(&mut open);
    assert!(class_line(&fwd.stderr()).is_none(), "{}", fwd.stderr());

    let _next = TcpStream::connect(("127.0.0.1", port)).unwrap();
    let (code, err) = fwd.finish();
    assert_eq!(code, 13, "{err}");
    assert!(
        err.contains(&format!(
            "`{ID}`'s supervisor is not connected yet; retrying."
        )),
        "{err}"
    );
    assert_eq!(
        class_line(&err),
        Some(
            format!(
                "hedra-cli: Not ready or stopped (13): `{ID}` is ended (caller_stopped). \
                 Create another, or run `hedra-cli sandbox ls` for the running ones."
            )
            .as_str()
        ),
        "{err}"
    );
    assert_eq!(err.matches("hedra-cli: ").count(), 1, "{err}");
    assert_eq!(mints(&server, ID).await, 2);
    assert!(
        TcpStream::connect(("127.0.0.1", port)).is_err(),
        "still listening"
    );
}

/// `--debug` on `cp` and `port-forward` prints the broker lines `exec`
/// prints: the broker it dials, a refusal and the admission, never the
/// token.
#[tokio::test(flavor = "multi_thread")]
async fn debug_on_cp_and_port_forward_shows_the_broker_and_never_the_token() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::refuse(Status::unavailable("restarting")));
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"dir\n".to_vec()),
        Step::Count,
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    std::fs::write(run.home.path().join("a.txt"), b"a").unwrap();
    let dst = format!("{ID}:/work");
    let out = run.run(&["sandbox", "cp", "--debug", "a.txt", &dst]);
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

    let broker = FakeBroker::start();
    broker.push_port(PortScript::refuse(Status::unavailable("restarting")));
    broker.push_port(PortScript::hold());
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let fwd = Forward::start_with(&run, &["sandbox", "port-forward", "--debug", ID, "0:8888"]);
    let _conn = TcpStream::connect(("127.0.0.1", fwd.port())).unwrap();
    fwd.wait_for("< port_forward admitted");
    signal(&fwd.child, libc::SIGINT);
    let (code, err) = fwd.finish();
    assert_eq!(code, 0, "{err}");
    assert!(
        err.contains(&format!("> port_forward https://{}", broker.broker_url())),
        "{err}"
    );
    assert!(
        err.contains("< port_forward refused: code Unavailable"),
        "{err}"
    );
    assert!(!err.contains(TOKEN), "{err}");
}
