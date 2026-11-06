//! A `SessionService` server over TLS for the stream verbs' tests.
//!
//! Each call takes the next script queued for its method: refuse before
//! admission with a status, or admit (the `sandbox-admitted` header) and run
//! steps. An `Exec` reads its client's frames only as fast as its steps take
//! them, with a few frames of room, as the broker's input window does. Its
//! output queue toward the client follows the broker's rules: an exec
//! without `-t` waits while 192 frames sit unsent (the output window), and
//! one with `-t` ends `rate_limited` once 256 do. `Logs` replays a ring
//! buffer of chunks with strictly increasing stamps, applying `since`
//! inclusively. Every call is recorded: its
//! `authorization`, the frames it received, and whether the client went
//! away before the script finished. The server runs on a thread of its own,
//! so a test may block on a child process while it serves.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use prost_types::Timestamp;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, oneshot};
use tokio_rustls::TlsAcceptor;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::server::Connected;
use tonic::{Code, Request, Response, Status, Streaming};
use tonic_types::{ErrorDetails, StatusExt};

use crate::pb::hedra::sandbox::v1::session_service_server::{SessionService, SessionServiceServer};
use crate::pb::hedra::sandbox::v1::{
    exec_client_frame, exec_server_frame, ExecClientFrame, ExecData, ExecServerFrame, ExecStart,
    InvokeClientFrame, InvokeServerFrame, LogChunk, LogsRequest, PortClientFrame, PortServerFrame,
    Stream,
};
use crate::stream::calls::ADMITTED_HEADER;

/// How long the fake broker's certificates are valid either side of now.
const CERT_VALIDITY: time::Duration = time::Duration::days(7);

/// The fake broker's TLS material, made once per process: a CA, and a
/// certificate for `localhost` and `127.0.0.1` it signed. macOS refuses a
/// server certificate valid for more than 825 days even under a CA the
/// caller added, so a checked-in certificate would expire under the tests.
struct TestPki {
    ca_pem: String,
    cert: rustls::pki_types::CertificateDer<'static>,
    key: Vec<u8>,
}

fn test_pki() -> &'static TestPki {
    static PKI: OnceLock<TestPki> = OnceLock::new();
    PKI.get_or_init(make_test_pki)
}

fn make_test_pki() -> TestPki {
    use rcgen::{
        BasicConstraints, CertificateParams, DnType, ExtendedKeyUsagePurpose, IsCa, KeyPair,
        KeyUsagePurpose, SanType,
    };
    let now = time::OffsetDateTime::now_utc();
    let ca_key = KeyPair::generate().expect("a CA key");
    let mut ca = CertificateParams::new(Vec::<String>::new()).expect("CA parameters");
    ca.distinguished_name
        .push(DnType::CommonName, "hedra-cli fake broker test CA");
    ca.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    ca.not_before = now - CERT_VALIDITY;
    ca.not_after = now + CERT_VALIDITY;
    let ca_cert = ca.self_signed(&ca_key).expect("the CA certificate");

    let key = KeyPair::generate().expect("a server key");
    let mut server = CertificateParams::new(vec!["localhost".to_string()]).expect("parameters");
    server
        .distinguished_name
        .push(DnType::CommonName, "localhost");
    server
        .subject_alt_names
        .push(SanType::IpAddress(std::net::Ipv4Addr::LOCALHOST.into()));
    server.key_usages = vec![KeyUsagePurpose::DigitalSignature];
    server.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];
    server.not_before = now - CERT_VALIDITY;
    server.not_after = now + CERT_VALIDITY;
    let cert = server
        .signed_by(&key, &ca_cert, &ca_key)
        .expect("the server certificate");
    TestPki {
        ca_pem: ca_cert.pem(),
        cert: cert.der().clone(),
        key: key.serialize_der(),
    }
}

/// The PEM of the CA that signed the fake broker's certificate, for a file
/// `HEDRA_CLI_CA_BUNDLE` names. Every fake broker in a process shares it.
pub fn ca_pem() -> &'static str {
    &test_pki().ca_pem
}

/// A status carrying an `ErrorInfo` in the broker's domain.
pub fn broker_status(code: Code, reason: &str, message: &str, metadata: &[(&str, &str)]) -> Status {
    let metadata: HashMap<String, String> = metadata
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    Status::with_error_details(
        code,
        message,
        ErrorDetails::with_error_info(reason, crate::stream::status::DOMAIN, metadata),
    )
}

/// One step of an admitted `Exec` call.
#[derive(Clone, Debug)]
pub enum Step {
    /// Waits for the `start` frame.
    Start,
    Stdout(Vec<u8>),
    Stderr(Vec<u8>),
    /// Sends this chunk on stdout at this interval until the client goes away.
    StdoutForever(Vec<u8>, Duration),
    /// Sends this many bytes on stdout in frames of this size, as fast as
    /// the output queue takes them.
    StdoutBulk(usize, usize),
    /// Echoes stdin to stdout until `stdin_eof`.
    Echo,
    /// Reads stdin until `stdin_eof`, counting its bytes.
    Count,
    /// Writes the count and a newline on stdout.
    PrintCount,
    /// Waits for `stdin_eof`.
    StdinEof,
    Sleep(Duration),
    Exit(i32),
    /// Ends the call with this status.
    Fail(Status),
    /// Closes the TCP connection the call is on.
    Close,
    /// Keeps the call open until the client goes away.
    Hold,
}

/// How an `Exec` call answers before its steps run.
#[derive(Clone, Debug)]
enum Answer {
    Admit,
    Refuse(Status),
    /// Response headers without `sandbox-admitted`.
    Unadmitted,
    /// No response headers at all.
    Stall,
    /// Closes the TCP connection after reading the request, before any
    /// response headers.
    Drop,
}

/// What one `Exec` call does.
#[derive(Clone, Debug)]
pub struct ExecScript {
    answer: Answer,
    steps: Vec<Step>,
}

impl ExecScript {
    pub fn admit(steps: impl IntoIterator<Item = Step>) -> Self {
        Self {
            answer: Answer::Admit,
            steps: steps.into_iter().collect(),
        }
    }

    /// Refuses the call before admission.
    pub fn refuse(status: Status) -> Self {
        Self {
            answer: Answer::Refuse(status),
            steps: Vec::new(),
        }
    }

    /// Answers headers that lack `sandbox-admitted`, then runs `steps`.
    pub fn unadmitted(steps: impl IntoIterator<Item = Step>) -> Self {
        Self {
            answer: Answer::Unadmitted,
            steps: steps.into_iter().collect(),
        }
    }

    /// Never answers: no headers, no status.
    pub fn stall() -> Self {
        Self {
            answer: Answer::Stall,
            steps: Vec::new(),
        }
    }

    /// Reads the request, then closes the connection without answering: a
    /// broker restarting mid-open.
    pub fn drop_connection() -> Self {
        Self {
            answer: Answer::Drop,
            steps: Vec::new(),
        }
    }
}

/// Client frames an `Exec` holds unread before it stops reading.
const INPUT_WINDOW: usize = 4;
/// Frames an exec without `-t` may have queued toward the client.
const OUTPUT_WINDOW: usize = 192;
/// Frames queued toward a client with no output window that end its session.
const QUEUE_DEPTH: usize = 256;

/// How an admitted `Logs` call ends once it has replayed the ring buffer.
#[derive(Clone, Debug)]
pub enum LogsEnd {
    /// Without `follow`, OK trailers once the replay is sent, as the real
    /// broker ends it (it consumes the buffer's eof marker and sends no
    /// chunk for it); with it, held open.
    Default,
    Hold,
    Fail(Status),
    Close,
}

/// What one `PortForward` call does.
#[derive(Clone, Debug)]
pub struct PortScript {
    refuse: Option<Status>,
}

impl PortScript {
    /// Admits the call and holds it open until the client goes away.
    pub fn hold() -> Self {
        Self { refuse: None }
    }

    /// Refuses the call before admission.
    pub fn refuse(status: Status) -> Self {
        Self {
            refuse: Some(status),
        }
    }
}

/// What one `Logs` call does.
#[derive(Clone, Debug)]
pub struct LogsScript {
    refuse: Option<Status>,
    append: Vec<Vec<u8>>,
    end: LogsEnd,
}

impl LogsScript {
    pub fn admit(end: LogsEnd) -> Self {
        Self {
            refuse: None,
            append: Vec::new(),
            end,
        }
    }

    pub fn refuse(status: Status) -> Self {
        Self {
            refuse: Some(status),
            append: Vec::new(),
            end: LogsEnd::Default,
        }
    }

    /// Adds chunks to the ring buffer after the replay, without sending them.
    pub fn then_append(mut self, data: &[u8]) -> Self {
        self.append.push(data.to_vec());
        self
    }
}

/// What the fake broker saw of one call.
#[derive(Clone, Debug, Default)]
pub struct Call {
    pub method: &'static str,
    pub authorization: Option<String>,
    pub admitted: bool,
    /// Frames received while the call was being refused.
    pub frames_before_admission: usize,
    pub start: Option<ExecStart>,
    pub stdin: Vec<u8>,
    pub stdin_eof: bool,
    pub frames: usize,
    /// Each `resize`, as rows and columns.
    pub resizes: Vec<(u32, u32)>,
    /// Output bytes queued toward the client.
    pub sent: usize,
    pub logs: Option<LogsRequest>,
    /// The script ran to its end.
    pub finished: bool,
    /// The client reset the call or closed its connection before the
    /// script finished.
    pub client_gone: bool,
    /// The code the request stream ended with, when it ended in an error.
    pub reset: Option<Code>,
    /// The request stream ended cleanly, which the broker reads as the end
    /// of input.
    pub half_closed: bool,
}

#[derive(Default)]
struct Inner {
    exec: VecDeque<ExecScript>,
    logs: VecDeque<LogsScript>,
    port: VecDeque<PortScript>,
    ring: Vec<LogChunk>,
    calls: Vec<Call>,
    conns: HashMap<u64, std::net::TcpStream>,
    connections: usize,
    failed_handshakes: usize,
}

type State = Arc<Mutex<Inner>>;

fn update(state: &State, idx: usize, f: impl FnOnce(&mut Call)) {
    if let Ok(mut s) = state.lock() {
        f(&mut s.calls[idx]);
    }
}

/// A running fake broker; it stops when dropped.
pub struct FakeBroker {
    addr: SocketAddr,
    state: State,
    shutdown: Option<oneshot::Sender<()>>,
}

impl FakeBroker {
    pub fn start() -> Self {
        let state = State::default();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel();
        let (shutdown, stop) = oneshot::channel::<()>();
        let served = state.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .worker_threads(2)
                .enable_all()
                .build()
                .expect("a runtime");
            rt.block_on(async move {
                let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
                ready_tx.send(listener.local_addr().expect("addr")).ok();
                serve(listener, served, stop).await;
            });
        });
        let addr = ready_rx.recv().expect("the fake broker started");
        Self {
            addr,
            state,
            shutdown: Some(shutdown),
        }
    }

    /// `127.0.0.1:<port>`, as the session route's `broker_url`.
    pub fn broker_url(&self) -> String {
        self.addr.to_string()
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn push_exec(&self, script: ExecScript) {
        self.state.lock().unwrap().exec.push_back(script);
    }

    pub fn push_logs(&self, script: LogsScript) {
        self.state.lock().unwrap().logs.push_back(script);
    }

    pub fn push_port(&self, script: PortScript) {
        self.state.lock().unwrap().port.push_back(script);
    }

    /// Appends a chunk to the ring buffer, stamped after every earlier one.
    pub fn log(&self, data: &[u8]) -> Timestamp {
        let mut s = self.state.lock().unwrap();
        append(&mut s.ring, data)
    }

    pub fn calls(&self) -> Vec<Call> {
        self.state.lock().unwrap().calls.clone()
    }

    /// TCP connections accepted.
    pub fn connections(&self) -> usize {
        self.state.lock().unwrap().connections
    }

    /// TLS handshakes the client abandoned or failed.
    pub fn failed_handshakes(&self) -> usize {
        self.state.lock().unwrap().failed_handshakes
    }

    /// Waits until `done` holds of the calls, and returns them; panics after
    /// `timeout`.
    pub fn wait_for(&self, timeout: Duration, done: impl Fn(&[Call]) -> bool) -> Vec<Call> {
        let deadline = Instant::now() + timeout;
        loop {
            let calls = self.calls();
            if done(&calls) {
                return calls;
            }
            assert!(
                Instant::now() < deadline,
                "the fake broker waited {timeout:?} in vain; it saw {calls:#?}"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for FakeBroker {
    fn drop(&mut self) {
        if let Some(s) = self.shutdown.take() {
            let _ = s.send(());
        }
    }
}

fn append(ring: &mut Vec<LogChunk>, data: &[u8]) -> Timestamp {
    let i = ring.len() as i64;
    let at = Timestamp {
        seconds: 1_793_750_400 + i,
        nanos: 500_000_000 + i as i32,
    };
    ring.push(LogChunk {
        data: data.to_vec(),
        at: Some(at),
        stream: Stream::Stdout as i32,
        ..Default::default()
    });
    at
}

fn at_or_after(at: &Option<Timestamp>, since: &Option<Timestamp>) -> bool {
    match (at, since) {
        (_, None) => true,
        (None, Some(_)) => false,
        (Some(a), Some(s)) => (a.seconds, a.nanos) >= (s.seconds, s.nanos),
    }
}

fn tls_acceptor() -> TlsAcceptor {
    let pki = test_pki();
    let certs = vec![pki.cert.clone()];
    let key = rustls::pki_types::PrivatePkcs8KeyDer::from(pki.key.clone()).into();
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .expect("protocol versions")
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .expect("the test certificate and key");
    config.alpn_protocols = vec![b"h2".to_vec()];
    TlsAcceptor::from(Arc::new(config))
}

async fn serve(listener: TcpListener, state: State, stop: oneshot::Receiver<()>) {
    let acceptor = tls_acceptor();
    let (conn_tx, conn_rx) = mpsc::channel::<Result<Tls, std::io::Error>>(16);
    let accepting = state.clone();
    tokio::spawn(async move {
        let mut next_id = 0u64;
        loop {
            let Ok((tcp, _)) = listener.accept().await else {
                continue;
            };
            next_id += 1;
            let id = next_id;
            // A second handle on the socket, so a step can close it.
            let Ok(std_tcp) = tcp.into_std() else {
                continue;
            };
            let Ok(handle) = std_tcp.try_clone() else {
                continue;
            };
            let Ok(tcp) = TcpStream::from_std(std_tcp) else {
                continue;
            };
            {
                let mut s = accepting.lock().unwrap();
                s.connections += 1;
                s.conns.insert(id, handle);
            }
            let acceptor = acceptor.clone();
            let conn_tx = conn_tx.clone();
            let state = accepting.clone();
            tokio::spawn(async move {
                match acceptor.accept(tcp).await {
                    Ok(inner) => {
                        let _ = conn_tx.send(Ok(Tls { inner, id })).await;
                    }
                    Err(_) => state.lock().unwrap().failed_handshakes += 1,
                }
            });
        }
    });
    let service = Arc::new(Broker { state });
    let _ = tonic::transport::Server::builder()
        .layer(tower::layer::layer_fn(RecordEnd))
        .serve_with_incoming_shutdown(
            SessionServiceServer::new(service),
            ReceiverStream::new(conn_rx),
            async {
                let _ = stop.await;
            },
        )
        .await;
}

/// How a call's request body ended on the wire. tonic's server reads a
/// client's `RST_STREAM(CANCEL)` as a clean end, which the real broker does
/// not, so the fake reads the body beneath tonic.
#[derive(Clone, Default)]
struct BodyEnd(Arc<Mutex<Option<bool>>>);

impl BodyEnd {
    /// `Some(true)` for `END_STREAM`, `Some(false)` for a reset or error.
    fn get(&self) -> Option<bool> {
        *self.0.lock().unwrap()
    }
}

#[derive(Clone)]
struct RecordEnd<S>(S);

impl<S> tower::Service<http::Request<tonic::body::Body>> for RecordEnd<S>
where
    S: tower::Service<http::Request<tonic::body::Body>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), S::Error>> {
        self.0.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<tonic::body::Body>) -> S::Future {
        let end = BodyEnd::default();
        let (mut parts, body) = req.into_parts();
        parts.extensions.insert(end.clone());
        let body = tonic::body::Body::new(Recording { inner: body, end });
        self.0.call(http::Request::from_parts(parts, body))
    }
}

struct Recording {
    inner: tonic::body::Body,
    end: BodyEnd,
}

impl http_body::Body for Recording {
    type Data = bytes::Bytes;
    type Error = Status;

    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<http_body::Frame<bytes::Bytes>, Status>>> {
        let polled = Pin::new(&mut self.inner).poll_frame(cx);
        let ended = match &polled {
            Poll::Ready(None) => Some(true),
            Poll::Ready(Some(Err(_))) => Some(false),
            _ => None,
        };
        if let Some(clean) = ended {
            self.end.0.lock().unwrap().get_or_insert(clean);
        }
        polled
    }
}

/// The id of the connection a call arrived on.
#[derive(Clone, Copy, Debug)]
struct ConnId(u64);

struct Tls {
    inner: tokio_rustls::server::TlsStream<TcpStream>,
    id: u64,
}

impl Connected for Tls {
    type ConnectInfo = ConnId;

    fn connect_info(&self) -> ConnId {
        ConnId(self.id)
    }
}

impl AsyncRead for Tls {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl AsyncWrite for Tls {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

struct Broker {
    state: State,
}

type Out<T> = mpsc::Sender<Result<T, Status>>;

impl Broker {
    fn record<T>(&self, method: &'static str, req: &Request<T>) -> (usize, Option<u64>) {
        let authorization = req
            .metadata()
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);
        let conn = req.extensions().get::<ConnId>().map(|c| c.0);
        let mut s = self.state.lock().unwrap();
        s.calls.push(Call {
            method,
            authorization,
            ..Default::default()
        });
        (s.calls.len() - 1, conn)
    }

    fn close(&self, conn: Option<u64>) {
        let s = self.state.lock().unwrap();
        if let Some(tcp) = conn.and_then(|c| s.conns.get(&c)) {
            let _ = tcp.shutdown(std::net::Shutdown::Both);
        }
    }
}

fn admitted<T>(
    rx: mpsc::Receiver<Result<T, Status>>,
) -> Response<ReceiverStream<Result<T, Status>>> {
    let mut response = Response::new(ReceiverStream::new(rx));
    response
        .metadata_mut()
        .insert(ADMITTED_HEADER, "1".parse().expect("ascii"));
    response
}

/// Runs `script` until it ends or the client goes away, and records which.
async fn supervise<T>(
    state: State,
    idx: usize,
    out: &Out<T>,
    script: impl std::future::Future<Output = ()>,
) {
    tokio::select! {
        _ = script => update(&state, idx, |c| c.finished = true),
        _ = out.closed() => update(&state, idx, |c| c.client_gone = true),
    }
}

fn data(stream: Stream, chunk: Vec<u8>) -> Result<ExecServerFrame, Status> {
    Ok(ExecServerFrame {
        frame: Some(exec_server_frame::Frame::Data(ExecData {
            stream: stream as i32,
            chunk,
        })),
    })
}

/// An exec's output queue toward the client.
struct Output {
    out: Out<ExecServerFrame>,
    tty: bool,
    state: State,
    idx: usize,
}

impl Output {
    /// Queues a data frame; false once the session has ended.
    async fn data(&self, stream: Stream, chunk: Vec<u8>) -> bool {
        let len = chunk.len();
        let queued = || self.out.max_capacity() - self.out.capacity();
        if self.tty {
            if queued() >= QUEUE_DEPTH {
                let _ = self
                    .out
                    .send(Err(broker_status(
                        Code::Unavailable,
                        crate::stream::status::reason::RATE_LIMITED,
                        "client fell behind the sandbox's output; reconnect",
                        &[],
                    )))
                    .await;
                return false;
            }
        } else {
            while queued() >= OUTPUT_WINDOW {
                if self.out.is_closed() {
                    return false;
                }
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }
        if self.out.send(data(stream, chunk)).await.is_err() {
            return false;
        }
        update(&self.state, self.idx, |c| c.sent += len);
        true
    }
}

async fn run_exec(
    broker: Arc<Broker>,
    idx: usize,
    conn: Option<u64>,
    steps: Vec<Step>,
    mut inbound: mpsc::Receiver<ExecClientFrame>,
    out: Out<ExecServerFrame>,
) {
    use exec_client_frame::Frame;
    let script = async {
        let mut count = 0usize;
        let mut output = Output {
            out: out.clone(),
            tty: false,
            state: broker.state.clone(),
            idx,
        };
        for step in steps {
            match step {
                Step::Start => loop {
                    match inbound.recv().await {
                        Some(ExecClientFrame {
                            frame: Some(Frame::Start(s)),
                        }) => {
                            output.tty = s.tty;
                            break;
                        }
                        Some(_) => {}
                        None => return,
                    }
                },
                Step::Stdout(b) => {
                    if !output.data(Stream::Stdout, b).await {
                        return;
                    }
                }
                Step::Stderr(b) => {
                    if !output.data(Stream::Stderr, b).await {
                        return;
                    }
                }
                Step::StdoutForever(b, every) => loop {
                    if !output.data(Stream::Stdout, b.clone()).await {
                        std::future::pending::<()>().await;
                    }
                    tokio::time::sleep(every).await;
                },
                Step::StdoutBulk(total, size) => {
                    let mut left = total;
                    while left > 0 {
                        let n = left.min(size);
                        if !output.data(Stream::Stdout, vec![b'b'; n]).await {
                            std::future::pending::<()>().await;
                        }
                        left -= n;
                    }
                }
                Step::Echo | Step::Count | Step::StdinEof => loop {
                    match inbound.recv().await.and_then(|f| f.frame) {
                        Some(Frame::Stdin(b)) => {
                            count += b.len();
                            if matches!(step, Step::Echo) && !output.data(Stream::Stdout, b).await {
                                return;
                            }
                        }
                        Some(Frame::StdinEof(_)) => break,
                        Some(_) => {}
                        None => std::future::pending::<()>().await,
                    }
                },
                Step::PrintCount => {
                    let _ = output
                        .data(Stream::Stdout, format!("{count}\n").into_bytes())
                        .await;
                }
                Step::Sleep(d) => tokio::time::sleep(d).await,
                Step::Exit(code) => {
                    let _ = out
                        .send(Ok(ExecServerFrame {
                            frame: Some(exec_server_frame::Frame::ExitCode(code)),
                        }))
                        .await;
                }
                Step::Fail(status) => {
                    let _ = out.send(Err(status)).await;
                }
                Step::Close => broker.close(conn),
                Step::Hold => std::future::pending::<()>().await,
            }
        }
    };
    supervise(broker.state.clone(), idx, &out, script).await;
}

#[tonic::async_trait]
impl SessionService for Arc<Broker> {
    type ExecStream = ReceiverStream<Result<ExecServerFrame, Status>>;
    type LogsStream = ReceiverStream<Result<LogChunk, Status>>;
    type PortForwardStream = ReceiverStream<Result<PortServerFrame, Status>>;
    type InvokeStream = ReceiverStream<Result<InvokeServerFrame, Status>>;

    async fn exec(
        &self,
        req: Request<Streaming<ExecClientFrame>>,
    ) -> Result<Response<Self::ExecStream>, Status> {
        let (idx, conn) = self.record("Exec", &req);
        let script = self.state.lock().unwrap().exec.pop_front();
        let script =
            script.unwrap_or_else(|| ExecScript::refuse(Status::unimplemented("no Exec script")));
        let body_end = req
            .extensions()
            .get::<BodyEnd>()
            .cloned()
            .unwrap_or_default();
        let mut inbound = req.into_inner();
        if let Answer::Stall = script.answer {
            std::future::pending::<()>().await;
        }
        if let Answer::Drop = script.answer {
            self.close(conn);
            std::future::pending::<()>().await;
        }
        if let Answer::Refuse(status) = script.answer.clone() {
            // Anything sent before admission would arrive in this window.
            let window = tokio::time::sleep(Duration::from_millis(150));
            tokio::pin!(window);
            loop {
                tokio::select! {
                    _ = &mut window => break,
                    m = inbound.message() => match m {
                        Ok(Some(_)) => update(&self.state, idx, |c| c.frames_before_admission += 1),
                        _ => break,
                    },
                }
            }
            return Err(status);
        }
        let header = matches!(script.answer, Answer::Admit);
        update(&self.state, idx, |c| c.admitted = header);

        // A bounded queue: once the steps stop taking frames, the reader
        // stops reading and HTTP/2 flow control holds the client back.
        let (in_tx, in_rx) = mpsc::channel(INPUT_WINDOW);
        let state = self.state.clone();
        tokio::spawn(async move {
            loop {
                match inbound.message().await {
                    Ok(Some(f)) => {
                        update(&state, idx, |c| {
                            c.frames += 1;
                            match &f.frame {
                                Some(exec_client_frame::Frame::Start(s)) => {
                                    c.start = Some(s.clone())
                                }
                                Some(exec_client_frame::Frame::Stdin(b)) => {
                                    c.stdin.extend_from_slice(b)
                                }
                                Some(exec_client_frame::Frame::StdinEof(_)) => c.stdin_eof = true,
                                Some(exec_client_frame::Frame::Resize(r)) => {
                                    c.resizes.push((r.rows, r.cols))
                                }
                                _ => {}
                            }
                        });
                        // Once the steps end, frames are read and dropped.
                        let _ = in_tx.send(f).await;
                    }
                    Ok(None) if body_end.get() == Some(true) => {
                        update(&state, idx, |c| c.half_closed = true);
                        break;
                    }
                    Ok(None) => {
                        update(&state, idx, |c| {
                            c.reset = Some(Code::Cancelled);
                            if !c.finished {
                                c.client_gone = true;
                            }
                        });
                        break;
                    }
                    Err(status) => {
                        update(&state, idx, |c| {
                            c.reset = Some(status.code());
                            if !c.finished {
                                c.client_gone = true;
                            }
                        });
                        break;
                    }
                }
            }
        });
        // One slot past the deepest queue, for the status that ends it.
        let (out, rx) = mpsc::channel(QUEUE_DEPTH + 1);
        tokio::spawn(run_exec(self.clone(), idx, conn, script.steps, in_rx, out));
        if header {
            Ok(admitted(rx))
        } else {
            Ok(Response::new(ReceiverStream::new(rx)))
        }
    }

    async fn logs(&self, req: Request<LogsRequest>) -> Result<Response<Self::LogsStream>, Status> {
        let (idx, conn) = self.record("Logs", &req);
        let request = req.into_inner();
        let script = {
            let mut s = self.state.lock().unwrap();
            s.calls[idx].logs = Some(request);
            s.logs.pop_front()
        }
        .unwrap_or_else(|| LogsScript::admit(LogsEnd::Default));
        if let Some(status) = script.refuse {
            return Err(status);
        }
        update(&self.state, idx, |c| c.admitted = true);
        let replay: Vec<LogChunk> = {
            let mut s = self.state.lock().unwrap();
            let replay = s
                .ring
                .iter()
                .filter(|c| at_or_after(&c.at, &request.since))
                .cloned()
                .collect();
            for data in &script.append {
                append(&mut s.ring, data);
            }
            replay
        };
        let (out, rx) = mpsc::channel(16);
        let broker = self.clone();
        tokio::spawn(async move {
            let run = async {
                for chunk in replay {
                    let _ = out.send(Ok(chunk)).await;
                }
                match script.end {
                    // Dropping `out` ends the call with OK trailers.
                    LogsEnd::Default if !request.follow => {}
                    LogsEnd::Default | LogsEnd::Hold => std::future::pending::<()>().await,
                    LogsEnd::Fail(status) => {
                        let _ = out.send(Err(status)).await;
                    }
                    LogsEnd::Close => broker.close(conn),
                }
            };
            supervise(broker.state.clone(), idx, &out, run).await;
        });
        Ok(admitted(rx))
    }

    async fn port_forward(
        &self,
        req: Request<Streaming<PortClientFrame>>,
    ) -> Result<Response<Self::PortForwardStream>, Status> {
        let (idx, _) = self.record("PortForward", &req);
        let script = self.state.lock().unwrap().port.pop_front();
        let script = script
            .unwrap_or_else(|| PortScript::refuse(Status::unimplemented("no PortForward script")));
        if let Some(status) = script.refuse {
            return Err(status);
        }
        update(&self.state, idx, |c| c.admitted = true);
        let body_end = req
            .extensions()
            .get::<BodyEnd>()
            .cloned()
            .unwrap_or_default();
        let mut inbound = req.into_inner();
        let state = self.state.clone();
        tokio::spawn(async move {
            loop {
                match inbound.message().await {
                    Ok(Some(_)) => update(&state, idx, |c| c.frames += 1),
                    Ok(None) if body_end.get() == Some(true) => {
                        update(&state, idx, |c| c.half_closed = true);
                        break;
                    }
                    Ok(None) => {
                        update(&state, idx, |c| c.reset = Some(Code::Cancelled));
                        break;
                    }
                    Err(status) => {
                        update(&state, idx, |c| c.reset = Some(status.code()));
                        break;
                    }
                }
            }
        });
        let (out, rx) = mpsc::channel(16);
        let state = self.state.clone();
        tokio::spawn(async move {
            supervise(state, idx, &out, std::future::pending::<()>()).await;
        });
        Ok(admitted(rx))
    }

    async fn invoke(
        &self,
        _: Request<Streaming<InvokeClientFrame>>,
    ) -> Result<Response<Self::InvokeStream>, Status> {
        Err(Status::unimplemented("not faked"))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::SystemTime;

    use super::*;
    use crate::session::{MintFuture, Minted, Session, SessionMinter};
    use crate::stream::calls::{self, Open, OpenError};
    use crate::stream::transport::TransportConfig;

    struct Minter {
        url: String,
        mints: AtomicUsize,
    }

    impl SessionMinter for &Minter {
        type Error = ();
        fn mint(&self) -> MintFuture<'_, ()> {
            self.mints.fetch_add(1, Ordering::SeqCst);
            let now = SystemTime::now();
            let minted = Minted {
                token: format!("t{}", self.mints.load(Ordering::SeqCst)),
                expires_at: now + Duration::from_secs(600),
                date: Some(now),
                broker_url: self.url.clone(),
            };
            Box::pin(async move { Ok(minted) })
        }
    }

    fn session(m: &Minter) -> Session<&Minter> {
        Session::new(
            m,
            TransportConfig {
                extra_roots_pem: vec![ca_pem().as_bytes().to_vec()],
                user_agent: "test".into(),
                ..Default::default()
            },
        )
    }

    fn minter(broker: &FakeBroker) -> Minter {
        Minter {
            url: broker.broker_url(),
            mints: AtomicUsize::new(0),
        }
    }

    fn frame(f: exec_client_frame::Frame) -> ExecClientFrame {
        ExecClientFrame { frame: Some(f) }
    }

    #[tokio::test]
    async fn a_call_refused_before_admission_sent_nothing_and_its_retry_runs() {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::refuse(broker_status(
            Code::Unavailable,
            "sandbox_disconnected",
            "not yet",
            &[],
        )));
        broker.push_exec(ExecScript::admit([Step::Start, Step::Echo, Step::Exit(3)]));
        let m = minter(&broker);
        let s = session(&m);
        let mut call = calls::open_exec(&s, &Open::default()).await.ok().unwrap();
        use exec_client_frame::Frame;
        call.send
            .send(frame(Frame::Start(ExecStart::default())))
            .await
            .unwrap();
        call.send
            .send(frame(Frame::Stdin(b"hi".to_vec())))
            .await
            .unwrap();
        call.send.send(frame(Frame::StdinEof(true))).await.unwrap();
        let mut got = Vec::new();
        while let Some(f) = call.recv.message().await.unwrap() {
            got.push(f.frame.unwrap());
        }
        assert_eq!(
            got,
            [
                exec_server_frame::Frame::Data(ExecData {
                    stream: Stream::Stdout as i32,
                    chunk: b"hi".to_vec()
                }),
                exec_server_frame::Frame::ExitCode(3)
            ]
        );
        let seen = broker.calls();
        assert_eq!(seen[0].frames_before_admission, 0);
        assert!(!seen[0].admitted);
        assert_eq!(seen[1].authorization.as_deref(), Some("Bearer t2"));
        // sandbox_disconnected invalidates the session.
        assert_eq!(m.mints.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn unauthenticated_mints_once_more_and_a_second_is_returned() {
        let broker = FakeBroker::start();
        for _ in 0..3 {
            broker.push_exec(ExecScript::refuse(broker_status(
                Code::Unauthenticated,
                "unauthenticated",
                "unknown key",
                &[],
            )));
        }
        let m = minter(&broker);
        let s = session(&m);
        match calls::open_exec(&s, &Open::default()).await {
            Err(OpenError::Refused(st)) => assert!(st.is("unauthenticated")),
            _ => panic!("expected a refusal"),
        }
        assert_eq!(m.mints.load(Ordering::SeqCst), 2);
        assert_eq!(broker.calls().len(), 2);
    }

    #[tokio::test]
    async fn headers_without_the_admitted_header_are_retried_having_sent_nothing() {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::unadmitted([Step::Start, Step::Exit(9)]));
        broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
        let m = minter(&broker);
        let s = session(&m);
        let call = calls::open_exec(&s, &Open::default()).await.ok().unwrap();
        drop(call);
        let seen = broker.calls();
        assert_eq!(seen.len(), 2);
        assert_eq!(seen[0].frames, 0);
        assert!(seen[1].admitted);
    }

    #[tokio::test]
    async fn an_open_whose_headers_never_come_gives_up_at_its_deadline() {
        let broker = FakeBroker::start();
        for _ in 0..10 {
            broker.push_exec(ExecScript::stall());
        }
        let m = minter(&broker);
        let s = session(&m);
        let open = Open {
            retry_for: Duration::from_secs(1),
            attempt: Duration::from_millis(200),
            ..Default::default()
        };
        let opened = tokio::time::timeout(Duration::from_secs(10), calls::open_exec(&s, &open))
            .await
            .expect("the open hung");
        match opened {
            Err(OpenError::Refused(st)) => assert_eq!(st.code, Code::DeadlineExceeded),
            _ => panic!("expected a refusal"),
        }
        assert!(!broker.calls().is_empty());
    }

    #[tokio::test]
    async fn sandbox_disconnected_invalidates_the_session_without_a_retry() {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::refuse(broker_status(
            Code::Unavailable,
            "sandbox_disconnected",
            "not yet",
            &[],
        )));
        broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
        let m = minter(&broker);
        let s = session(&m);
        let once = Open {
            retry_for: Duration::ZERO,
            ..Default::default()
        };
        assert!(calls::open_exec(&s, &once).await.is_err());
        assert!(calls::open_exec(&s, &Open::default()).await.is_ok());
        assert_eq!(m.mints.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn repeated_sandbox_disconnected_mints_again_at_most_once_per_floor() {
        let broker = FakeBroker::start();
        for _ in 0..3 {
            broker.push_exec(ExecScript::refuse(broker_status(
                Code::Unavailable,
                "sandbox_disconnected",
                "not yet",
                &[],
            )));
        }
        broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
        let m = minter(&broker);
        let s = session(&m);
        assert!(calls::open_exec(&s, &Open::default()).await.is_ok());
        assert_eq!(broker.calls().len(), 4);
        // The first refusal re-mints; the next two, within the floor, do not.
        assert_eq!(m.mints.load(Ordering::SeqCst), 2);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dropping_a_call_resets_it_rather_than_ending_its_input() {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let m = minter(&broker);
        let s = session(&m);
        use exec_client_frame::Frame;
        let wait = Duration::from_secs(10);
        let ended = |c: &Call| c.reset.is_some() || c.half_closed;

        // The sender alone, with the response still held.
        let call = calls::open_exec(&s, &Open::default()).await.ok().unwrap();
        call.send
            .send(frame(Frame::Start(ExecStart::default())))
            .await
            .unwrap();
        broker.wait_for(wait, |c| c[0].start.is_some());
        drop(call.send);
        let seen = broker.wait_for(wait, |c| ended(&c[0]));
        assert!(
            seen[0].reset.is_some() && !seen[0].half_closed,
            "{:?}",
            seen[0]
        );
        drop(call.recv);

        // `cancel`, which drops both halves.
        let call = calls::open_exec(&s, &Open::default()).await.ok().unwrap();
        call.send
            .send(frame(Frame::Start(ExecStart::default())))
            .await
            .unwrap();
        broker.wait_for(wait, |c| c.len() == 2 && c[1].start.is_some());
        call.cancel();
        let seen = broker.wait_for(wait, |c| ended(&c[1]));
        assert!(
            seen[1].reset.is_some() && !seen[1].half_closed,
            "{:?}",
            seen[1]
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dropping_the_senders_of_a_call_with_no_window_left_reaches_the_broker() {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let m = minter(&broker);
        let s = session(&m);
        use exec_client_frame::Frame;
        let call = calls::open_exec(&s, &Open::default()).await.ok().unwrap();
        call.send
            .send(frame(Frame::Start(ExecStart::default())))
            .await
            .unwrap();
        // Stdin the broker never reads, until the stream's window is gone and
        // the sender's buffer is full.
        let chunk = || frame(Frame::Stdin(vec![7u8; 32 * 1024]));
        while tokio::time::timeout(Duration::from_millis(500), call.send.send(chunk()))
            .await
            .is_ok()
        {}
        drop(call.send);
        let seen = broker.wait_for(Duration::from_secs(3), |c| c[0].client_gone);
        assert!(!seen[0].half_closed, "{:?}", seen[0]);
        drop(call.recv);
    }

    /// Opens an Exec call, sends `start`, and fills its stream's window and
    /// its sender's buffer with stdin the broker never reads.
    async fn stuck_call(s: &Session<&Minter>, broker: &FakeBroker, idx: usize) -> calls::ExecCall {
        use exec_client_frame::Frame;
        let call = calls::open_exec(s, &Open::default()).await.ok().unwrap();
        call.send
            .send(frame(Frame::Start(ExecStart::default())))
            .await
            .unwrap();
        broker.wait_for(Duration::from_secs(10), |c| {
            c.get(idx).is_some_and(|c| c.start.is_some())
        });
        let chunk = || frame(Frame::Stdin(vec![7u8; 32 * 1024]));
        while tokio::time::timeout(Duration::from_millis(500), call.send.send(chunk()))
            .await
            .is_ok()
        {}
        call
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_call_whose_reset_went_out_leaves_its_connection_open() {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        broker.push_exec(ExecScript::admit([Step::Start, Step::Exit(0)]));
        let m = minter(&broker);
        let s = session(&m);
        use exec_client_frame::Frame;
        let call = calls::open_exec(&s, &Open::default()).await.ok().unwrap();
        call.send
            .send(frame(Frame::Start(ExecStart::default())))
            .await
            .unwrap();
        broker.wait_for(Duration::from_secs(10), |c| c[0].start.is_some());
        call.cancel();
        broker.wait_for(Duration::from_secs(10), |c| c[0].reset.is_some());
        tokio::time::sleep(Duration::from_secs(1)).await;
        let next = calls::open_exec(&s, &Open::default()).await.ok().unwrap();
        drop(next);
        assert_eq!(broker.connections(), 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_stuck_call_leaves_open_a_connection_another_exec_uses() {
        let broker = FakeBroker::start();
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let m = minter(&broker);
        let s = session(&m);
        use exec_client_frame::Frame;
        let mut other = calls::open_exec(&s, &Open::default()).await.ok().unwrap();
        other
            .send
            .send(frame(Frame::Start(ExecStart::default())))
            .await
            .unwrap();
        let stuck = stuck_call(&s, &broker, 1).await;
        drop(stuck.send);
        let answered = tokio::time::timeout(Duration::from_secs(1), other.recv.message()).await;
        assert!(answered.is_err(), "the other call ended: {answered:?}");
        assert!(!broker.calls()[0].client_gone);
        drop(stuck.recv);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_stuck_call_leaves_open_a_connection_a_logs_call_uses() {
        let broker = FakeBroker::start();
        broker.push_logs(LogsScript::admit(LogsEnd::Hold));
        broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
        let m = minter(&broker);
        let s = session(&m);
        let request = LogsRequest {
            since: None,
            follow: true,
        };
        let mut logs = calls::open_logs(&s, request, &Open::default())
            .await
            .ok()
            .unwrap();
        let stuck = stuck_call(&s, &broker, 1).await;
        drop(stuck.send);
        let answered = tokio::time::timeout(Duration::from_secs(1), logs.message()).await;
        assert!(answered.is_err(), "the logs call ended: {answered:?}");
        drop(stuck.recv);
    }

    #[tokio::test]
    async fn logs_replays_from_since_inclusively() {
        let broker = FakeBroker::start();
        broker.log(b"a");
        let b = broker.log(b"b");
        broker.log(b"c");
        let m = minter(&broker);
        let s = session(&m);
        let request = LogsRequest {
            since: Some(b),
            follow: false,
        };
        let mut stream = calls::open_logs(&s, request, &Open::default())
            .await
            .ok()
            .unwrap();
        let mut data = Vec::new();
        while let Some(chunk) = stream.message().await.unwrap() {
            if chunk.eof {
                break;
            }
            data.extend(chunk.data);
        }
        assert_eq!(data, b"bc");
    }
}
