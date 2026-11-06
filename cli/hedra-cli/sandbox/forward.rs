//! `port-forward`: a local port forwarded to a port in a sandbox, one
//! `PortForward` call per accepted connection.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use hedra_cli_sandbox::forward::{
    self, Event, Family, ForwardError, Limits, PortOpener, PortSource,
};
use hedra_cli_sandbox::pb::hedra::sandbox::v1::{PortClientFrame, PortServerFrame};
use hedra_cli_sandbox::session::{Session, SessionError};
use hedra_cli_sandbox::stream::calls::{self, Open, OpenError, Waiting};
use hedra_cli_sandbox::stream::status::{reason, BrokerStatus};
use hedra_cli_sandbox::stream::transport::{TransportConfig, TransportError};

use super::api::{Api, CallError};
use super::error::{escape_controls, Class, Failure, SANDBOX_NOT_FOUND};
use super::names::{self, Target};
use super::stream::{self, Needs, OwnedMinter, Sig, Signals};

/// One connection's failure, and whether no later connection can succeed.
pub(crate) struct ConnectionFailure {
    failure: Failure,
    permanent: bool,
}

impl std::fmt::Display for ConnectionFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.failure.message)
    }
}

impl ForwardError for ConnectionFailure {
    fn ends_listener(&self) -> bool {
        self.permanent
    }
}

/// A refusal no later call can get past: the sandbox or its replica stopped,
/// or the key lost a scope or was revoked. Any other 404 refuses only its
/// connection.
fn permanent_mint(e: &CallError) -> bool {
    e.http().is_some_and(|h| {
        matches!(h.status, 401 | 403)
            || (h.status == 404 && h.message() == SANDBOX_NOT_FOUND)
            || h.is(400, "FAILED_PRECONDITION")
    }) || matches!(e, CallError::NoCredential(_))
}

/// Said after the broker's message when it ends a forwarded connection
/// `rate_limited`.
const RATE_LIMITED_HINT: &str = "`port-forward` cannot slow the sandbox's server; copy large \
                                 files with `hedra-cli sandbox cp`.";

pub(crate) struct Source {
    stream: hedra_cli_sandbox::tonic::Streaming<PortServerFrame>,
    name: String,
    /// `--debug`: print the broker's status when it ends an admitted call.
    debug: bool,
}

impl PortSource for Source {
    type Error = ConnectionFailure;

    async fn recv(&mut self) -> Result<Option<PortServerFrame>, ConnectionFailure> {
        self.stream.message().await.map_err(|status| {
            let s = BrokerStatus::from(&status);
            if self.debug {
                stream::note(escape_controls(&format!("< port_forward ended: {s}")));
            }
            let mut failure = stream::broker_failure(&s, &self.name, Needs::Write, true);
            // The broker gives a forwarded connection no output window, so
            // a server writing faster than the link ends it however promptly
            // hedra-cli reads, and reconnecting meets the same limit.
            if s.is(reason::RATE_LIMITED) {
                failure.message = format!("{}. {RATE_LIMITED_HINT}", failure.message);
            }
            ConnectionFailure {
                permanent: s.is(reason::PERMISSION_DENIED),
                failure,
            }
        })
    }
}

struct Opener {
    api: Arc<Api>,
    session: Session<OwnedMinter>,
    id: String,
    name: String,
    quota_said: AtomicBool,
}

impl PortOpener for Opener {
    type Error = ConnectionFailure;
    type Sink = calls::Sender<PortClientFrame>;
    type Source = Source;

    async fn open(&self) -> Result<(Self::Sink, Self::Source), ConnectionFailure> {
        // The proxy only when the broker does not bypass it.
        let on_wait = |w: &Waiting| {
            stream::note(escape_controls(&stream::waiting_line(
                w,
                &self.name,
                self.session.proxy().as_deref(),
            )));
        };
        let debug = |line: &str| stream::note(escape_controls(line));
        let open = Open {
            on_wait: &on_wait,
            on_debug: self.api.debug.then_some(&debug as _),
            ..Default::default()
        };
        match calls::open_port_forward(&self.session, &open).await {
            Ok(call) => Ok((
                call.send,
                Source {
                    stream: call.recv,
                    name: self.name.clone(),
                    debug: self.api.debug,
                },
            )),
            Err(e) => {
                let permanent = match &e {
                    OpenError::Session(SessionError::Mint(m)) => permanent_mint(m),
                    OpenError::Refused(s) => s.is(reason::PERMISSION_DENIED),
                    // A proxy's answer to `CONNECT` other than 2xx, 5xx, 408
                    // or 429, a broker certificate the roots do not trust, a
                    // peer without HTTP/2, or a configuration that cannot
                    // work: every later connection would be refused the same
                    // way.
                    OpenError::Session(SessionError::Transport(
                        TransportError::Refused(_) | TransportError::Config(_),
                    )) => true,
                    OpenError::Session(SessionError::Transport(TransportError::Connect(_))) => {
                        false
                    }
                };
                if let OpenError::Refused(s) = &e {
                    if s.is(reason::QUOTA_EXCEEDED) && !self.quota_said.swap(true, Ordering::SeqCst)
                    {
                        stream::say_line(
                            "The workspace is at its limit of open sandbox sessions; connections \
                             are refused until some close.",
                        )
                        .await;
                    }
                }
                let proxy = self.session.proxy();
                let failure = stream::open_failure(
                    &self.api,
                    &self.id,
                    &self.name,
                    Needs::Write,
                    proxy.as_deref(),
                    e,
                )
                .await;
                Err(ConnectionFailure { failure, permanent })
            }
        }
    }
}

/// `<local>:<remote>`, or one port for both, each in decimal digits.
pub(crate) fn parse_ports(spec: &str) -> Result<(u16, u16), Failure> {
    let bad = |p: &str| Failure::rejected(format!("`{}` is not a port.", escape_controls(p)));
    let (local, remote) = match spec.split_once(':') {
        Some((l, r)) => (l, r),
        None => (spec, spec),
    };
    let port = |p: &str| {
        if p.is_empty() || !p.bytes().all(|b| b.is_ascii_digit()) {
            return Err(bad(p));
        }
        p.parse::<u16>().map_err(|_| bad(p))
    };
    let (local, remote) = (port(local)?, port(remote)?);
    if remote == 0 {
        return Err(Failure::rejected(
            "Name the sandbox's port: `<local>:<remote>`, where only the local port may be 0.",
        ));
    }
    Ok((local, remote))
}

/// "127.0.0.1:P and [::1]:P", naming only the families listening.
fn listening(bound: &forward::Bound) -> String {
    bound
        .families()
        .iter()
        .map(|f| match f {
            Family::V4 => format!("127.0.0.1:{}", bound.port),
            Family::V6 => format!("[::1]:{}", bound.port),
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

/// Runs `port-forward` until Ctrl-C (exit 0) or a refusal no later
/// connection can get past (its class's code).
pub(crate) async fn run(
    api: Arc<Api>,
    transport: TransportConfig,
    target: &Target,
    ports: &str,
    json_errors: bool,
) -> i32 {
    stream::ignore_sigpipe();
    let fail = |f: Failure| async move {
        f.say(json_errors).await;
        f.class.code()
    };
    let (local, remote) = match parse_ports(ports) {
        Ok(p) => p,
        Err(f) => return fail(f).await,
    };
    // Bound before any request, so a taken port costs none.
    let bound = match forward::bind(local).await {
        Ok(b) => b,
        Err(e) => return fail(Failure::rejected(e.to_string())).await,
    };
    let name = target.as_str().to_string();
    let mut signals = Signals::new();
    let id = tokio::select! {
        r = names::resolve(&api, target) => match r {
            Ok(id) => id,
            Err(f) => return fail(f).await,
        },
        sig = signals.next() => return signal_code(sig),
    };
    for w in &bound.warnings {
        stream::say_line(w).await;
    }
    stream::say_line(format!(
        "Forwarding {} to port {remote} in `{name}`. Any local user can connect to it, and so \
         can web pages in your browser, and it skips the sandbox's endpoint token; forward only \
         servers that have their own auth.",
        listening(&bound)
    ))
    .await;
    let opener = Opener {
        api: api.clone(),
        session: Session::new(
            OwnedMinter {
                api: api.clone(),
                id: id.clone(),
            },
            transport.clone(),
        ),
        id,
        name,
        quota_said: AtomicBool::new(false),
    };
    let (events_tx, mut events) = forward::events::<ConnectionFailure>();
    // Prints each event as it comes, and a refusal that closed the listener
    // at once, so a Ctrl-C while open connections finish still shows why.
    let printer = tokio::spawn(async move {
        let mut stopped = None;
        while let Some(e) = events.recv().await {
            match e {
                Event::Stopped(cf) => {
                    let f = cf.failure;
                    f.say(json_errors).await;
                    stopped = Some(f.class.code());
                }
                e => stream::say_line(escape_controls(&e.to_string())).await,
            }
        }
        stopped
    });
    let sig = tokio::select! {
        () = forward::serve(bound, opener, remote, Limits::default(), events_tx) => None,
        sig = signals.next() => Some(sig),
    };
    let stopped = printer.await.ok().flatten();
    match (stopped, sig) {
        (Some(code), _) => code,
        (None, Some(sig)) => signal_code(sig),
        (None, None) => Class::Internal.code(),
    }
}

/// Ctrl-C is how a port-forward ends.
fn signal_code(sig: Sig) -> i32 {
    match sig {
        Sig::Int => 0,
        other => other.class().code(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_mint_404_for_the_sandbox_is_permanent() {
        use super::super::error::HttpError;
        let http = |status, code: &str, message: &str| {
            let body = format!(r#"{{"error":{{"code":"{code}","message":"{message}"}}}}"#);
            CallError::Http(HttpError::new(status, &body))
        };
        assert!(permanent_mint(&http(404, "NOT_FOUND", SANDBOX_NOT_FOUND)));
        assert!(!permanent_mint(&http(404, "NOT_FOUND", "Not Found")));
        assert!(!permanent_mint(&http(
            404,
            "NOT_FOUND",
            "Sandboxes are not available."
        )));
        assert!(!permanent_mint(&CallError::Http(HttpError::new(
            404,
            "<html>404</html>"
        ))));
        assert!(permanent_mint(&http(
            400,
            "FAILED_PRECONDITION",
            "The sandbox is not active."
        )));
        assert!(permanent_mint(&http(
            401,
            "UNAUTHENTICATED",
            "Invalid API key."
        )));
        assert!(permanent_mint(&http(403, "PERMISSION_DENIED", "No.")));
        assert!(!permanent_mint(&http(503, "UNAVAILABLE", "Try again.")));
    }

    #[test]
    fn ports_are_one_or_local_and_remote() {
        assert_eq!(parse_ports("8888").unwrap(), (8888, 8888));
        assert_eq!(parse_ports("0:8888").unwrap(), (0, 8888));
        assert_eq!(parse_ports("9000:80").unwrap(), (9000, 80));
        assert!(parse_ports("0").is_err());
        assert!(parse_ports("x:80").is_err());
        assert!(parse_ports("70000").is_err());
        for bad in ["+80", "80:+80", "", ":80", "8 0", "-1"] {
            assert!(parse_ports(bad).is_err(), "{bad}");
        }
    }
}
