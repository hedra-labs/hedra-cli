//! Shared harness for the stream verbs' tests: the built binary, a mock v3
//! API whose session route points at a fake broker, a pseudo-terminal, and a
//! local `CONNECT` proxy.

#![allow(dead_code)]

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Output, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[allow(unused_imports)]
pub use hedra_cli_sandbox::fake_broker::{
    broker_status, ca_bundle, ExecScript, FakeBroker, LogsEnd, LogsScript, Step,
};
#[allow(unused_imports)]
pub use hedra_cli_sandbox::tonic::{Code, Status};
use serde_json::{json, Value};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Request, Respond, ResponseTemplate};

pub const ID: &str = "sandbox_3f2504e0-4f89-41d3-9a0c-0305e82c3301";
pub const TOKEN: &str = "jwt.session.one";

pub struct Run {
    pub home: tempfile::TempDir,
    pub base: String,
}

impl Run {
    pub fn new(server: &MockServer) -> Self {
        Self {
            home: tempfile::tempdir().unwrap(),
            base: format!("{}/v3", server.uri()),
        }
    }

    /// The binary with an isolated home, the API key, the preview gate and
    /// the fake broker's CA.
    pub fn command(&self, args: &[&str]) -> Command {
        // Before any `--`, after which everything is the remote command.
        let split = args.iter().position(|a| *a == "--").unwrap_or(args.len());
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_hedra-cli"));
        cmd.args(&args[..split])
            .arg("--base-url")
            .arg(&self.base)
            .args(&args[split..])
            .current_dir(self.home.path())
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap_or_default())
            .env("HOME", self.home.path())
            .env("FERN_CLI_CREDENTIAL_STORE", "file")
            .env("NO_COLOR", "1")
            .env("HEDRA_API_KEY", "key_test:secret")
            .env("HEDRA_CLI_SANDBOX_PREVIEW", "1")
            .env("HEDRA_CLI_CA_BUNDLE", ca_bundle())
            .stdin(Stdio::null());
        cmd
    }

    pub fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    pub fn spawn(&self, args: &[&str]) -> Child {
        self.command(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap()
    }
}

pub fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

pub fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

pub fn sandbox(id: &str, name: &str, status: &str, instances: Value) -> Value {
    json!({
        "id": id, "name": name, "gpu": "h100:1", "gpu_model": "h100", "replicas": 1,
        "status": status, "created_at": "2026-11-03T12:00:00Z", "env_keys": [],
        "instances": instances, "url": null,
        "billing": {"usd_per_hour": 2.5, "paid_usd": 0.0, "settled": false},
    })
}

pub fn running() -> Value {
    json!([{"replica": 0, "status": "running"}])
}

pub fn envelope(
    status: u16,
    code: &str,
    message: &str,
    retry_after: Option<u64>,
) -> ResponseTemplate {
    ResponseTemplate::new(status).set_body_json(json!({
        "error": {"code": code, "message": message, "retryable": retry_after.is_some(),
                  "retry_after": retry_after},
        "trace_id": "trace_test",
    }))
}

pub fn session_answer(broker: &FakeBroker) -> ResponseTemplate {
    ResponseTemplate::new(200).set_body_json(json!({
        "token": TOKEN, "broker_url": broker.broker_url(),
        "expires_at": "2099-01-01T00:00:00Z", "replica": 0,
    }))
}

/// The session route for `id`, answering `broker`.
pub async fn mount_session(server: &MockServer, id: &str, broker: &FakeBroker) {
    Mock::given(method("POST"))
        .and(path(format!("/v3/sandboxes/{id}/session")))
        .respond_with(session_answer(broker))
        .mount(server)
        .await;
}

/// `GET /sandboxes/{id}` answering `body`.
pub async fn mount_sandbox(server: &MockServer, id: &str, body: Value) {
    Mock::given(method("GET"))
        .and(path(format!("/v3/sandboxes/{id}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .mount(server)
        .await;
}

/// A running sandbox `ID` whose session route answers `broker`.
pub async fn running_sandbox(broker: &FakeBroker) -> MockServer {
    let server = MockServer::start().await;
    mount_session(&server, ID, broker).await;
    mount_sandbox(&server, ID, sandbox(ID, "box", "active", running())).await;
    server
}

pub async fn requests_to(server: &MockServer, m: &str, p: &str) -> Vec<Request> {
    server
        .received_requests()
        .await
        .unwrap_or_default()
        .into_iter()
        .filter(|r| r.method.as_str() == m && r.url.path() == p)
        .collect()
}

pub async fn mints(server: &MockServer, id: &str) -> usize {
    requests_to(server, "POST", &format!("/v3/sandboxes/{id}/session"))
        .await
        .len()
}

/// Answers in turn from `answers`, repeating the last.
pub struct Sequence {
    answers: Vec<ResponseTemplate>,
    next: AtomicUsize,
}

impl Sequence {
    pub fn new(answers: Vec<ResponseTemplate>) -> Self {
        Self {
            answers,
            next: AtomicUsize::new(0),
        }
    }
}

impl Respond for Sequence {
    fn respond(&self, _: &Request) -> ResponseTemplate {
        let i = self.next.fetch_add(1, Ordering::SeqCst);
        self.answers[i.min(self.answers.len() - 1)].clone()
    }
}

pub fn signal(child: &Child, sig: i32) {
    // SAFETY: sends a signal to a child this test spawned.
    assert_eq!(unsafe { libc::kill(child.id() as i32, sig) }, 0);
}

/// Waits for `child` with a timeout, killing it on expiry.
pub fn wait(mut child: Child, timeout: Duration) -> Output {
    let deadline = Instant::now() + timeout;
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let out = child.wait_with_output().unwrap();
            panic!(
                "hedra-cli did not exit within {timeout:?}; stderr: {}",
                stderr(&out)
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// The `hedra-cli: <class> (<code>):` line, if stderr has one.
pub fn class_line(err: &str) -> Option<&str> {
    err.lines().find(|l| l.starts_with("hedra-cli: "))
}

/// A local HTTP `CONNECT` proxy that records each request line and relays.
pub struct Proxy {
    pub port: u16,
    pub requests: Arc<Mutex<Vec<String>>>,
    tunnels: Arc<Mutex<Vec<(TcpStream, TcpStream)>>>,
    control: Arc<Control>,
}

#[derive(Default)]
struct Control {
    frozen: std::sync::atomic::AtomicBool,
    resetting: std::sync::atomic::AtomicBool,
}

impl Proxy {
    pub fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let tunnels = Arc::new(Mutex::new(Vec::new()));
        let control = Arc::new(Control::default());
        let seen = requests.clone();
        let open = tunnels.clone();
        let ctl = control.clone();
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                let Ok(mut client) = conn else { continue };
                let seen = seen.clone();
                let open = open.clone();
                let ctl = ctl.clone();
                std::thread::spawn(move || {
                    let mut head = Vec::new();
                    let mut b = [0u8; 1];
                    while !head.ends_with(b"\r\n\r\n") {
                        if client.read(&mut b).unwrap_or(0) == 0 {
                            return;
                        }
                        head.push(b[0]);
                    }
                    let line = String::from_utf8_lossy(&head)
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .to_string();
                    seen.lock().unwrap().push(line.clone());
                    let target = line.split_whitespace().nth(1).unwrap_or_default();
                    let Ok(upstream) = TcpStream::connect(target) else {
                        let _ = client.write_all(b"HTTP/1.1 502 Bad Gateway\r\n\r\n");
                        return;
                    };
                    let _ = client.write_all(b"HTTP/1.1 200 Connection established\r\n\r\n");
                    open.lock()
                        .unwrap()
                        .push((client.try_clone().unwrap(), upstream.try_clone().unwrap()));
                    relay(client, upstream, ctl);
                });
            }
        });
        Self {
            port,
            requests,
            tunnels,
            control,
        }
    }

    /// Stops relaying either way and stops reading, keeping every
    /// connection open: a network that has gone dead.
    pub fn freeze(&self) {
        self.control.frozen.store(true, Ordering::SeqCst);
    }

    /// Resets each tunnel's client connection (`RST`, not `FIN`).
    pub fn reset(&self) {
        use std::os::fd::AsRawFd;
        self.control.resetting.store(true, Ordering::SeqCst);
        for (c, upstream) in self.tunnels.lock().unwrap().drain(..) {
            let linger = libc::linger {
                l_onoff: 1,
                l_linger: 0,
            };
            // SAFETY: sets an option on a socket this proxy owns.
            unsafe {
                libc::setsockopt(
                    c.as_raw_fd(),
                    libc::SOL_SOCKET,
                    libc::SO_LINGER,
                    (&linger as *const libc::linger).cast(),
                    std::mem::size_of::<libc::linger>() as libc::socklen_t,
                );
            }
            // Ends both relay threads' reads, so every handle closes and the
            // last close sends the reset.
            let _ = c.shutdown(std::net::Shutdown::Read);
            let _ = upstream.shutdown(std::net::Shutdown::Read);
        }
    }

    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}", self.port)
    }

    pub fn requests(&self) -> Vec<String> {
        self.requests.lock().unwrap().clone()
    }
}

fn relay(a: TcpStream, b: TcpStream, control: Arc<Control>) {
    let (a2, b2) = (a.try_clone().unwrap(), b.try_clone().unwrap());
    let both = control.clone();
    std::thread::spawn(move || copy(a2, b2, both));
    copy(b, a, control);
}

/// Copies `from` to `to` until either ends; once frozen, drops what it read
/// and reads no more.
fn copy(mut from: TcpStream, mut to: TcpStream, control: Arc<Control>) {
    let mut buf = [0u8; 16 * 1024];
    loop {
        let n = match from.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => n,
        };
        if control.frozen.load(Ordering::SeqCst) {
            loop {
                std::thread::sleep(Duration::from_secs(3600));
            }
        }
        if to.write_all(&buf[..n]).is_err() {
            break;
        }
    }
    // A reset closes without a FIN.
    if !control.resetting.load(Ordering::SeqCst) {
        let _ = to.shutdown(std::net::Shutdown::Write);
    }
}

/// A pseudo-terminal whose slave becomes a child's controlling terminal,
/// with the child in its foreground process group. What the child writes is
/// read and discarded, so its writes never block.
#[cfg(unix)]
pub struct Pty {
    master: Mutex<Option<std::fs::File>>,
    pub slave: std::fs::File,
    draining: Arc<std::sync::atomic::AtomicBool>,
    drain: Mutex<Option<std::thread::JoinHandle<()>>>,
}

#[cfg(unix)]
impl Pty {
    pub fn open(rows: u16, cols: u16) -> Self {
        use std::os::fd::FromRawFd;
        let mut master = -1;
        let mut slave = -1;
        let mut size = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: openpty fills two descriptors this test then owns; neither
        // is inherited, so closing the master here hangs the terminal up.
        let rc = unsafe {
            let rc = libc::openpty(
                &mut master,
                &mut slave,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                &mut size,
            );
            if rc == 0 {
                libc::fcntl(master, libc::F_SETFD, libc::FD_CLOEXEC);
                libc::fcntl(slave, libc::F_SETFD, libc::FD_CLOEXEC);
                // Typed keys never block the test on a full input queue.
                let flags = libc::fcntl(master, libc::F_GETFL);
                libc::fcntl(master, libc::F_SETFL, flags | libc::O_NONBLOCK);
            }
            rc
        };
        assert_eq!(rc, 0, "openpty: {}", std::io::Error::last_os_error());
        // SAFETY: both descriptors are valid and owned from here on.
        let (master, slave) = unsafe {
            (
                std::fs::File::from_raw_fd(master),
                std::fs::File::from_raw_fd(slave),
            )
        };
        let draining = Arc::new(std::sync::atomic::AtomicBool::new(true));
        let drain = Self::drain(master.try_clone().unwrap(), draining.clone());
        Self {
            master: Mutex::new(Some(master)),
            slave,
            draining,
            drain: Mutex::new(Some(drain)),
        }
    }

    /// Reads the master until told to stop, polling so it can.
    fn drain(
        mut master: std::fs::File,
        draining: Arc<std::sync::atomic::AtomicBool>,
    ) -> std::thread::JoinHandle<()> {
        use std::os::fd::AsRawFd;
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while draining.load(Ordering::SeqCst) {
                let mut fd = libc::pollfd {
                    fd: master.as_raw_fd(),
                    events: libc::POLLIN,
                    revents: 0,
                };
                // SAFETY: polls one descriptor this thread owns.
                if unsafe { libc::poll(&mut fd, 1, 50) } <= 0 {
                    continue;
                }
                if fd.revents & libc::POLLIN == 0
                    || !matches!(master.read(&mut buf), Ok(n) if n > 0)
                {
                    std::thread::sleep(Duration::from_millis(20));
                }
            }
        })
    }

    /// Writes `bytes` as typed keys, dropping what the input queue cannot take.
    pub fn type_keys(&self, bytes: &[u8]) {
        if let Some(m) = self.master.lock().unwrap().as_mut() {
            let _ = m.write_all(bytes);
        }
    }

    /// Changes the terminal's size, which signals its foreground group.
    pub fn resize(&self, rows: u16, cols: u16) {
        use std::os::fd::AsRawFd;
        let size = libc::winsize {
            ws_row: rows,
            ws_col: cols,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        let master = self.master.lock().unwrap();
        let fd = master.as_ref().expect("the terminal is open").as_raw_fd();
        // SAFETY: sets the size of a terminal this test owns.
        assert_eq!(unsafe { libc::ioctl(fd, libc::TIOCSWINSZ as _, &size) }, 0);
    }

    /// Closes the master, as closing a terminal window does.
    pub fn close(&self) {
        self.draining.store(false, Ordering::SeqCst);
        if let Some(t) = self.drain.lock().unwrap().take() {
            let _ = t.join();
        }
        self.master.lock().unwrap().take();
    }

    /// Runs `cmd` with the slave as stdin and stdout and as its controlling
    /// terminal; stderr stays a pipe. A `sh` session leader runs it and
    /// outlives it, since on macOS the leader's exit revokes the terminal
    /// before the test can read its mode.
    pub fn spawn(&self, cmd: &Command) -> OnPty {
        self.spawn_with(cmd, false)
    }

    /// `spawn`, with SIGHUP ignored from the start, as `nohup` leaves it.
    pub fn spawn_nohup(&self, cmd: &Command) -> OnPty {
        self.spawn_with(cmd, true)
    }

    fn spawn_with(&self, cmd: &Command, ignore_hup: bool) -> OnPty {
        use std::os::unix::process::CommandExt;
        let mut sh = Command::new("/bin/sh");
        sh.arg("-c")
            .arg(r#""$@"; printf '\n@@exit:%s@@\n' "$?" >&2; read _"#)
            .arg("sh")
            .arg(cmd.get_program())
            .args(cmd.get_args())
            .env_clear();
        for (k, v) in cmd.get_envs() {
            if let Some(v) = v {
                sh.env(k, v);
            }
        }
        if let Some(dir) = cmd.get_current_dir() {
            sh.current_dir(dir);
        }
        sh.stdin(Stdio::from(self.slave.try_clone().unwrap()))
            .stdout(Stdio::from(self.slave.try_clone().unwrap()))
            .stderr(Stdio::piped());
        // SAFETY: only async-signal-safe calls between fork and exec.
        unsafe {
            sh.pre_exec(move || {
                if ignore_hup {
                    libc::signal(libc::SIGHUP, libc::SIG_IGN);
                }
                if libc::setsid() < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                if libc::ioctl(0, libc::TIOCSCTTY as _, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                libc::tcsetpgrp(0, libc::getpid());
                Ok(())
            });
        }
        let mut sh = sh.spawn().unwrap();
        let err = Arc::new(Mutex::new(Vec::new()));
        let into = err.clone();
        let mut pipe = sh.stderr.take().unwrap();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            while let Ok(n) = pipe.read(&mut buf) {
                if n == 0 {
                    break;
                }
                into.lock().unwrap().extend_from_slice(&buf[..n]);
            }
        });
        OnPty { sh, err }
    }

    /// Sets or clears `O_NONBLOCK` on the terminal's open file description,
    /// which hedra-cli's stdin and stdout share.
    pub fn set_nonblocking(&self, on: bool) {
        use std::os::fd::AsRawFd;
        let fd = self.slave.as_raw_fd();
        // SAFETY: a flag change on a descriptor this test owns.
        unsafe {
            let flags = libc::fcntl(fd, libc::F_GETFL);
            let flags = if on {
                flags | libc::O_NONBLOCK
            } else {
                flags & !libc::O_NONBLOCK
            };
            assert_eq!(libc::fcntl(fd, libc::F_SETFL, flags), 0);
        }
    }

    /// Whether the terminal is in canonical mode with echo, as a shell
    /// leaves it.
    pub fn cooked(&self) -> bool {
        use std::os::fd::AsRawFd;
        // SAFETY: reads the attributes of a descriptor this test owns.
        unsafe {
            let mut t: libc::termios = std::mem::zeroed();
            assert_eq!(
                libc::tcgetattr(self.slave.as_raw_fd(), &mut t),
                0,
                "tcgetattr: {}",
                std::io::Error::last_os_error()
            );
            t.c_lflag & libc::ICANON != 0 && t.c_lflag & libc::ECHO != 0
        }
    }
}

#[cfg(unix)]
impl Drop for Pty {
    fn drop(&mut self) {
        self.draining.store(false, Ordering::SeqCst);
    }
}

/// hedra-cli running on a pseudo-terminal under its `sh` session leader.
#[cfg(unix)]
pub struct OnPty {
    sh: Child,
    err: Arc<Mutex<Vec<u8>>>,
}

/// How hedra-cli on a pseudo-terminal ended.
#[cfg(unix)]
pub struct Ended {
    pub code: i32,
    pub stderr: String,
}

#[cfg(unix)]
impl OnPty {
    /// hedra-cli's process id.
    pub fn pid(&self) -> i32 {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let out = Command::new("pgrep")
                .args(["-P", &self.sh.id().to_string()])
                .output()
                .unwrap();
            if let Some(pid) = String::from_utf8_lossy(&out.stdout)
                .split_whitespace()
                .next()
                .and_then(|p| p.parse().ok())
            {
                return pid;
            }
            assert!(Instant::now() < deadline, "hedra-cli did not start");
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    /// Whether the `sh` session leader exits within `timeout`: its `read`
    /// after hedra-cli returns once a line is waiting for it.
    pub fn leader_exits_within(&mut self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if self.sh.try_wait().unwrap().is_some() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    pub fn signal(&self, sig: i32) {
        // SAFETY: sends a signal to a process this test started.
        assert_eq!(unsafe { libc::kill(self.pid(), sig) }, 0);
    }

    /// Waits for hedra-cli to exit; the terminal stays open until `self`
    /// is dropped.
    pub fn finish(&self, timeout: Duration) -> Ended {
        let deadline = Instant::now() + timeout;
        loop {
            let err = String::from_utf8_lossy(&self.err.lock().unwrap()).into_owned();
            const MARK: &str = "\n@@exit:";
            if let Some(at) = err.rfind(MARK) {
                let code = err[at + MARK.len()..]
                    .split("@@")
                    .next()
                    .and_then(|c| c.parse().ok())
                    .unwrap();
                return Ended {
                    code,
                    stderr: err[..at].to_string(),
                };
            }
            assert!(
                Instant::now() < deadline,
                "hedra-cli did not exit within {timeout:?}; stderr: {err}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

#[cfg(unix)]
impl Drop for OnPty {
    fn drop(&mut self) {
        let _ = self.sh.kill();
        let _ = self.sh.wait();
    }
}
