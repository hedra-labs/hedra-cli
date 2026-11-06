//! Port-forward over a fake opener whose calls the test plays.

use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpSocket, TcpStream};
use tokio::sync::mpsc;
use tokio::time::timeout;

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct FakeErr {
    msg: &'static str,
    permanent: bool,
}

impl std::fmt::Display for FakeErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg)
    }
}

impl ForwardError for FakeErr {
    fn ends_listener(&self) -> bool {
        self.permanent
    }
}

struct FakeSource(mpsc::Receiver<Result<PortServerFrame, FakeErr>>);

impl PortSource for FakeSource {
    type Error = FakeErr;
    async fn recv(&mut self) -> Result<Option<PortServerFrame>, FakeErr> {
        match self.0.recv().await {
            Some(Ok(f)) => Ok(Some(f)),
            Some(Err(e)) => Err(e),
            None => Ok(None),
        }
    }
}

/// The sandbox's half of one forwarded connection's call.
struct Call {
    from_client: mpsc::Receiver<PortClientFrame>,
    to_client: mpsc::Sender<Result<PortServerFrame, FakeErr>>,
}

impl Call {
    async fn frame(&mut self) -> Option<port_client_frame::Frame> {
        self.from_client.recv().await.and_then(|f| f.frame)
    }

    async fn opened(&mut self) -> u32 {
        let first = timeout(Duration::from_secs(5), self.frame()).await;
        match first.expect("no first frame") {
            Some(port_client_frame::Frame::Open(p)) => p,
            other => panic!("the first frame is not open: {other:?}"),
        }
    }

    async fn data(&self, b: &[u8]) {
        let f = PortServerFrame {
            frame: Some(port_server_frame::Frame::Data(b.to_vec())),
        };
        let _ = self.to_client.send(Ok(f)).await;
    }

    async fn eof(&self) {
        let f = PortServerFrame {
            frame: Some(port_server_frame::Frame::Eof(true)),
        };
        let _ = self.to_client.send(Ok(f)).await;
    }

    /// Reads data frames to `eof`.
    async fn read_to_eof(&mut self) -> Vec<u8> {
        let mut got = Vec::new();
        loop {
            let next = timeout(Duration::from_secs(5), self.frame()).await;
            match next.expect("no eof") {
                Some(port_client_frame::Frame::Data(b)) => got.extend(b),
                Some(port_client_frame::Frame::Eof(_)) | None => return got,
                Some(other) => panic!("unexpected {other:?}"),
            }
        }
    }

    async fn cancelled(&self) {
        timeout(Duration::from_secs(5), self.to_client.closed())
            .await
            .expect("the call was not cancelled");
    }
}

struct Opener {
    calls: mpsc::UnboundedSender<Call>,
    refusals: Mutex<VecDeque<Option<FakeErr>>>,
}

impl PortOpener for Opener {
    type Error = FakeErr;
    type Sink = mpsc::Sender<PortClientFrame>;
    type Source = FakeSource;
    async fn open(&self) -> Result<(Self::Sink, FakeSource), FakeErr> {
        if let Some(Some(e)) = self.refusals.lock().unwrap().pop_front() {
            return Err(e);
        }
        let (ctx, crx) = mpsc::channel(64);
        let (stx, srx) = mpsc::channel(4);
        let _ = self.calls.send(Call {
            from_client: crx,
            to_client: stx,
        });
        Ok((ctx, FakeSource(srx)))
    }
}

const REMOTE: u16 = 8888;

struct Harness {
    port: u16,
    calls: mpsc::UnboundedReceiver<Call>,
    events: mpsc::Receiver<Event<FakeErr>>,
    serve: tokio::task::JoinHandle<()>,
    budget: Arc<Budget>,
    v6: bool,
}

async fn harness(limits: Limits, refusals: Vec<Option<FakeErr>>) -> Harness {
    let bound = bind(0).await.unwrap();
    let port = bound.port;
    let v6 = bound.warnings.is_empty();
    let (ctx, calls) = mpsc::unbounded_channel();
    let (etx, events) = events();
    let opener = Opener {
        calls: ctx,
        refusals: Mutex::new(refusals.into()),
    };
    let budget = Arc::new(Budget::new(limits));
    let serve = tokio::spawn(serve_with(
        bound,
        opener,
        REMOTE,
        limits,
        budget.clone(),
        etx,
    ));
    Harness {
        port,
        calls,
        events,
        serve,
        budget,
        v6,
    }
}

impl Harness {
    async fn connect(&self) -> TcpStream {
        TcpStream::connect(("127.0.0.1", self.port)).await.unwrap()
    }

    /// A client that holds little in its own receive buffer.
    async fn connect_small(&self) -> TcpStream {
        let s = TcpSocket::new_v4().unwrap();
        s.set_recv_buffer_size(4096).unwrap();
        s.connect(([127, 0, 0, 1], self.port).into()).await.unwrap()
    }

    /// Connects, and reports whether the connection was reset.
    async fn connect_reset(&self) -> bool {
        match TcpStream::connect(("127.0.0.1", self.port)).await {
            Ok(mut s) => was_reset(&mut s).await,
            Err(e) => e.kind() == io::ErrorKind::ConnectionReset,
        }
    }

    async fn call(&mut self) -> Call {
        let mut c = timeout(Duration::from_secs(5), self.calls.recv())
            .await
            .expect("no call was opened")
            .unwrap();
        assert_eq!(c.opened().await, u32::from(REMOTE));
        c
    }

    async fn event(&mut self) -> Event<FakeErr> {
        timeout(Duration::from_secs(5), self.events.recv())
            .await
            .expect("no event")
            .unwrap()
    }
}

/// Whether the connection was reset, not closed.
async fn was_reset(s: &mut TcpStream) -> bool {
    let mut got = Vec::new();
    match timeout(Duration::from_secs(5), s.read_to_end(&mut got)).await {
        Ok(Err(e)) => e.kind() == io::ErrorKind::ConnectionReset,
        Ok(Ok(_)) => false,
        Err(_) => panic!("the connection stayed open"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn a_server_that_speaks_first_is_heard() {
    let mut h = harness(Limits::default(), vec![]).await;
    let mut client = h.connect().await;
    let call = h.call().await;
    call.data(b"220 ready\r\n").await;
    let mut buf = [0u8; 11];
    timeout(Duration::from_secs(5), client.read_exact(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&buf, b"220 ready\r\n");
}

#[tokio::test(flavor = "multi_thread")]
async fn localhost_resolving_to_ipv6_is_forwarded() {
    let mut h = harness(Limits::default(), vec![]).await;
    if !h.v6 {
        return; // no IPv6 here
    }
    let mut client = TcpStream::connect(("::1", h.port)).await.unwrap();
    let mut call = h.call().await;
    client.write_all(b"ping").await.unwrap();
    client.shutdown().await.unwrap();
    assert_eq!(call.read_to_eof().await, b"ping");
}

#[tokio::test(flavor = "multi_thread")]
async fn two_connections_get_a_call_each() {
    let mut h = harness(Limits::default(), vec![]).await;
    let mut a = h.connect().await;
    let mut ca = h.call().await;
    let mut b = h.connect().await;
    let mut cb = h.call().await;
    a.write_all(b"from a").await.unwrap();
    b.write_all(b"from b").await.unwrap();
    a.shutdown().await.unwrap();
    b.shutdown().await.unwrap();
    assert_eq!(ca.read_to_eof().await, b"from a");
    assert_eq!(cb.read_to_eof().await, b"from b");
    cb.data(b"to b").await;
    ca.data(b"to a").await;
    drop((ca, cb));
    let (mut ra, mut rb) = (Vec::new(), Vec::new());
    timeout(Duration::from_secs(5), a.read_to_end(&mut ra))
        .await
        .expect("read timed out")
        .unwrap();
    timeout(Duration::from_secs(5), b.read_to_end(&mut rb))
        .await
        .expect("read timed out")
        .unwrap();
    assert_eq!((ra.as_slice(), rb.as_slice()), (&b"to a"[..], &b"to b"[..]));
}

#[tokio::test(flavor = "multi_thread")]
async fn a_local_half_close_still_gets_the_reply() {
    let mut h = harness(Limits::default(), vec![]).await;
    let mut client = h.connect().await;
    let mut call = h.call().await;
    client.write_all(b"GET /").await.unwrap();
    client.shutdown().await.unwrap();
    assert_eq!(call.read_to_eof().await, b"GET /");
    call.data(b"200 OK").await;
    call.eof().await;
    drop(call);
    let mut got = Vec::new();
    timeout(Duration::from_secs(5), client.read_to_end(&mut got))
        .await
        .expect("read timed out")
        .unwrap();
    assert_eq!(got, b"200 OK");
    // A reply arrived, so nothing is said about a missing server.
    assert!(h.events.try_recv().is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_server_eof_half_closes_the_local_socket() {
    let mut h = harness(Limits::default(), vec![]).await;
    let mut client = h.connect().await;
    let mut call = h.call().await;
    call.data(b"bye").await;
    call.eof().await;
    let mut got = Vec::new();
    timeout(Duration::from_secs(5), client.read_to_end(&mut got))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(got, b"bye");
    // The other direction stays open until the call ends.
    client.write_all(b"more").await.unwrap();
    match timeout(Duration::from_secs(5), call.frame())
        .await
        .expect("no data")
    {
        Some(port_client_frame::Frame::Data(b)) => assert_eq!(b, b"more"),
        other => panic!("{other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn an_eof_before_any_byte_says_nothing_is_listening() {
    let mut h = harness(Limits::default(), vec![]).await;
    let mut client = h.connect().await;
    let call = h.call().await;
    call.eof().await;
    drop(call);
    match h.event().await {
        Event::NothingListening { port } => assert_eq!(port, REMOTE),
        other => panic!("{other:?}"),
    }
    let msg = Event::<FakeErr>::NothingListening { port: REMOTE }.to_string();
    assert!(
        msg.starts_with("nothing is listening on 127.0.0.1:8888 in the sandbox"),
        "{msg}"
    );
    let mut got = Vec::new();
    timeout(Duration::from_secs(5), client.read_to_end(&mut got))
        .await
        .expect("read timed out")
        .unwrap();
    assert!(got.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn the_connection_past_the_cap_waits() {
    let limits = Limits {
        max_connections: 2,
        ..Limits::default()
    };
    let mut h = harness(limits, vec![]).await;
    let _a = h.connect().await;
    let ca = h.call().await;
    let _b = h.connect().await;
    let _cb = h.call().await;
    let mut c = h.connect().await;
    assert!(
        timeout(Duration::from_millis(300), h.calls.recv())
            .await
            .is_err(),
        "a third call opened past the cap"
    );
    drop(ca);
    let mut cc = h.call().await;
    c.write_all(b"third").await.unwrap();
    c.shutdown().await.unwrap();
    assert_eq!(cc.read_to_eof().await, b"third");
}

#[tokio::test(flavor = "multi_thread")]
async fn a_refusal_for_one_connection_resets_only_it() {
    let transient = FakeErr {
        msg: "quota_exceeded",
        permanent: false,
    };
    let mut h = harness(Limits::default(), vec![Some(transient.clone())]).await;
    assert!(h.connect_reset().await);
    match h.event().await {
        Event::Refused(e) => assert_eq!(e, transient),
        other => panic!("{other:?}"),
    }
    let mut b = h.connect().await;
    let mut cb = h.call().await;
    b.write_all(b"ok").await.unwrap();
    b.shutdown().await.unwrap();
    assert_eq!(cb.read_to_eof().await, b"ok");
    assert!(!h.serve.is_finished());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_call_lost_after_open_resets_its_connection() {
    let mut h = harness(Limits::default(), vec![]).await;
    let mut a = h.connect().await;
    let ca = h.call().await;
    ca.data(b"partial").await;
    let _ = ca
        .to_client
        .send(Err(FakeErr {
            msg: "rate_limited",
            permanent: false,
        }))
        .await;
    match h.event().await {
        Event::Lost(e) => assert_eq!(e.msg, "rate_limited"),
        other => panic!("{other:?}"),
    }
    assert!(was_reset(&mut a).await);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_permanent_refusal_closes_the_listener_once_open_connections_end() {
    let fatal = FakeErr {
        msg: "permission_denied",
        permanent: true,
    };
    let mut h = harness(Limits::default(), vec![None, Some(fatal.clone())]).await;
    let mut a = h.connect().await;
    let mut ca = h.call().await;
    assert!(h.connect_reset().await);
    // The listener is closed, and says why at once, but the open
    // connection carries on.
    match h.event().await {
        Event::Stopped(e) => assert_eq!(e, fatal),
        other => panic!("{other:?}"),
    }
    assert!(
        TcpStream::connect(("127.0.0.1", h.port)).await.is_err(),
        "still listening"
    );
    assert!(!h.serve.is_finished());
    a.write_all(b"still here").await.unwrap();
    a.shutdown().await.unwrap();
    assert_eq!(ca.read_to_eof().await, b"still here");
    drop(ca);
    timeout(Duration::from_secs(5), h.serve)
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn dropping_serve_cancels_every_call() {
    let mut h = harness(Limits::default(), vec![]).await;
    let _a = h.connect().await;
    let ca = h.call().await;
    let _b = h.connect().await;
    let cb = h.call().await;
    h.serve.abort();
    ca.cancelled().await;
    cb.cancelled().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_connection_whose_buffer_fills_is_reset() {
    let limits = Limits {
        connection_buffer: 1 << 20,
        ..Limits::default()
    };
    let mut h = harness(limits, vec![]).await;
    let mut stalled = h.connect_small().await;
    let cs = h.call().await;
    let mut live = h.connect().await;
    let mut cl = h.call().await;
    let chunk = vec![7u8; 32 << 10];
    let sender = tokio::spawn(async move {
        for _ in 0..256 {
            if cs.to_client.is_closed() {
                break;
            }
            cs.data(&chunk).await;
        }
        cs.cancelled().await;
    });
    match h.event().await {
        Event::FellBehind => {}
        other => panic!("{other:?}"),
    }
    timeout(Duration::from_secs(5), sender)
        .await
        .unwrap()
        .unwrap();
    assert!(was_reset(&mut stalled).await);
    // The other connection keeps working.
    cl.data(b"fine").await;
    let mut buf = [0u8; 4];
    timeout(Duration::from_secs(5), live.read_exact(&mut buf))
        .await
        .expect("read timed out")
        .unwrap();
    assert_eq!(&buf, b"fine");
    live.write_all(b"up").await.unwrap();
    live.shutdown().await.unwrap();
    assert_eq!(cl.read_to_eof().await, b"up");
}

/// Sends `n` bytes on `call` in 32 KiB frames, then `eof`, at about the
/// broker's 64 MiB/s cap of two frames a millisecond, so a test can tell a
/// reader that keeps up from a burst the fake sends faster than any socket
/// drains.
fn feed_paced(call: Call, n: usize) -> tokio::task::JoinHandle<bool> {
    tokio::spawn(async move {
        let chunk = vec![3u8; 32 << 10];
        let mut tick = tokio::time::interval(Duration::from_millis(1));
        for i in 0..n / chunk.len() {
            if i % 2 == 0 {
                tick.tick().await;
            }
            call.data(&chunk).await;
        }
        call.eof().await;
        !call.to_client.is_closed()
    })
}

/// Waits for every connection to leave the budget.
async fn budget_empties(h: &Harness) {
    timeout(Duration::from_secs(5), async {
        while h.budget.connections() != 0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("a closed connection stayed in the budget");
    assert_eq!(h.budget.total(), 0);
}

/// Bytes the local socket took are given back, so a connection whose reader
/// keeps up moves far more than its buffer.
#[tokio::test(flavor = "multi_thread")]
async fn a_connection_whose_reader_keeps_up_is_never_reset() {
    let mut h = harness(Limits::default(), vec![]).await;
    let mut client = h.connect().await;
    let call = h.call().await;
    let n = 32 << 20;
    let fed = feed_paced(call, n);
    let mut got = Vec::new();
    timeout(Duration::from_secs(30), client.read_to_end(&mut got))
        .await
        .expect("read timed out")
        .unwrap();
    assert_eq!(got.len(), n);
    assert!(fed.await.unwrap(), "the call was cancelled");
    assert!(h.events.try_recv().is_err(), "{:?}", h.events.try_recv());
    budget_empties(&h).await;
}

/// Connections one after another, together over the invocation's budget,
/// are never reset, and each leaves the budget when it closes.
#[tokio::test(flavor = "multi_thread")]
async fn sequential_connections_past_the_total_budget_are_never_reset() {
    let mut h = harness(Limits::default(), vec![]).await;
    let each = 6 << 20;
    for i in 0..12 {
        let mut client = h.connect().await;
        let call = h.call().await;
        let fed = feed_paced(call, each);
        let mut got = Vec::new();
        timeout(Duration::from_secs(30), client.read_to_end(&mut got))
            .await
            .expect("read timed out")
            .unwrap();
        assert_eq!(got.len(), each, "connection {i}");
        assert!(fed.await.unwrap(), "connection {i}'s call was cancelled");
        budget_empties(&h).await;
    }
    assert!(h.events.try_recv().is_err(), "{:?}", h.events.try_recv());
}

/// A local connection that fails (a client's reset) cancels its call.
#[tokio::test(flavor = "multi_thread")]
async fn a_local_read_error_cancels_the_call() {
    let mut h = harness(Limits::default(), vec![]).await;
    let client = h.connect().await;
    let call = h.call().await;
    client.set_zero_linger().unwrap();
    drop(client);
    call.cancelled().await;
    budget_empties(&h).await;
}

/// A caller not reading events holds at most the backlog: a client looping
/// refused connects cannot grow it, and the refusal that stops the listener
/// still arrives.
#[tokio::test(flavor = "multi_thread")]
async fn unread_events_are_bounded_and_the_stop_still_arrives() {
    let transient = FakeErr {
        msg: "quota_exceeded",
        permanent: false,
    };
    let fatal = FakeErr {
        msg: "permission_denied",
        permanent: true,
    };
    let n = 3 * EVENT_BACKLOG;
    let mut refusals = vec![Some(transient); n];
    refusals.push(Some(fatal.clone()));
    let mut h = harness(Limits::default(), refusals).await;
    for _ in 0..n {
        assert!(h.connect_reset().await);
    }
    let _ = h.connect_reset().await;
    timeout(Duration::from_secs(5), async {
        while h.events.len() < EVENT_BACKLOG {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("the backlog did not fill");
    assert_eq!(h.events.len(), EVENT_BACKLOG);
    let mut got = Vec::new();
    while let Ok(Some(e)) = timeout(Duration::from_secs(5), h.events.recv()).await {
        got.push(e);
    }
    assert!(got.len() <= EVENT_BACKLOG + 1, "{} events", got.len());
    match got.last() {
        Some(Event::Stopped(e)) => assert_eq!(*e, fatal),
        other => panic!("{other:?}"),
    }
}

/// With the invocation's budget full, the connection holding the most is
/// reset, not the one whose bytes overflowed it.
#[tokio::test(flavor = "multi_thread")]
async fn a_full_budget_resets_the_connection_holding_the_most() {
    let limits = Limits {
        max_connections: 90,
        connection_buffer: 16 << 20,
        total_buffer: 20 << 20,
    };
    let mut h = harness(limits, vec![]).await;
    let mut big = h.connect_small().await;
    let cbig = h.call().await;
    let mut small = h.connect_small().await;
    let csmall = h.call().await;
    let chunk = vec![1u8; 32 << 10];
    for _ in 0..(14 << 20) / chunk.len() {
        cbig.data(&chunk).await;
    }
    // Wait for the big connection's bytes to settle in the buffer.
    let mut last = 0;
    loop {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let held = h.budget.held(0).unwrap();
        if held == last {
            break;
        }
        last = held;
    }
    let small_bytes = 10 << 20;
    let feeder = {
        let chunk = chunk.clone();
        tokio::spawn(async move {
            for _ in 0..small_bytes / chunk.len() {
                csmall.data(&chunk).await;
            }
            csmall.eof().await;
            csmall
        })
    };
    match h.event().await {
        Event::FellBehind => {}
        other => panic!("{other:?}"),
    }
    cbig.cancelled().await;
    assert_eq!(
        h.budget.held(0),
        None,
        "the big connection was not the one reset"
    );
    assert!(h.budget.held(1).unwrap() < last);
    assert!(was_reset(&mut big).await);
    let csmall = feeder.await.unwrap();
    let mut got = Vec::new();
    timeout(Duration::from_secs(5), small.read_to_end(&mut got))
        .await
        .expect("read timed out")
        .unwrap();
    assert_eq!(got.len(), small_bytes);
    assert!(
        !csmall.to_client.is_closed(),
        "the small connection's call was cancelled"
    );
}

fn refuse_v6(kind: io::Error) -> impl FnMut(SocketAddr) -> io::Result<std::net::TcpListener> {
    let mut kind = Some(kind);
    move |addr| {
        if addr.is_ipv6() {
            if let Some(e) = kind.take() {
                return Err(e);
            }
        }
        std::net::TcpListener::bind(addr)
    }
}

#[tokio::test]
async fn an_ipv6_bind_failing_for_want_of_ipv6_is_a_warning() {
    let b = bind_with(
        0,
        refuse_v6(io::Error::from(io::ErrorKind::AddrNotAvailable)),
    )
    .unwrap();
    assert_eq!(b.listeners.len(), 1);
    assert_eq!(b.warnings.len(), 1);
    assert_eq!(b.warnings[0].family, Family::V6);
    assert_eq!(b.families(), vec![Family::V4]);
    let afnosupport = io::Error::from_raw_os_error(super::afnosupport());
    let b = bind_with(0, refuse_v6(afnosupport)).unwrap();
    assert_eq!((b.listeners.len(), b.warnings.len()), (1, 1));
    // Any other failure on IPv6 fails the port-forward.
    let err = bind_with(
        0,
        refuse_v6(io::Error::from(io::ErrorKind::PermissionDenied)),
    )
    .unwrap_err();
    assert_eq!(err.family, Family::V6);
}

#[tokio::test]
async fn a_port_taken_on_ipv4_fails_naming_the_family() {
    let held = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = held.local_addr().unwrap().port();
    let err = bind(port).await.unwrap_err();
    assert_eq!(err.family, Family::V4);
    assert_eq!(err.error.kind(), io::ErrorKind::AddrInUse);
    assert!(err.to_string().contains("IPv4"), "{err}");
}

/// Another process's wildcard listener on the port fails the bind, rather
/// than losing it the port's local traffic.
#[tokio::test]
async fn a_port_held_by_a_wildcard_listener_fails() {
    let held = std::net::TcpListener::bind("0.0.0.0:0").unwrap();
    let port = held.local_addr().unwrap().port();
    let err = bind(port).await.unwrap_err();
    assert_eq!(err.error.kind(), io::ErrorKind::AddrInUse, "{err}");
}

/// A listener on `[::]:0`, IPv6-only or dual-stack, when the host has IPv6.
fn wildcard_v6_listener(only_v6: bool) -> Option<socket2::Socket> {
    use socket2::{Domain, Protocol, Socket, Type};
    let s = Socket::new(Domain::IPV6, Type::STREAM, Some(Protocol::TCP)).ok()?;
    s.set_only_v6(only_v6).ok()?;
    s.bind(&SocketAddr::from((std::net::Ipv6Addr::UNSPECIFIED, 0)).into())
        .ok()?;
    s.listen(16).ok()?;
    Some(s)
}

/// `[::]` taking IPv4 too, and `[::]` with `IPV6_V6ONLY`, which on Windows
/// a `0.0.0.0` probe alone would miss.
#[tokio::test]
async fn a_port_held_by_an_ipv6_wildcard_listener_fails() {
    for only_v6 in [false, true] {
        let Some(held) = wildcard_v6_listener(only_v6) else {
            continue;
        };
        let port = held.local_addr().unwrap().as_socket().unwrap().port();
        let err = bind(port).await.unwrap_err();
        assert_eq!(
            err.error.kind(),
            io::ErrorKind::AddrInUse,
            "{only_v6}: {err}"
        );
    }
}

#[tokio::test]
async fn a_port_held_on_ipv6_only_fails_naming_it() {
    let Ok(held) = std::net::TcpListener::bind("[::1]:0") else {
        return;
    };
    let port = held.local_addr().unwrap().port();
    let err = bind(port).await.unwrap_err();
    assert_eq!(err.family, Family::V6);
    assert_eq!(err.error.kind(), io::ErrorKind::AddrInUse);
    assert!(err.to_string().contains("IPv6"), "{err}");
}

#[tokio::test]
async fn port_zero_retries_a_port_taken_on_ipv6() {
    let mut tries = Vec::new();
    let b = bind_with(0, |addr| {
        tries.push(addr);
        if addr.is_ipv6() && tries.iter().filter(|a| a.is_ipv6()).count() == 1 {
            return Err(io::Error::from(io::ErrorKind::AddrInUse));
        }
        std::net::TcpListener::bind(addr)
    })
    .unwrap();
    assert_eq!(tries.iter().filter(|a| a.is_ipv4()).count(), 2);
    assert_eq!(b.listeners.len(), 2);
    assert!(b.warnings.is_empty());
    for l in &b.listeners {
        assert_eq!(l.local_addr().unwrap().port(), b.port);
    }
}

/// On macOS a fixed port held only by a connection in `TIME_WAIT` refuses
/// the bind too, so the message says so.
#[cfg(target_os = "macos")]
#[tokio::test]
async fn a_taken_fixed_port_on_macos_mentions_a_closed_connection() {
    let held = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = held.local_addr().unwrap().port();
    let e = bind(port).await.unwrap_err();
    let msg = e.to_string();
    assert!(msg.contains("closed in the last minute"), "{msg}");
    assert!(msg.contains("another port"), "{msg}");
}
