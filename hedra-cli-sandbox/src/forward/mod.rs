//! `port-forward`: listens on `127.0.0.1` and `[::1]` and carries each
//! accepted connection over a `PortForward` call of its own.
//!
//! The caller binds with [`bind`], prints [`Bound::port`], and runs [`serve`]
//! with a [`PortOpener`] that opens and admits one call per connection.
//! Dropping the `serve` future closes the listeners and cancels every call.

mod budget;
#[cfg(test)]
mod tests;

use std::future::Future;
use std::io;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use bytes::Bytes;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Semaphore};
use tokio::task::JoinSet;

use crate::pb::hedra::sandbox::v1::{
    port_client_frame, port_server_frame, PortClientFrame, PortServerFrame,
};

use crate::stream::calls::CallClosed;
use budget::Budget;

/// The most one `data` frame carries.
const CHUNK: usize = 32 << 10;

/// The sending half of an admitted `PortForward` call.
pub trait PortSink: Send + 'static {
    /// Sends one frame; an error means the call ended, and the source says why.
    fn send(
        &mut self,
        frame: PortClientFrame,
    ) -> impl Future<Output = Result<(), CallClosed>> + Send;
}

/// The receiving half of an admitted `PortForward` call. A connection
/// cancels its call by dropping the source, then the sink.
pub trait PortSource: Send + 'static {
    /// The call's failure.
    type Error: ForwardError;
    /// The next frame, or `None` when the call ended without a status.
    fn recv(&mut self)
        -> impl Future<Output = Result<Option<PortServerFrame>, Self::Error>> + Send;
}

/// A failure of one forwarded connection's call.
pub trait ForwardError: std::fmt::Display + Send + 'static {
    /// Whether no later call can succeed, so the listener closes.
    fn ends_listener(&self) -> bool;
}

/// Opens one admitted `PortForward` call per accepted connection.
pub trait PortOpener: Send + Sync + 'static {
    /// The call's failure.
    type Error: ForwardError;
    /// The sending half.
    type Sink: PortSink;
    /// The receiving half.
    type Source: PortSource<Error = Self::Error>;
    /// Opens a call and waits for its admission; nothing is sent on it yet.
    fn open(&self) -> impl Future<Output = Result<(Self::Sink, Self::Source), Self::Error>> + Send;
}

impl PortSink for mpsc::Sender<PortClientFrame> {
    async fn send(&mut self, frame: PortClientFrame) -> Result<(), CallClosed> {
        mpsc::Sender::send(self, frame)
            .await
            .map_err(|_| CallClosed)
    }
}

impl PortSink for crate::stream::calls::Sender<PortClientFrame> {
    async fn send(&mut self, frame: PortClientFrame) -> Result<(), CallClosed> {
        mpsc::Sender::send(self, frame)
            .await
            .map_err(|_| CallClosed)
    }
}

/// The invocation's limits.
#[derive(Debug, Clone, Copy)]
pub struct Limits {
    /// Open connections at once; one more waits for one to close.
    pub max_connections: usize,
    /// Bytes buffered for one connection's local socket.
    pub connection_buffer: usize,
    /// Bytes buffered across every connection.
    pub total_buffer: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            max_connections: 90,
            connection_buffer: 8 << 20,
            total_buffer: 64 << 20,
        }
    }
}

/// An address family the forward listens on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// `127.0.0.1`.
    V4,
    /// `[::1]`.
    V6,
}

impl Family {
    fn addr(self, port: u16) -> SocketAddr {
        match self {
            Family::V4 => (Ipv4Addr::LOCALHOST, port).into(),
            Family::V6 => (Ipv6Addr::LOCALHOST, port).into(),
        }
    }
}

impl std::fmt::Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Family::V4 => "IPv4 (127.0.0.1)",
            Family::V6 => "IPv6 ([::1])",
        })
    }
}

/// A family that could not be bound for want of it; the other serves.
#[derive(Debug)]
pub struct BindWarning {
    /// The family left out.
    pub family: Family,
    /// Why.
    pub error: io::Error,
}

impl std::fmt::Display for BindWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not listening on {}: {}", self.family, self.error)
    }
}

/// A bind that failed the port-forward.
#[derive(Debug)]
pub struct BindError {
    /// The family that failed.
    pub family: Family,
    /// The port asked for.
    pub port: u16,
    /// Why.
    pub error: io::Error,
}

impl std::fmt::Display for BindError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "cannot listen on port {} over {}: {}",
            self.port, self.family, self.error
        )
    }
}

impl std::error::Error for BindError {}

/// The bound listeners.
#[derive(Debug)]
pub struct Bound {
    listeners: Vec<TcpListener>,
    /// The local port, the chosen one when 0 was asked for.
    pub port: u16,
    /// Families left out.
    pub warnings: Vec<BindWarning>,
}

impl Bound {
    /// The families listening.
    pub fn families(&self) -> Vec<Family> {
        [Family::V4, Family::V6]
            .into_iter()
            .filter(|f| !self.warnings.iter().any(|w| w.family == *f))
            .collect()
    }
}

/// Whether a bind error means the family is missing, which is a warning
/// while the other family serves.
fn family_missing(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::AddrNotAvailable || e.raw_os_error() == Some(afnosupport())
}

/// The OS's error code for an address family it does not support.
fn afnosupport() -> i32 {
    #[cfg(unix)]
    return rustix::io::Errno::AFNOSUPPORT.raw_os_error();
    #[cfg(windows)]
    return 10047; // WSAEAFNOSUPPORT
    #[cfg(not(any(unix, windows)))]
    return -1;
}

/// Port 0's tries at a port free on both families.
const PORT_ZERO_TRIES: usize = 20;

/// Binds `127.0.0.1:<port>` and `[::1]:<port>`; for port 0, the port the
/// IPv4 bind chose, retrying when it is taken on IPv6.
pub async fn bind(port: u16) -> Result<Bound, BindError> {
    let bound = bind_with(port, |addr| bind_exclusive(addr, port != 0))?;
    #[cfg(any(windows, target_os = "macos"))]
    refuse_a_wildcard_holder(bound.port)?;
    Ok(bound)
}

/// Fails when another socket holds `port` on a wildcard address, which our
/// loopback binds do not refuse on Windows or macOS, while that process's
/// local clients, with their requests and tokens, would then reach the
/// sandbox.
///
/// Windows lets our `SO_EXCLUSIVEADDRUSE` bind of `127.0.0.1:P` succeed
/// beside a same-user `0.0.0.0:P` or `[::]:P` bound without that option,
/// and sends the connections to the more specific address to us. A plain
/// wildcard bind of our own, which never listens, fails beside any wildcard
/// holder and succeeds beside sockets bound only to a specific address,
/// ours included (the Windows Server 2003 and later tables in Microsoft
/// Learn's "Using SO_REUSEADDR and SO_EXCLUSIVEADDRUSE": a default wildcard
/// bind after a specific one with `SO_EXCLUSIVEADDRUSE` succeeds, for the
/// same user or another).
///
/// macOS binds a fixed port's loopback addresses with `SO_REUSEADDR`, so a
/// connection the last run closed, in `TIME_WAIT` for about 30 s, does not
/// hold the port; that option also lets the bind succeed beside a wildcard
/// holder. The probe there sets `SO_REUSEADDR` too, so it binds beside our
/// own sockets and a `TIME_WAIT`, and fails beside a wildcard holder,
/// whatever options the holder set.
///
/// It runs after the loopback binds, since port 0's port is known only
/// then, and probes `0.0.0.0:P`, `[::]:P` with `IPV6_V6ONLY`, and a
/// dual-stack `[::]:P`: the first fails beside a `0.0.0.0` holder, the
/// second beside an IPv6-only `[::]` one, and the third beside a dual-stack
/// `[::]` one, which on Windows lets both of the first two bind.
#[cfg(any(windows, target_os = "macos"))]
fn refuse_a_wildcard_holder(port: u16) -> Result<(), BindError> {
    use socket2::{Domain, Protocol, Socket, Type};
    let probes = [
        (Family::V4, None),
        (Family::V6, Some(true)),
        (Family::V6, Some(false)),
    ];
    for (family, only_v6) in probes {
        let addr: SocketAddr = match family {
            Family::V4 => (Ipv4Addr::UNSPECIFIED, port).into(),
            Family::V6 => (Ipv6Addr::UNSPECIFIED, port).into(),
        };
        let probe = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))
            .and_then(|s| {
                #[cfg(target_os = "macos")]
                s.set_reuse_address(true)?;
                if let Some(only) = only_v6 {
                    s.set_only_v6(only)?;
                }
                s.bind(&addr.into())
            });
        match probe {
            Ok(()) => {}
            // A family the host lacks was already left out with a warning.
            Err(e) if family_missing(&e) => {}
            Err(error) => {
                return Err(BindError {
                    family,
                    port,
                    error,
                })
            }
        }
    }
    Ok(())
}

/// Binds so that another socket on the port fails the bind. Linux refuses a
/// bind beside another socket on the port with or without `SO_REUSEADDR`,
/// so sets it, for a port with connections in `TIME_WAIT`. macOS sets it
/// only for a fixed port, since it lets the bind succeed beside a wildcard
/// holder, which `bind` then probes for; a port-0 pick with it would skip
/// the kernel's wildcard check. Windows needs `SO_EXCLUSIVEADDRUSE`, and
/// `bind` probes for a wildcard holder that option does not refuse.
fn bind_exclusive(addr: SocketAddr, fixed: bool) -> io::Result<std::net::TcpListener> {
    use socket2::{Domain, Protocol, Socket, Type};
    let s = Socket::new(Domain::for_address(addr), Type::STREAM, Some(Protocol::TCP))?;
    #[cfg(any(target_os = "linux", target_os = "android"))]
    s.set_reuse_address(true)?;
    // Port 0's IPv6 bind, at the port the IPv4 bind picked, keeps the
    // kernel's check too, so a holder there sends it on to another port.
    #[cfg(target_os = "macos")]
    if fixed {
        s.set_reuse_address(true)?;
    }
    #[cfg(not(target_os = "macos"))]
    let _ = fixed;
    #[cfg(windows)]
    exclusive_addr_use(&s)?;
    if addr.is_ipv6() {
        s.set_only_v6(true)?;
    }
    s.bind(&addr.into())?;
    s.listen(1024)?;
    Ok(s.into())
}

#[cfg(windows)]
fn exclusive_addr_use(s: &socket2::Socket) -> io::Result<()> {
    use std::os::windows::io::AsRawSocket;
    use windows_sys::Win32::Networking::WinSock::{setsockopt, SOL_SOCKET, SO_EXCLUSIVEADDRUSE};
    let on: i32 = 1;
    // SAFETY: the socket is open, and the option is a 4-byte BOOL.
    let rc = unsafe {
        setsockopt(
            s.as_raw_socket() as _,
            SOL_SOCKET,
            SO_EXCLUSIVEADDRUSE,
            (&on as *const i32).cast(),
            std::mem::size_of::<i32>() as i32,
        )
    };
    if rc != 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn listen(l: std::net::TcpListener) -> io::Result<TcpListener> {
    l.set_nonblocking(true)?;
    TcpListener::from_std(l)
}

pub(crate) fn bind_with(
    port: u16,
    mut bind_one: impl FnMut(SocketAddr) -> io::Result<std::net::TcpListener>,
) -> Result<Bound, BindError> {
    let fail = |family, error| BindError {
        family,
        port,
        error,
    };
    let mut try_bind = |family: Family, p: u16| bind_one(family.addr(p)).and_then(listen);
    if port != 0 {
        let v4 = try_bind(Family::V4, port);
        let v6 = try_bind(Family::V6, port);
        return match (v4, v6) {
            (Err(e), _) if !family_missing(&e) => Err(fail(Family::V4, e)),
            (_, Err(e)) if !family_missing(&e) => Err(fail(Family::V6, e)),
            (Err(e), Err(_)) => Err(fail(Family::V4, e)),
            (Ok(a), Ok(b)) => Ok(Bound {
                listeners: vec![a, b],
                port,
                warnings: vec![],
            }),
            (Ok(a), Err(e)) => Ok(Bound {
                listeners: vec![a],
                port,
                warnings: vec![BindWarning {
                    family: Family::V6,
                    error: e,
                }],
            }),
            (Err(e), Ok(b)) => Ok(Bound {
                listeners: vec![b],
                port,
                warnings: vec![BindWarning {
                    family: Family::V4,
                    error: e,
                }],
            }),
        };
    }
    let mut last = None;
    for _ in 0..PORT_ZERO_TRIES {
        let v4 = match try_bind(Family::V4, 0) {
            Ok(l) => l,
            Err(e) if family_missing(&e) => {
                let v6 = try_bind(Family::V6, 0).map_err(|e| fail(Family::V6, e))?;
                let p = v6.local_addr().map_err(|e| fail(Family::V6, e))?.port();
                return Ok(Bound {
                    listeners: vec![v6],
                    port: p,
                    warnings: vec![BindWarning {
                        family: Family::V4,
                        error: e,
                    }],
                });
            }
            Err(e) => return Err(fail(Family::V4, e)),
        };
        let p = v4.local_addr().map_err(|e| fail(Family::V4, e))?.port();
        match try_bind(Family::V6, p) {
            Ok(v6) => {
                return Ok(Bound {
                    listeners: vec![v4, v6],
                    port: p,
                    warnings: vec![],
                })
            }
            Err(e) if e.kind() == io::ErrorKind::AddrInUse => last = Some(e),
            Err(e) if family_missing(&e) => {
                return Ok(Bound {
                    listeners: vec![v4],
                    port: p,
                    warnings: vec![BindWarning {
                        family: Family::V6,
                        error: e,
                    }],
                })
            }
            Err(e) => return Err(fail(Family::V6, e)),
        }
    }
    Err(fail(
        Family::V6,
        last.unwrap_or_else(|| io::Error::from(io::ErrorKind::AddrInUse)),
    ))
}

/// What happened to one forwarded connection, for the caller to print.
#[derive(Debug)]
pub enum Event<E> {
    /// Its call was refused; the connection was reset.
    Refused(E),
    /// Its call failed after `open`; the connection was reset.
    Lost(E),
    /// The sandbox closed it before sending a byte: nothing is listening on
    /// `127.0.0.1:<port>` in the sandbox, or the server closed it. A local
    /// client that closed first having sent nothing, such as a port probe,
    /// is not reported.
    NothingListening {
        /// The sandbox port.
        port: u16,
    },
    /// It fell behind and was reset.
    FellBehind,
    /// Accepting a connection failed; the listener carries on.
    AcceptFailed(io::Error),
    /// A call failed in a way no later call can get past: the listener is
    /// closed, and `serve` returns once the open connections end.
    Stopped(E),
}

impl<E: std::fmt::Display> std::fmt::Display for Event<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Refused(e) => write!(f, "a forwarded connection was refused: {e}"),
            Event::Lost(e) => write!(f, "a forwarded connection was reset: {e}"),
            Event::NothingListening { port } => write!(
                f,
                "nothing is listening on 127.0.0.1:{port} in the sandbox (a server bound only to ::1 or another address is not reached), or it closed the connection"
            ),
            Event::FellBehind => f.write_str("the forwarded connection fell behind and was reset"),
            Event::AcceptFailed(e) => write!(f, "could not accept a connection: {e}"),
            Event::Stopped(e) => write!(f, "stopped listening: {e}"),
        }
    }
}

/// Events queued for a caller that is not reading them; later ones are
/// dropped until it does, except [`Event::Stopped`], which waits for room.
pub const EVENT_BACKLOG: usize = 64;

/// The channel [`serve`] sends its events on.
pub fn events<E>() -> (mpsc::Sender<Event<E>>, mpsc::Receiver<Event<E>>) {
    mpsc::channel(EVENT_BACKLOG)
}

/// Accepts connections and forwards each to `remote` in the sandbox until a
/// call fails in a way no later call can recover from; then stops accepting,
/// sends that failure as [`Event::Stopped`], and returns once the open
/// connections end.
pub async fn serve<O: PortOpener>(
    bound: Bound,
    opener: O,
    remote: u16,
    limits: Limits,
    events: mpsc::Sender<Event<O::Error>>,
) {
    serve_with(
        bound,
        opener,
        remote,
        limits,
        Arc::new(Budget::new(limits)),
        events,
    )
    .await
}

async fn accept_any(ls: &[TcpListener]) -> io::Result<TcpStream> {
    match ls {
        [a] => a.accept().await.map(|(s, _)| s),
        [a, b] => tokio::select! {
            r = a.accept() => r.map(|(s, _)| s),
            r = b.accept() => r.map(|(s, _)| s),
        },
        _ => std::future::pending().await,
    }
}

async fn serve_with<O: PortOpener>(
    bound: Bound,
    opener: O,
    remote: u16,
    limits: Limits,
    budget: Arc<Budget>,
    events: mpsc::Sender<Event<O::Error>>,
) {
    let opener = Arc::new(opener);
    let permits = Arc::new(Semaphore::new(limits.max_connections.max(1)));
    let (fatal_tx, mut fatal_rx) = mpsc::channel::<O::Error>(1);
    let mut tasks = JoinSet::new();
    let listeners = bound.listeners;
    let fatal = loop {
        while tasks.try_join_next().is_some() {}
        let permit = tokio::select! {
            p = permits.clone().acquire_owned() => p.expect("never closed"),
            e = fatal_rx.recv() => break e.expect("a sender is held"),
        };
        let stream = tokio::select! {
            s = accept_any(&listeners) => s,
            e = fatal_rx.recv() => break e.expect("a sender is held"),
        };
        match stream {
            Ok(stream) => {
                let conn = Connection {
                    opener: opener.clone(),
                    remote,
                    budget: budget.clone(),
                    events: events.clone(),
                    fatal: fatal_tx.clone(),
                };
                tasks.spawn(async move {
                    conn.run(stream).await;
                    drop(permit);
                });
            }
            Err(e) => {
                let _ = events.try_send(Event::AcceptFailed(e));
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    };
    drop(listeners);
    let _ = events.send(Event::Stopped(fatal)).await;
    while tasks.join_next().await.is_some() {}
}

/// One accepted connection's shared state.
struct Connection<O: PortOpener> {
    opener: Arc<O>,
    remote: u16,
    budget: Arc<Budget>,
    events: mpsc::Sender<Event<O::Error>>,
    fatal: mpsc::Sender<O::Error>,
}

/// How a forwarded connection ends.
enum End {
    /// The call ended; the local connection closes once its bytes are written.
    Drain,
    /// The local connection ended or failed; the call is cancelled.
    Close,
    /// The local connection is reset and the call cancelled.
    Reset,
}

/// What the writer to the local socket is handed.
enum Out {
    Data(Bytes),
    /// The sandbox closed its side: half-close the local socket.
    Eof,
}

/// How the call's receiving side ended.
enum Down<E> {
    Ended,
    Failed(E),
    FellBehind,
}

impl<O: PortOpener> Connection<O> {
    fn failed(&self, e: O::Error, refused: bool) {
        if e.ends_listener() {
            let _ = self.fatal.try_send(e);
        } else if refused {
            let _ = self.events.try_send(Event::Refused(e));
        } else {
            let _ = self.events.try_send(Event::Lost(e));
        }
    }

    async fn run(self, mut stream: TcpStream) {
        let (mut sink, source) = match self.opener.open().await {
            Ok(call) => call,
            Err(e) => {
                let _ = stream.set_zero_linger();
                self.failed(e, true);
                return;
            }
        };
        let open = PortClientFrame {
            frame: Some(port_client_frame::Frame::Open(u32::from(self.remote))),
        };
        // A failed send means the call ended; the source says why.
        let _ = sink.send(open).await;
        let (id, reset) = self.budget.register();
        let end = self.pump(&mut stream, sink, source, id, &reset).await;
        self.budget.unregister(id);
        if matches!(end, End::Reset) {
            let _ = stream.set_zero_linger();
        }
    }

    async fn pump(
        &self,
        stream: &mut TcpStream,
        sink: O::Sink,
        mut source: O::Source,
        id: u64,
        reset: &tokio::sync::Notify,
    ) -> End {
        let (mut rd, mut wr) = stream.split();
        let (out_tx, mut out_rx) = mpsc::unbounded_channel::<Out>();
        // The local client closed before sending a byte, as a port probe or
        // a browser's preconnect does: the server closing in turn says
        // nothing about whether one is listening.
        let closed_quietly = AtomicBool::new(false);
        let closed_quietly = &closed_quietly;

        // Local to sandbox: data, then `eof` at the local half-close.
        let mut sink = sink;
        let up = async move {
            let mut buf = vec![0u8; CHUNK];
            let mut sent_any = false;
            loop {
                match rd.read(&mut buf).await {
                    Ok(0) => {
                        // Before the send: `down` may take the reply's
                        // `eof` while this one is still being sent.
                        if !sent_any {
                            closed_quietly.store(true, Ordering::SeqCst);
                        }
                        let eof = PortClientFrame {
                            frame: Some(port_client_frame::Frame::Eof(true)),
                        };
                        let _ = sink.send(eof).await;
                        // Holding the sink keeps the call open for the reply.
                        std::future::pending::<()>().await;
                    }
                    Ok(n) => {
                        sent_any = true;
                        let data = PortClientFrame {
                            frame: Some(port_client_frame::Frame::Data(buf[..n].to_vec())),
                        };
                        if sink.send(data).await.is_err() {
                            std::future::pending::<()>().await;
                        }
                    }
                    Err(_) => return,
                }
            }
        };
        tokio::pin!(up);

        let budget = &self.budget;
        let writer = async {
            while let Some(out) = out_rx.recv().await {
                match out {
                    Out::Data(b) => {
                        let n = b.len();
                        wr.write_all(&b).await?;
                        budget.release(id, n);
                    }
                    Out::Eof => wr.shutdown().await?,
                }
            }
            io::Result::Ok(())
        };
        tokio::pin!(writer);

        let remote = self.remote;
        let events = &self.events;
        // Declared after `up`, so on a cancel the source drops before the sink.
        let down = async move {
            let out_tx = out_tx;
            let mut got_any = false;
            loop {
                match source.recv().await {
                    Ok(Some(f)) => match f.frame {
                        Some(port_server_frame::Frame::Data(d)) => {
                            if d.is_empty() {
                                continue;
                            }
                            got_any = true;
                            if !budget.take(id, d.len()) {
                                return Down::FellBehind;
                            }
                            let _ = out_tx.send(Out::Data(Bytes::from(d)));
                            // Frames already received arrive with no wait, so
                            // let the writer run between them: a reader that
                            // keeps up is reset only when its socket stops
                            // taking bytes, not by a burst the call had queued.
                            tokio::task::yield_now().await;
                        }
                        Some(port_server_frame::Frame::Eof(_)) => {
                            if !got_any && !closed_quietly.load(Ordering::SeqCst) {
                                let _ = events.try_send(Event::NothingListening { port: remote });
                            }
                            let _ = out_tx.send(Out::Eof);
                        }
                        None => {}
                    },
                    Ok(None) => return Down::Ended,
                    Err(e) => return Down::Failed(e),
                }
            }
        };
        tokio::pin!(down);

        let end = tokio::select! {
            () = &mut up => End::Close,
            d = &mut down => match d {
                Down::Ended => End::Drain,
                Down::Failed(e) => {
                    self.failed(e, false);
                    End::Reset
                }
                Down::FellBehind => {
                    let _ = self.events.try_send(Event::FellBehind);
                    End::Reset
                }
            },
            // The local socket failed, or everything was written.
            _ = &mut writer => End::Close,
            () = reset.notified() => {
                let _ = self.events.try_send(Event::FellBehind);
                End::Reset
            }
        };
        if !matches!(end, End::Drain) {
            return end;
        }
        // The call ended: write what it sent, then close the connection.
        tokio::select! {
            _ = &mut writer => End::Close,
            () = reset.notified() => {
                let _ = self.events.try_send(Event::FellBehind);
                End::Reset
            }
        }
    }
}
