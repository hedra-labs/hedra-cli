//! The broker connection: TLS with rustls's ring provider, an optional HTTP
//! `CONNECT` tunnel, HTTP/2 with an adaptive flow-control window and
//! keepalive, on one tonic channel per invocation.

use std::future::Future;
use std::net::IpAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use super::{KEEPALIVE_INTERVAL, KEEPALIVE_TIMEOUT};

use hyper_util::rt::TokioIo;
use rustls::pki_types::ServerName;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tonic::transport::{Channel, Endpoint, Uri};

/// How to reach the broker, filled by the caller from its own HTTP settings.
#[derive(Clone, Debug, Default)]
pub struct TransportConfig {
    /// An `http://` proxy to tunnel through with `CONNECT`, with optional
    /// `user:password@` credentials.
    pub proxy: Option<String>,
    /// Hosts that bypass `proxy`: comma-separated names or `*`, matching the
    /// host itself or any subdomain.
    pub no_proxy: Option<String>,
    /// PEM bundles trusted beside the platform's roots.
    pub extra_roots_pem: Vec<Vec<u8>>,
    /// Accept any server certificate.
    pub insecure: bool,
    pub connect_timeout: Option<Duration>,
    pub user_agent: String,
}

/// Why the broker could not be reached, before any gRPC status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    /// The configuration cannot work: a proxy scheme the tunnel does not
    /// speak, an unparseable URL, an unusable root bundle.
    Config(String),
    /// The connection failed.
    Connect(String),
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::Config(m) | TransportError::Connect(m) => f.write_str(m),
        }
    }
}

impl std::error::Error for TransportError {}

const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// A lazily connected channel to `broker_url` (`host:port`, no scheme). The
/// first call connects; a dropped connection reconnects on the next call.
pub fn channel(broker_url: &str, config: &TransportConfig) -> Result<Channel, TransportError> {
    channel_with_breaker(broker_url, config).map(|(channel, _)| channel)
}

/// `channel`, with a breaker that can close its connection.
pub fn channel_with_breaker(
    broker_url: &str,
    config: &TransportConfig,
) -> Result<(Channel, Breaker), TransportError> {
    let (host, port) = split_host_port(broker_url)?;
    let proxy = match proxy_for(config, broker_url) {
        Some(url) => Some(Proxy::parse(url)?),
        None => None,
    };
    let tls = Arc::new(tls_config(config)?);
    let connect_timeout = config.connect_timeout.unwrap_or(DEFAULT_CONNECT_TIMEOUT);
    let endpoint = Endpoint::from_shared(endpoint_uri(&host, port))
        .map_err(|e| {
            TransportError::Config(format!("the broker address {broker_url} is invalid: {e}"))
        })?
        .user_agent(config.user_agent.clone())
        .map_err(|e| TransportError::Config(format!("invalid User-Agent: {e}")))?
        .connect_timeout(connect_timeout)
        .http2_adaptive_window(true)
        .http2_keep_alive_interval(KEEPALIVE_INTERVAL)
        .keep_alive_timeout(KEEPALIVE_TIMEOUT)
        .keep_alive_while_idle(true);
    let breaker = Breaker::default();
    let connector = Connector {
        host,
        port,
        proxy: proxy.map(Arc::new),
        tls: TlsConnector::from(tls),
        timeout: connect_timeout,
        breaker: breaker.clone(),
    };
    Ok((endpoint.connect_with_connector_lazy(connector), breaker))
}

/// `https://host:port`, an IPv6 host in brackets.
fn endpoint_uri(host: &str, port: u16) -> String {
    if host.contains(':') {
        format!("https://[{host}]:{port}")
    } else {
        format!("https://{host}:{port}")
    }
}

/// Closes a channel's connection from outside hyper. hyper stops polling a
/// request body while the stream has no flow-control window, so a call
/// cancelled then never sends its reset; closing the connection is how the
/// broker learns of it.
#[derive(Clone, Default)]
pub struct Breaker(Arc<BreakerState>);

#[derive(Default)]
struct BreakerState {
    current: std::sync::Mutex<Option<Arc<Kill>>>,
    calls: std::sync::Mutex<usize>,
}

impl Breaker {
    /// Closes the connection open now; the next call connects again.
    pub fn close(&self) {
        if let Some(kill) = self.0.current.lock().ok().and_then(|mut c| c.take()) {
            kill.killed.store(true, Ordering::SeqCst);
            kill.waker.wake();
        }
    }

    /// The number of calls open on the channel, as `calls` counts them.
    /// Holding it keeps that count and each call's end from changing.
    pub(crate) fn calls(&self) -> std::sync::MutexGuard<'_, usize> {
        self.0.calls.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn track(&self) -> Arc<Kill> {
        let kill = Arc::new(Kill::default());
        if let Ok(mut c) = self.0.current.lock() {
            *c = Some(kill.clone());
        }
        kill
    }
}

#[derive(Default)]
struct Kill {
    killed: AtomicBool,
    waker: futures_util::task::AtomicWaker,
}

/// A connection that fails every read and write once its breaker closes it.
struct Breakable<T> {
    inner: T,
    kill: Arc<Kill>,
}

impl<T> Breakable<T> {
    fn check(&self, cx: &Context<'_>) -> std::io::Result<()> {
        self.kill.waker.register(cx.waker());
        if self.kill.killed.load(Ordering::SeqCst) {
            return Err(std::io::ErrorKind::ConnectionAborted.into());
        }
        Ok(())
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for Breakable<T> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.check(cx)?;
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for Breakable<T> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.check(cx)?;
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> Poll<std::io::Result<usize>> {
        self.check(cx)?;
        Pin::new(&mut self.inner).poll_write_vectored(cx, bufs)
    }

    fn is_write_vectored(&self) -> bool {
        self.inner.is_write_vectored()
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        self.check(cx)?;
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// Refuses a proxy the tunnel cannot speak, before any request is sent.
pub fn check_proxy(config: &TransportConfig) -> Result<(), TransportError> {
    match &config.proxy {
        Some(url) => Proxy::parse(url).map(|_| ()),
        None => Ok(()),
    }
}

fn split_host_port(broker_url: &str) -> Result<(String, u16), TransportError> {
    let bad = || {
        TransportError::Config(format!(
            "the broker address `{broker_url}` is not host:port"
        ))
    };
    let (host, port) = broker_url.rsplit_once(':').ok_or_else(bad)?;
    let port: u16 = port.parse().map_err(|_| bad())?;
    let host = host.trim_start_matches('[').trim_end_matches(']');
    if host.is_empty() || host.contains('/') {
        return Err(bad());
    }
    Ok((host.to_string(), port))
}

/// The proxy a channel to `broker_url` goes through: `config.proxy`, unless
/// `config.no_proxy` names the broker's host.
pub fn proxy_for<'a>(config: &'a TransportConfig, broker_url: &str) -> Option<&'a str> {
    let proxy = config.proxy.as_deref()?;
    match split_host_port(broker_url) {
        Ok((host, _)) if bypasses(config.no_proxy.as_deref(), &host) => None,
        _ => Some(proxy),
    }
}

/// Whether `host` is in a `NO_PROXY`-style list. An IP host matches an
/// equal address or a CIDR block (`10.0.0.0/8`), as reqwest's does.
pub fn bypasses(no_proxy: Option<&str>, host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    let ip: Option<IpAddr> = host.parse().ok();
    no_proxy
        .into_iter()
        .flat_map(|l| l.split(','))
        .any(|entry| {
            let entry = entry.trim().to_ascii_lowercase();
            if let Some(ip) = ip {
                let bare = entry.trim_start_matches('[').trim_end_matches(']');
                if let Some(net) = cidr(bare) {
                    return net.contains(ip);
                }
                if let Ok(e) = bare.parse::<IpAddr>() {
                    return e == ip;
                }
            }
            let entry = entry.trim_start_matches('.');
            let entry = entry.split(':').next().unwrap_or_default();
            !entry.is_empty()
                && (entry == "*" || host == entry || host.ends_with(&format!(".{entry}")))
        })
}

/// An address block such as `10.0.0.0/8` or `fd00::/8`.
struct Cidr {
    net: IpAddr,
    prefix: u32,
}

fn cidr(entry: &str) -> Option<Cidr> {
    let (net, prefix) = entry.split_once('/')?;
    let net: IpAddr = net.parse().ok()?;
    let prefix: u32 = prefix.parse().ok()?;
    let max = if net.is_ipv4() { 32 } else { 128 };
    (prefix <= max).then_some(Cidr { net, prefix })
}

impl Cidr {
    fn contains(&self, ip: IpAddr) -> bool {
        let masked = |bits: u128, width: u32| match self.prefix {
            0 => 0,
            p => bits >> (width - p),
        };
        match (self.net, ip) {
            (IpAddr::V4(n), IpAddr::V4(a)) => {
                masked(u32::from(n).into(), 32) == masked(u32::from(a).into(), 32)
            }
            (IpAddr::V6(n), IpAddr::V6(a)) => {
                masked(u128::from(n), 128) == masked(u128::from(a), 128)
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Proxy {
    host: String,
    port: u16,
    authorization: Option<String>,
}

impl Proxy {
    /// Parses `http://[user:password@]host[:port][/]`. Any other scheme is
    /// refused: the tunnel speaks only plain-HTTP `CONNECT`.
    fn parse(raw: &str) -> Result<Self, TransportError> {
        parse_proxy(raw)
    }
}

fn parse_proxy(raw: &str) -> Result<Proxy, TransportError> {
    let lower = raw.to_ascii_lowercase();
    let rest = if let Some(rest) = lower.strip_prefix("http://").map(|_| &raw[7..]) {
        rest
    } else if lower.starts_with("https://") {
        return Err(TransportError::Config(
            "an https:// proxy is not supported for sandbox sessions; use an http:// proxy".into(),
        ));
    } else if lower.starts_with("socks") {
        return Err(TransportError::Config(
            "a SOCKS proxy is not supported for sandbox sessions; use an http:// proxy".into(),
        ));
    } else if !raw.contains("://") {
        raw
    } else {
        return Err(TransportError::Config(format!(
            "the proxy `{}` is not an http:// URL",
            redact(raw)
        )));
    };
    let rest = rest.trim_end_matches('/');
    let (auth, hostport) = match rest.rsplit_once('@') {
        Some((auth, hp)) => (Some(auth), hp),
        None => (None, rest),
    };
    let invalid_port =
        || TransportError::Config(format!("the proxy `{}` has an invalid port", redact(raw)));
    // An IPv6 host is bracketed, and its own colons are not the port's.
    let (host, port) = match hostport.strip_prefix('[').and_then(|r| r.split_once(']')) {
        Some((h, "")) => (h, 80),
        Some((h, p)) => (
            h,
            p.strip_prefix(':')
                .and_then(|p| p.parse::<u16>().ok())
                .ok_or_else(invalid_port)?,
        ),
        None => match hostport.rsplit_once(':') {
            Some((h, p)) => (h, p.parse::<u16>().map_err(|_| invalid_port())?),
            None => (hostport, 80),
        },
    };
    if host.is_empty() {
        return Err(TransportError::Config(format!(
            "the proxy `{}` names no host",
            redact(raw)
        )));
    }
    let authorization = auth.map(|a| format!("Basic {}", base64(&percent_decode(a))));
    Ok(Proxy {
        host: host.to_string(),
        port,
        authorization,
    })
}

/// The proxy URL with any credentials replaced, for messages.
pub fn redact(raw: &str) -> String {
    match (raw.find("://"), raw.rfind('@')) {
        (Some(scheme), Some(at)) if at > scheme => {
            format!("{}://***@{}", &raw[..scheme], &raw[at + 1..])
        }
        (None, Some(at)) => format!("***@{}", &raw[at + 1..]),
        _ => raw.to_string(),
    }
}

/// `%XX` escapes decoded to their bytes; anything else is kept as it is.
fn percent_decode(s: &str) -> Vec<u8> {
    fn hex(b: u8) -> Option<u8> {
        (b as char).to_digit(16).map(|d| d as u8)
    }
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                out.push(hi << 4 | lo);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    out
}

fn base64(input: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (b[0] as u32) << 16 | (b[1] as u32) << 8 | b[2] as u32;
        for i in 0..4 {
            if i <= chunk.len() {
                out.push(T[(n >> (18 - 6 * i) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn tls_config(config: &TransportConfig) -> Result<rustls::ClientConfig, TransportError> {
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let builder = rustls::ClientConfig::builder_with_provider(provider.clone())
        .with_safe_default_protocol_versions()
        .map_err(|e| TransportError::Config(format!("TLS setup failed: {e}")))?;
    let mut tls = if config.insecure {
        builder
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(insecure::AcceptAny(provider)))
            .with_no_client_auth()
    } else {
        let mut roots = rustls::RootCertStore::empty();
        let native = rustls_native_certs::load_native_certs();
        for cert in native.certs {
            let _ = roots.add(cert);
        }
        for pem in &config.extra_roots_pem {
            for cert in rustls_pemfile::certs(&mut &pem[..]) {
                let cert = cert.map_err(|e| {
                    TransportError::Config(format!("a CA bundle could not be read: {e}"))
                })?;
                roots.add(cert).map_err(|e| {
                    TransportError::Config(format!(
                        "a CA bundle holds an unusable certificate: {e}"
                    ))
                })?;
            }
        }
        if roots.is_empty() {
            return Err(TransportError::Config(
                "no trusted root certificates were found".into(),
            ));
        }
        builder.with_root_certificates(roots).with_no_client_auth()
    };
    tls.alpn_protocols = vec![b"h2".to_vec()];
    Ok(tls)
}

mod insecure {
    use std::sync::Arc;

    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::crypto::CryptoProvider;
    use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
    use rustls::{DigitallySignedStruct, SignatureScheme};

    /// Accepts any certificate, still checking handshake signatures.
    #[derive(Debug)]
    pub(super) struct AcceptAny(pub(super) Arc<CryptoProvider>);

    impl ServerCertVerifier for AcceptAny {
        fn verify_server_cert(
            &self,
            _: &CertificateDer<'_>,
            _: &[CertificateDer<'_>],
            _: &ServerName<'_>,
            _: &[u8],
            _: UnixTime,
        ) -> Result<ServerCertVerified, rustls::Error> {
            Ok(ServerCertVerified::assertion())
        }

        fn verify_tls12_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            rustls::crypto::verify_tls12_signature(
                message,
                cert,
                dss,
                &self.0.signature_verification_algorithms,
            )
        }

        fn verify_tls13_signature(
            &self,
            message: &[u8],
            cert: &CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            rustls::crypto::verify_tls13_signature(
                message,
                cert,
                dss,
                &self.0.signature_verification_algorithms,
            )
        }

        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            self.0.signature_verification_algorithms.supported_schemes()
        }
    }
}

#[derive(Clone)]
struct Connector {
    host: String,
    port: u16,
    proxy: Option<Arc<Proxy>>,
    tls: TlsConnector,
    timeout: Duration,
    breaker: Breaker,
}

type Io = TokioIo<Breakable<tokio_rustls::client::TlsStream<TcpStream>>>;

impl tower::Service<Uri> for Connector {
    type Response = Io;
    type Error = TransportError;
    type Future = Pin<Box<dyn Future<Output = Result<Io, TransportError>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: Uri) -> Self::Future {
        let this = self.clone();
        Box::pin(async move {
            tokio::time::timeout(this.timeout, this.connect())
                .await
                .map_err(|_| TransportError::Connect("connecting to the broker timed out".into()))?
        })
    }
}

impl Connector {
    async fn connect(&self) -> Result<Io, TransportError> {
        let tcp = match &self.proxy {
            Some(proxy) => tunnel(proxy, &self.host, self.port).await?,
            None => TcpStream::connect((self.host.as_str(), self.port))
                .await
                .map_err(|e| TransportError::Connect(format!("could not reach the broker: {e}")))?,
        };
        let _ = tcp.set_nodelay(true);
        let name = ServerName::try_from(self.host.clone())
            .map_err(|e| TransportError::Config(format!("invalid broker host name: {e}")))?;
        let tls = self
            .tls
            .connect(name, tcp)
            .await
            .map_err(|e| TransportError::Connect(format!("TLS with the broker failed: {e}")))?;
        Ok(TokioIo::new(Breakable {
            inner: tls,
            kill: self.breaker.track(),
        }))
    }
}

/// Opens a `CONNECT` tunnel through `proxy` to `host:port`.
async fn tunnel(proxy: &Proxy, host: &str, port: u16) -> Result<TcpStream, TransportError> {
    let mut tcp = TcpStream::connect((proxy.host.as_str(), proxy.port))
        .await
        .map_err(|e| {
            TransportError::Connect(format!(
                "could not reach the proxy {}:{}: {e}",
                proxy.host, proxy.port
            ))
        })?;
    let authority = if host.contains(':') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    };
    let mut request = format!("CONNECT {authority} HTTP/1.1\r\nHost: {authority}\r\n");
    if let Some(auth) = &proxy.authorization {
        request.push_str(&format!("Proxy-Authorization: {auth}\r\n"));
    }
    request.push_str("\r\n");
    tcp.write_all(request.as_bytes())
        .await
        .map_err(|e| TransportError::Connect(format!("the proxy closed the connection: {e}")))?;
    // Read the answer's head byte by byte, so no byte of the tunnel is
    // consumed past it.
    let mut head = Vec::with_capacity(256);
    while !head.ends_with(b"\r\n\r\n") {
        if head.len() > 16 * 1024 {
            return Err(TransportError::Connect(
                "the proxy's answer is too long".into(),
            ));
        }
        let b = tcp.read_u8().await.map_err(|e| {
            TransportError::Connect(format!("the proxy closed the connection: {e}"))
        })?;
        head.push(b);
    }
    let status_line = String::from_utf8_lossy(&head);
    let status_line = status_line.lines().next().unwrap_or_default();
    let code = status_line.split_whitespace().nth(1).unwrap_or_default();
    if !code.starts_with('2') {
        return Err(TransportError::Connect(format!(
            "the proxy refused the tunnel to the broker: {}",
            status_line.trim()
        )));
    }
    Ok(tcp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_escapes_decode_and_anything_else_is_kept() {
        assert_eq!(percent_decode("p%40ss"), b"p@ss");
        assert_eq!(percent_decode("%e2%82%ac"), "€".as_bytes());
        // A `%` before a multi-byte character, or a sign, is not an escape.
        assert_eq!(percent_decode("%€"), "%€".as_bytes());
        assert_eq!(percent_decode("a%€b"), "a%€b".as_bytes());
        assert_eq!(percent_decode("%+1"), b"%+1");
        assert_eq!(percent_decode("%4"), b"%4");
        let p = Proxy::parse("http://u:%€@p:3128").unwrap();
        assert_eq!(
            p.authorization.as_deref(),
            Some(format!("Basic {}", base64("u:%€".as_bytes())).as_str())
        );
    }

    #[test]
    fn proxies_parse_and_refuse_what_the_tunnel_cannot_speak() {
        let p = Proxy::parse("http://user:p%40ss@proxy.local:3128/").unwrap();
        assert_eq!(p.host, "proxy.local");
        assert_eq!(p.port, 3128);
        assert_eq!(p.authorization.as_deref(), Some("Basic dXNlcjpwQHNz"));
        assert_eq!(Proxy::parse("proxy.local").unwrap().port, 80);
        let v6 = Proxy::parse("http://[::1]").unwrap();
        assert_eq!((v6.host.as_str(), v6.port), ("::1", 80));
        let v6 = Proxy::parse("http://[::1]:3128/").unwrap();
        assert_eq!((v6.host.as_str(), v6.port), ("::1", 3128));
        assert!(Proxy::parse("http://[::1]x").is_err());
        for bad in [
            "https://proxy:443",
            "socks5://proxy:1080",
            "socks5h://p:1",
            "ftp://p",
        ] {
            assert!(Proxy::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_refused_proxy_message_hides_credentials() {
        let e = Proxy::parse("socks5://user:secret@p:1080").unwrap_err();
        assert!(!e.to_string().contains("secret"));
        let e = Proxy::parse("ftp://user:secret@p").unwrap_err();
        assert!(!e.to_string().contains("secret"), "{e}");
    }

    #[test]
    fn a_broker_no_proxy_names_is_reached_directly() {
        let config = TransportConfig {
            proxy: Some("http://proxy.local:3128".into()),
            no_proxy: Some("127.0.0.1, .internal".into()),
            ..Default::default()
        };
        assert_eq!(proxy_for(&config, "127.0.0.1:443"), None);
        assert_eq!(proxy_for(&config, "broker.internal:443"), None);
        assert_eq!(
            proxy_for(&config, "broker.example:443"),
            Some("http://proxy.local:3128")
        );
    }

    #[test]
    fn no_proxy_matches_hosts_and_subdomains() {
        let list = Some("localhost, .internal.example ,api.hedra.com:443");
        assert!(bypasses(list, "localhost"));
        assert!(bypasses(list, "a.internal.example"));
        assert!(bypasses(list, "api.hedra.com"));
        assert!(!bypasses(list, "broker.sandbox.hedra.com"));
        assert!(bypasses(Some("*"), "anything"));
        assert!(!bypasses(None, "anything"));
    }

    #[test]
    fn no_proxy_matches_ip_hosts_by_address_and_block() {
        let list = Some("10.0.0.0/8, 192.168.1.7, fd00::/8, [::1]");
        assert!(bypasses(list, "10.2.3.4"));
        assert!(!bypasses(list, "11.0.0.1"));
        assert!(bypasses(list, "192.168.1.7"));
        assert!(!bypasses(list, "192.168.1.8"));
        assert!(bypasses(list, "fd12::1"));
        assert!(!bypasses(list, "fe80::1"));
        assert!(bypasses(list, "::1"));
        assert!(bypasses(Some("0.0.0.0/0"), "8.8.8.8"));
        assert!(bypasses(Some("127.0.0.1/32"), "127.0.0.1"));
        assert!(!bypasses(Some("127.0.0.1/32"), "127.0.0.2"));
    }

    #[test]
    fn broker_addresses_are_host_and_port() {
        assert_eq!(
            split_host_port("broker.sandbox.hedra.com:443").unwrap(),
            ("broker.sandbox.hedra.com".to_string(), 443)
        );
        assert_eq!(
            split_host_port("[::1]:9443").unwrap(),
            ("::1".to_string(), 9443)
        );
        assert!(split_host_port("https://x:443").is_err());
        assert!(split_host_port("noport").is_err());
    }

    #[tokio::test]
    async fn an_ipv6_broker_is_bracketed_in_the_endpoint() {
        assert_eq!(endpoint_uri("::1", 9443), "https://[::1]:9443");
        assert_eq!(
            endpoint_uri("broker.example", 443),
            "https://broker.example:443"
        );
        let config = TransportConfig {
            insecure: true,
            user_agent: "test".into(),
            ..Default::default()
        };
        assert!(channel("[::1]:9443", &config).is_ok());
    }

    #[test]
    fn base64_matches_the_standard_alphabet() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"user:p@ss"), "dXNlcjpwQHNz");
    }

    #[tokio::test]
    async fn a_tunnel_sends_connect_and_reads_only_the_answers_head() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 512];
            let n = s.read(&mut buf).await.unwrap();
            let req = String::from_utf8_lossy(&buf[..n]).into_owned();
            s.write_all(b"HTTP/1.1 200 Connection established\r\n\r\nTLS")
                .await
                .unwrap();
            req
        });
        let proxy = Proxy {
            host: "127.0.0.1".into(),
            port,
            authorization: Some("Basic eDp5".into()),
        };
        let mut tcp = tunnel(&proxy, "broker.example", 443).await.unwrap();
        let mut rest = [0u8; 3];
        tcp.read_exact(&mut rest).await.unwrap();
        assert_eq!(&rest, b"TLS");
        let req = server.await.unwrap();
        assert!(
            req.starts_with("CONNECT broker.example:443 HTTP/1.1\r\n"),
            "{req}"
        );
        assert!(req.contains("Proxy-Authorization: Basic eDp5\r\n"), "{req}");
    }

    #[tokio::test]
    async fn a_refused_tunnel_names_the_proxys_answer() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            let (mut s, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 512];
            let _ = s.read(&mut buf).await;
            let _ = s.write_all(b"HTTP/1.1 403 Forbidden\r\n\r\n").await;
        });
        let proxy = Proxy {
            host: "127.0.0.1".into(),
            port,
            authorization: None,
        };
        let e = tunnel(&proxy, "broker.example", 443).await.unwrap_err();
        assert!(e.to_string().contains("403 Forbidden"), "{e}");
    }
}
