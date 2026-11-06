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

/// A ustar stream of one one-byte file, its name written as given.
fn one_entry(name: &str) -> Vec<u8> {
    let mut h = [0u8; 512];
    h[..name.len()].copy_from_slice(name.as_bytes());
    for (at, field) in [
        (100, &b"0000644\0"[..]),
        (108, b"0000000\0"),
        (116, b"0000000\0"),
        (124, b"00000000001\0"),
        (136, b"00000000000\0"),
    ] {
        h[at..at + field.len()].copy_from_slice(field);
    }
    h[156] = b'0';
    h[257..265].copy_from_slice(b"ustar\x0000");
    h[148..156].fill(b' ');
    let sum: u32 = h.iter().map(|b| u32::from(*b)).sum();
    h[148..155].copy_from_slice(format!("{sum:06o}\0").as_bytes());
    let mut out = h.to_vec();
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
    reader: std::thread::JoinHandle<()>,
}

impl Forward {
    fn start(run: &Run) -> Self {
        Self::start_with(run, &["sandbox", "port-forward", ID, "0:8888"])
    }

    fn start_with(run: &Run, args: &[&str]) -> Self {
        let mut child = run.spawn(args);
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
        Self { child, err, reader }
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
        let rest = err.split("127.0.0.1:").nth(1).expect("an IPv4 address");
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
        let _ = self.reader.join();
        let err = self.err.lock().unwrap().clone();
        (code, err)
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
        let (code, err) = fwd.finish();
        assert_eq!(code, want, "{case}: {err}");
        assert!(
            class_line(&err).is_some_and(|l| l.contains(class)),
            "{case}: {err}"
        );
        assert!(
            TcpStream::connect(("127.0.0.1", port)).is_err(),
            "{case}: still listening"
        );
    }
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
/// server's `eof` half-closes the local socket.
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
/// connection on the same broker connection keeps receiving at full speed.
#[tokio::test(flavor = "multi_thread")]
async fn a_stalled_connection_is_reset_while_another_keeps_its_throughput() {
    const LIVE: usize = 48 << 20;
    let broker = FakeBroker::start();
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Flood(512 << 20),
    ]));
    broker.push_port(PortScript::admit([
        PortStep::AwaitOpen,
        PortStep::Flood(LIVE),
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
    let calls = broker.wait_for(WAIT, |c| c[0].reset.is_some() || c[0].client_gone);
    assert!(calls[1].reset.is_none(), "{:?}", calls[1]);
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
