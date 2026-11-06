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
}

impl Forward {
    fn start(run: &Run) -> Self {
        let mut child = run.spawn(&["sandbox", "port-forward", ID, "0:8888"]);
        let mut pipe = child.stderr.take().unwrap();
        let err = Arc::new(Mutex::new(String::new()));
        let into = err.clone();
        std::thread::spawn(move || {
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
        Self { child, err }
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
        std::thread::sleep(Duration::from_millis(100));
        (code, self.stderr())
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
async fn a_socks_proxy_is_refused_before_any_request() {
    let broker = FakeBroker::start();
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cp_src = format!("{ID}:/work/tree");
    for (args, code) in [
        (vec!["sandbox", "cp", cp_src.as_str(), "out"], 255),
        (vec!["sandbox", "port-forward", ID, "0:80"], 19),
    ] {
        let out = run
            .command(&args)
            .env("HTTPS_PROXY", "socks5://127.0.0.1:1080")
            .output()
            .unwrap();
        let err = stderr(&out);
        assert_eq!(out.status.code(), Some(code), "{args:?}: {err}");
        assert!(
            class_line(&err)
                .is_some_and(|l| l.starts_with("hedra-cli: Rejected request (19): a SOCKS proxy")),
            "{args:?}: {err}"
        );
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
