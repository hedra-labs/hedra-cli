//! Opening calls on the session, up to admission.
//!
//! A call opens with its request stream empty and sends nothing, `start` and
//! `open` included, until the broker's `sandbox-admitted` header arrives, so a
//! retry before admission repeats a call that sent nothing. After admission a
//! call is never replayed.
//!
//! A call's request stream never ends cleanly: once every sender is dropped,
//! the stream is reset (`RST_STREAM(CANCEL)`), so the broker cancels the
//! command rather than reading an end of input. A command's input ends only
//! by its own frame (`stdin_eof`, `eof`). hyper sends that reset only once it
//! polls the request body again, which it does not while the stream has no
//! flow-control window, so a call whose reset has not gone out shortly after
//! its senders drop has its connection closed instead, when no other call,
//! `Logs` included, is open on it. Several such stuck calls on one channel
//! each see the others open, so their connection is not closed.

use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use tokio::sync::mpsc;
use tokio::time::Instant;
use tonic::Streaming;

use crate::backoff::Backoff;
use crate::pb::hedra::sandbox::v1::{
    ExecClientFrame, ExecServerFrame, LogChunk, LogsRequest, PortClientFrame, PortServerFrame,
};
use crate::session::{Session, SessionError, SessionMinter};
use crate::stream::status::{self, reason, BrokerStatus};
use crate::stream::transport::{Breaker, TransportError};

/// The header the broker sends once it has authorised a call.
pub const ADMITTED_HEADER: &str = "sandbox-admitted";
/// How long a call keeps retrying before admission.
pub const ADMISSION_RETRY: Duration = Duration::from_secs(60);
/// How long one attempt may wait for the broker's response headers.
pub const OPEN_ATTEMPT: Duration = Duration::from_secs(15);
const SEND_BUFFER: usize = 16;
/// Between refused opens: a supervisor reconnecting or a broker restarting
/// is usually back within seconds, so the 60 s budget tries a dozen times.
const ADMISSION_BACKOFF: Backoff = Backoff::new(Duration::from_millis(500), Duration::from_secs(5));
/// How long after its senders drop a call's reset may take to go out before
/// its connection is closed.
const RESET_WAIT: Duration = Duration::from_millis(250);

/// Why a call could not be opened.
#[derive(Debug)]
pub enum OpenError<E> {
    Session(SessionError<E>),
    /// The broker refused the call, after any retries.
    Refused(BrokerStatus),
}

/// What a retry before admission is waiting on, for the caller to print.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Waiting {
    /// The sandbox's supervisor has not connected to the broker.
    Supervisor,
    /// The broker could not be reached, or did not say why.
    Broker(String),
}

/// An admitted call that streams frames both ways. Dropping every clone of
/// `send` resets the call; the request stream never ends cleanly.
pub struct Duplex<In, Out> {
    pub send: Sender<In>,
    pub recv: Streaming<Out>,
}

/// A call's request sender; dropping the last clone resets the call.
pub struct Sender<T> {
    tx: mpsc::Sender<T>,
    _last: Arc<LastSender>,
}

impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            tx: self.tx.clone(),
            _last: self._last.clone(),
        }
    }
}

impl<T> std::ops::Deref for Sender<T> {
    type Target = mpsc::Sender<T>;

    fn deref(&self) -> &mpsc::Sender<T> {
        &self.tx
    }
}

/// Dropped with a call's last sender: closes the connection when the reset
/// has not gone out within `RESET_WAIT` and no other call is open on it.
struct LastSender {
    ended: Arc<AtomicBool>,
    breaker: Breaker,
}

impl Drop for LastSender {
    fn drop(&mut self) {
        let Ok(rt) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let ended = self.ended.clone();
        let breaker = self.breaker.clone();
        rt.spawn(async move {
            tokio::time::sleep(RESET_WAIT).await;
            // `ended` changes only under the count's lock.
            let calls = breaker.calls();
            if !ended.load(Ordering::SeqCst) && *calls <= 1 {
                breaker.close();
            }
        });
    }
}

/// Counts a call as open on its channel until dropped.
struct Counted(Breaker);

impl Counted {
    fn new(breaker: Breaker) -> Self {
        *breaker.calls() += 1;
        Self(breaker)
    }
}

impl Drop for Counted {
    fn drop(&mut self) {
        let mut calls = self.0.calls();
        *calls = calls.saturating_sub(1);
    }
}

/// An admitted `Logs` call's stream, counted as open on its channel until
/// dropped.
pub struct LogStream {
    stream: Streaming<LogChunk>,
    _open: Counted,
}

impl std::ops::Deref for LogStream {
    type Target = Streaming<LogChunk>;

    fn deref(&self) -> &Streaming<LogChunk> {
        &self.stream
    }
}

impl std::ops::DerefMut for LogStream {
    fn deref_mut(&mut self) -> &mut Streaming<LogChunk> {
        &mut self.stream
    }
}

impl<In, Out> Duplex<In, Out> {
    /// Resets the call, as dropping it does.
    pub fn cancel(self) {}
}

/// A request stream that, once its senders are gone, cancels its call
/// instead of ending, so the broker never reads a clean end of input.
struct Body<T> {
    rx: mpsc::Receiver<T>,
    cancel: Option<tonic::CancellationHandle>,
    /// Set once the cancel is handed to hyper, or hyper drops the body.
    ended: Arc<AtomicBool>,
    open: Counted,
}

impl<T> Body<T> {
    fn end(&self) {
        let _calls = self.open.0.calls();
        self.ended.store(true, Ordering::SeqCst);
    }
}

impl<T> Drop for Body<T> {
    fn drop(&mut self) {
        self.end();
    }
}

impl<T> tokio_stream::Stream for Body<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<T>> {
        match self.rx.poll_recv(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Ready(None) => {
                if let Some(cancel) = self.cancel.take() {
                    cancel.cancel();
                    self.end();
                }
                Poll::Pending
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A sender and a streaming request whose body it feeds, reset when the
/// sender's last clone drops.
fn body<T>(breaker: Breaker) -> (Sender<T>, tonic::Request<Body<T>>) {
    let (tx, rx) = mpsc::channel::<T>(SEND_BUFFER);
    let ended = Arc::new(AtomicBool::new(false));
    let mut request = tonic::Request::new(Body {
        rx,
        cancel: None,
        ended: ended.clone(),
        open: Counted::new(breaker.clone()),
    });
    let cancel = request.cancellation_handle();
    request.get_mut().cancel = Some(cancel);
    let send = Sender {
        tx,
        _last: Arc::new(LastSender { ended, breaker }),
    };
    (send, request)
}

pub type ExecCall = Duplex<ExecClientFrame, ExecServerFrame>;
pub type PortCall = Duplex<PortClientFrame, PortServerFrame>;

/// What a retry before admission after `status` is waiting on.
pub fn waiting_for(status: &BrokerStatus) -> Waiting {
    if status.is(reason::SANDBOX_DISCONNECTED) {
        Waiting::Supervisor
    } else {
        Waiting::Broker(status.message.clone())
    }
}

/// Options for opening a call.
pub struct Open<'a> {
    /// Retry before admission until this long has passed.
    pub retry_for: Duration,
    /// How long each attempt waits for the response headers.
    pub attempt: Duration,
    pub on_wait: &'a (dyn Fn(&Waiting) + Send + Sync),
    /// Takes `--debug`'s lines: each attempt's broker, its admission or
    /// refusal. The token is never in them.
    pub on_debug: Option<&'a (dyn Fn(&str) + Send + Sync)>,
}

impl Default for Open<'_> {
    fn default() -> Self {
        Self {
            retry_for: ADMISSION_RETRY,
            attempt: OPEN_ATTEMPT,
            on_wait: &|_| {},
            on_debug: None,
        }
    }
}

/// Response headers without `sandbox-admitted` are not an admission: such a
/// call is refused as unavailable, so it is retried having sent nothing.
fn admitted<T>(response: tonic::Response<T>) -> Result<T, tonic::Status> {
    if response.metadata().get(ADMITTED_HEADER).is_none() {
        return Err(tonic::Status::unavailable(
            "the broker answered without admitting the call",
        ));
    }
    Ok(response.into_inner())
}

macro_rules! open_duplex {
    ($name:ident, $method:ident, $in:ty, $out:ty) => {
        /// Opens the call and waits for admission; sends nothing.
        pub async fn $name<M: SessionMinter>(
            session: &Session<M>,
            open: &Open<'_>,
        ) -> Result<Duplex<$in, $out>, OpenError<M::Error>> {
            retrying(
                session,
                open,
                stringify!($method),
                |mut client, breaker| async move {
                    let (send, request) = body::<$in>(breaker);
                    let response = client.$method(request).await?;
                    Ok(Duplex {
                        send,
                        recv: admitted(response)?,
                    })
                },
            )
            .await
        }
    };
}

open_duplex!(open_exec, exec, ExecClientFrame, ExecServerFrame);
open_duplex!(
    open_port_forward,
    port_forward,
    PortClientFrame,
    PortServerFrame
);

/// Opens a `Logs` call. Its single request is read before authorisation, so
/// it carries the request itself.
pub async fn open_logs<M: SessionMinter>(
    session: &Session<M>,
    request: LogsRequest,
    open: &Open<'_>,
) -> Result<LogStream, OpenError<M::Error>> {
    retrying(session, open, "logs", |mut client, breaker| {
        // `LogsRequest` holds only scalars, so each attempt copies it.
        async move {
            let open = Counted::new(breaker);
            Ok(LogStream {
                stream: admitted(client.logs(request).await?)?,
                _open: open,
            })
        }
    })
    .await
}

/// The connector's error behind a status tonic built from a transport
/// failure, if any.
fn transport_cause(status: &tonic::Status) -> Option<&TransportError> {
    let mut source = std::error::Error::source(status);
    while let Some(e) = source {
        if let Some(t) = e.downcast_ref::<TransportError>() {
            return Some(t);
        }
        source = e.source();
    }
    None
}

async fn retrying<M, T, F, Fut>(
    session: &Session<M>,
    open: &Open<'_>,
    method: &str,
    mut call: F,
) -> Result<T, OpenError<M::Error>>
where
    M: SessionMinter,
    F: FnMut(crate::session::Client, Breaker) -> Fut,
    Fut: std::future::Future<Output = Result<T, tonic::Status>>,
{
    let deadline = Instant::now() + open.retry_for;
    let mut backoff = ADMISSION_BACKOFF;
    let mut reauthenticated = false;
    loop {
        let (client, breaker) = session
            .client_with_breaker()
            .await
            .map_err(OpenError::Session)?;
        let debug = |line: String| {
            if let Some(d) = open.on_debug {
                d(&line);
            }
        };
        if open.on_debug.is_some() {
            let url = session.broker_url().await.unwrap_or_default();
            debug(format!(
                "> {method} https://{url} (authorization: Bearer [REDACTED])"
            ));
        }
        let answered = tokio::time::timeout(open.attempt, call(client, breaker)).await;
        let status = match answered {
            Ok(Ok(admitted)) => {
                debug(format!("< {method} admitted ({ADMITTED_HEADER})"));
                return Ok(admitted);
            }
            Ok(Err(status)) => {
                // A connection that can never work fails at once, naming
                // why, before the status loses its source.
                if let Some(TransportError::Refused(m)) = transport_cause(&status) {
                    debug(format!("< {method} refused: {m}"));
                    return Err(OpenError::Session(SessionError::Transport(
                        TransportError::Refused(m.clone()),
                    )));
                }
                let mut s = BrokerStatus::from(&status);
                // tonic built this status from a local transport error, such
                // as a connection closed before the response headers; the
                // broker's own statuses never carry a source. Nothing was
                // sent, so it is retried as an unreachable broker.
                if s.reason.is_none() && std::error::Error::source(&status).is_some() {
                    s.code = tonic::Code::Unavailable;
                }
                s
            }
            Err(_) => BrokerStatus::from(&tonic::Status::deadline_exceeded(format!(
                "the broker did not answer within {}s",
                open.attempt.as_secs()
            ))),
        };
        debug(format!("< {method} refused: {status}"));
        // A signing-key rotation between mint and dial: mint once more.
        if status.code == tonic::Code::Unauthenticated || status.is(reason::UNAUTHENTICATED) {
            session.invalidate().await;
            if reauthenticated {
                return Err(OpenError::Refused(status));
            }
            reauthenticated = true;
            continue;
        }
        // No supervisor for the token's instance: the next call mints again.
        if status.is(reason::SANDBOX_DISCONNECTED) {
            session.invalidate().await;
        }
        if !status::retry_before_admission(&status) || Instant::now() + backoff.wait() >= deadline {
            return Err(OpenError::Refused(status));
        }
        (open.on_wait)(&waiting_for(&status));
        tokio::time::sleep(backoff.wait()).await;
        backoff.grow();
    }
}
