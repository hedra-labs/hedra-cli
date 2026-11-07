//! `exec` and `shell`: a command in the sandbox with its stdio relayed.

use std::collections::HashMap;

use hedra_cli_sandbox::pb::hedra::sandbox::v1::{
    exec_client_frame, exec_server_frame, ExecClientFrame, ExecResize, ExecServerFrame, ExecStart,
    Stream,
};
use hedra_cli_sandbox::session::Session;
use hedra_cli_sandbox::stream::calls::{self, Duplex, Open, Waiting};
use hedra_cli_sandbox::stream::status::BrokerStatus;
use hedra_cli_sandbox::stream::transport::TransportConfig;
use tokio::sync::mpsc;

use super::api::Api;
use super::error::{escape_controls, Class, Failure};
use super::names::{self, Target};
use super::stream::{self, Minter, Needs, Out, RawMode, Sig, Signals, WriteEnd};

/// A larger `start` is refused before it is sent: the broker's frame limit.
const MAX_START_BYTES: usize = 1 << 20;
/// What the broker adds when it forwards argv and env in a frame of its own,
/// under the same limit: a session id (a UUID), the terminal's size and an
/// output window, about 50 bytes encoded.
const BROKER_START_OVERHEAD: usize = 64;

/// The argv `shell` runs: the image's shell, interactive and not a login
/// shell, so the image's `PATH` survives.
pub(crate) fn shell_argv() -> Vec<String> {
    vec![
        "sh".into(),
        "-c".into(),
        "exec \"${SHELL:-$(command -v bash || echo sh)}\"".into(),
    ]
}

pub(crate) struct Exec {
    pub(crate) argv: Vec<String>,
    pub(crate) tty: bool,
    /// `shell` rather than `exec -t`: names the alternative a terminal
    /// refusal offers.
    pub(crate) shell: bool,
    pub(crate) no_stdin: bool,
    pub(crate) env: HashMap<String, String>,
    pub(crate) term: Option<String>,
}

/// How an `exec` ended.
pub(crate) enum Ended {
    /// The command's own exit code.
    Exit(i32),
    /// hedra-cli's own failure; exits 255 after its line.
    Failed(Failure),
    /// A signal or a gone terminal; exits with the signal's code.
    Signal(Sig),
    /// stdout's reader went away.
    Pipe,
}

/// An `exec`'s end, with what `shell --gpu` needs to decide whether to stop
/// its sandbox.
pub(crate) struct Driven {
    pub(crate) ended: Ended,
    /// `start` was sent.
    pub(crate) started: bool,
    /// A frame came back.
    pub(crate) answered: bool,
    /// The status the stream ended with, when it ended with one.
    pub(crate) status: Option<BrokerStatus>,
}

impl Driven {
    fn before_start(ended: Ended) -> Self {
        Self {
            ended,
            started: false,
            answered: false,
            status: None,
        }
    }
}

/// Runs `exec` or `shell` and returns the process exit code. Errors go to
/// stderr, since stdout is the command's.
pub(crate) async fn run(
    api: &Api,
    transport: TransportConfig,
    target: &Target,
    exec: Exec,
    json_errors: bool,
) -> i32 {
    stream::ignore_sigpipe();
    let ended = run_inner(api, transport, target, exec).await;
    finish(ended, json_errors).await
}

/// The exit code for `ended`, after writing hedra-cli's own failure line.
pub(crate) async fn finish(ended: Ended, json_errors: bool) -> i32 {
    match ended {
        Ended::Exit(code) => code,
        Ended::Pipe => stream::die_by_sigpipe(),
        Ended::Signal(sig) => sig.class().code(),
        Ended::Failed(f) => {
            f.say(json_errors).await;
            255
        }
    }
}

async fn run_inner(api: &Api, transport: TransportConfig, target: &Target, exec: Exec) -> Ended {
    if exec.tty {
        let without = if exec.shell {
            stream::NoTerminal::Shell
        } else {
            stream::NoTerminal::ExecT
        };
        if let Err(f) = stream::foreground_terminal(without) {
            return Ended::Failed(f);
        }
    }
    let name = target.as_str().to_string();
    let mut signals = Signals::new();
    let id = tokio::select! {
        r = names::resolve(api, target) => match r {
            Ok(id) => id,
            Err(f) => return Ended::Failed(f),
        },
        sig = signals.next() => return Ended::Signal(sig),
    };
    let session = Session::new(
        Minter {
            api,
            id: id.clone(),
        },
        transport.clone(),
    );
    let driven = drive(api, &session, &id, &name, &exec, &mut signals).await;
    match driven.ended {
        Ended::Failed(f) if f.class == Class::ConnectionLost => {
            match lost_unless_signalled(api, &id, driven.status.as_ref(), &mut signals).await {
                Ok(f) => Ended::Failed(f),
                Err(sig) => Ended::Signal(sig),
            }
        }
        ended => ended,
    }
}

/// Opens an `Exec` call on `session` for `exec`, `shell` and `cp`: the
/// retry lines and, under `--debug`, the broker's lines on stderr; a
/// refusal read and classed; and a signal answered throughout, the read
/// that explains a refusal included.
pub(crate) async fn open_call(
    api: &Api,
    session: &Session<Minter<'_>>,
    id: &str,
    name: &str,
    signals: &mut Signals,
) -> Result<Duplex<ExecClientFrame, ExecServerFrame>, Ended> {
    // The proxy only when the broker does not bypass it.
    let proxy = || session.proxy();
    let on_wait = |w: &Waiting| {
        stream::note(escape_controls(&stream::waiting_line(
            w,
            name,
            proxy().as_deref(),
        )));
    };
    let debug = |line: &str| stream::note(escape_controls(line));
    let open = Open {
        on_wait: &on_wait,
        on_debug: api.debug.then_some(&debug as _),
        ..Default::default()
    };
    let opened = tokio::select! {
        r = calls::open_exec(session, &open) => r,
        sig = signals.next() => return Err(Ended::Signal(sig)),
    };
    let e = match opened {
        Ok(call) => return Ok(call),
        Err(e) => e,
    };
    let proxy = proxy();
    Err(tokio::select! {
        f = stream::open_failure(api, id, name, Needs::Exec, proxy.as_deref(), e) => {
            Ended::Failed(f)
        }
        sig = signals.next() => Ended::Signal(sig),
    })
}

/// The Connection lost failure, after a fresh read of the sandbox, or the
/// signal that came first: the read never holds a verb past Ctrl-C.
pub(crate) async fn lost_unless_signalled(
    api: &Api,
    id: &str,
    status: Option<&BrokerStatus>,
    signals: &mut Signals,
) -> Result<Failure, Sig> {
    tokio::select! {
        f = lost(api, id, status) => Ok(f),
        sig = signals.next() => Err(sig),
    }
}

/// Opens an `Exec` call on `session` and relays it until it ends.
pub(crate) async fn drive(
    api: &Api,
    session: &Session<Minter<'_>>,
    id: &str,
    name: &str,
    exec: &Exec,
    signals: &mut Signals,
) -> Driven {
    let call = match open_call(api, session, id, name, signals).await {
        Ok(call) => call,
        Err(ended) => return Driven::before_start(ended),
    };
    let debug = |line: &str| stream::note(escape_controls(line));
    let (mut out, mut err_out) = match (Out::dup(1), Out::dup(2)) {
        (Ok(o), Ok(e)) => (o, e),
        (Err(e), _) | (_, Err(e)) => return Driven::before_start(Ended::Failed(output_failed(e))),
    };

    // Listening for resizes before the size is read, so an early one is not lost.
    #[cfg(unix)]
    let winch = if exec.tty {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::window_change()).ok()
    } else {
        None
    };
    #[cfg(not(unix))]
    let winch = ();
    // Raw mode after admission, restored on every return by the guard.
    let (raw, size) = if exec.tty {
        match RawMode::enter() {
            Ok(guard) => (
                Some(guard),
                hedra_cli_sandbox::crossterm::terminal::size().unwrap_or((80, 24)),
            ),
            Err(f) => return Driven::before_start(Ended::Failed(f)),
        }
    } else {
        (None, (0, 0))
    };
    let mut env = exec.env.clone();
    if exec.tty {
        // `--term` wins, then the user's own `--env TERM=`, then the local
        // TERM normalized.
        match &exec.term {
            Some(t) => {
                env.insert("TERM".into(), t.clone());
            }
            None => {
                let local = std::env::var("TERM").ok();
                env.entry("TERM".into())
                    .or_insert_with(|| stream::remote_term(local.as_deref(), None));
            }
        }
    }
    let start = ExecStart {
        argv: exec.argv.clone(),
        tty: exec.tty,
        env,
        rows: u32::from(size.1),
        cols: u32::from(size.0),
    };
    let start = match start_frame(start) {
        Ok(start) => start,
        Err(f) => return Driven::before_start(Ended::Failed(f)),
    };
    let send = call.send;
    let mut recv = call.recv;
    let mut driven = Driven {
        ended: Ended::Exit(0),
        started: true,
        answered: false,
        status: None,
    };
    if send.send(start).await.is_err() {
        // The call ended before `start` was queued: the command never ran.
        driven.started = false;
        driven.ended = Ended::Failed(Failure::new(Class::ConnectionLost, ""));
        return driven;
    }

    // stdin on a thread of its own; its channel closing is stdin ending.
    // The reader stops when `drive` returns, so input typed afterwards, as
    // during `shell --gpu`'s stop, is left for the local shell.
    let (stdin_rx, _stdin_reader) = if exec.no_stdin {
        (None, None)
    } else {
        let (stdin_tx, stdin_rx) = mpsc::channel::<stream::Input>(8);
        let reader = stream::spawn_stdin(stdin_tx, exec.tty);
        (Some(stdin_rx), Some(reader))
    };
    // stdin and resizes are sent from a task of their own, so a broker that
    // stops reading never stops this loop from reading output and signals.
    let (gone_tx, mut gone_rx) = tokio::sync::oneshot::channel::<()>();
    let mut forwarder = tokio::spawn(forward(send, stdin_rx, exec.tty, size, winch, gone_tx));
    let mut forwarding = true;
    let mut gone_open = true;
    stream::test_panic("main");

    driven.ended = loop {
        tokio::select! {
            msg = recv.message() => match msg {
                Ok(Some(f)) => {
                    driven.answered = true;
                    match f.frame {
                        Some(exec_server_frame::Frame::Data(d)) => {
                            let target = if d.stream == Stream::Stderr as i32 { &mut err_out } else { &mut out };
                            let written = tokio::select! {
                                w = target.write(&d.chunk) => w,
                                sig = signals.next() => break Ended::Signal(sig),
                            };
                            match written {
                                Ok(()) => {}
                                Err(WriteEnd::Pipe) => break Ended::Pipe,
                                Err(WriteEnd::Hangup) => break Ended::Signal(Sig::Hup),
                                Err(WriteEnd::Failed(e)) => break Ended::Failed(output_failed(e)),
                            }
                        }
                        Some(exec_server_frame::Frame::ExitCode(code)) => break Ended::Exit(code),
                        None => {}
                    }
                }
                Ok(None) => break Ended::Failed(Failure::new(Class::ConnectionLost, "")),
                Err(status) => {
                    let s = BrokerStatus::from(&status);
                    if api.debug {
                        debug(&format!("< exec ended: {s}"));
                    }
                    let f = stream::broker_failure(&s, name, Needs::Exec, true);
                    driven.status = Some(s);
                    break Ended::Failed(f);
                }
            },
            // A pty ignores EOF: stdin ending under -t means the terminal
            // is gone.
            gone = &mut gone_rx, if gone_open => match gone {
                Ok(()) => break Ended::Signal(Sig::Hup),
                Err(_) => gone_open = false,
            },
            // It runs until aborted, so it ending is a panic.
            joined = &mut forwarder, if forwarding => {
                forwarding = false;
                if let Err(e) = joined {
                    stream::rethrow(e);
                }
            }
            sig = signals.next() => break Ended::Signal(sig),
        }
    };
    // Dropping both halves resets the stream, so the broker cancels the
    // command; then the terminal is restored.
    // A panic there ends the call too, so it may surface as the loop's end.
    if forwarding {
        forwarder.abort();
        if let Err(e) = forwarder.await {
            if e.is_panic() {
                stream::rethrow(e);
            }
        }
    }
    drop(recv);
    drop(raw);
    driven
}

/// Output failed, for a write to stdout or stderr that failed.
pub(crate) fn output_failed(e: std::io::Error) -> Failure {
    Failure::new(
        Class::OutputFailed,
        format!("Could not write the output: {e}."),
    )
}

/// Resizes, as `SIGWINCH` reports them; Windows polls the console instead.
#[cfg(unix)]
type Winch = Option<tokio::signal::unix::Signal>;
#[cfg(not(unix))]
type Winch = ();

/// Sends stdin and resizes until the call ends. Without stdin (`-n`),
/// `stdin_eof` goes at once. Under `-t`, stdin ending or failing is reported
/// on `gone`; otherwise it is `stdin_eof`, after a line when a read failed.
/// Holds `send` until aborted, since closing it would end the request half.
async fn forward(
    send: calls::Sender<ExecClientFrame>,
    stdin: Option<mpsc::Receiver<stream::Input>>,
    tty: bool,
    size: (u16, u16),
    winch: Winch,
    gone: tokio::sync::oneshot::Sender<()>,
) {
    stream::test_panic("task");
    let mut gone = Some(gone);
    let mut stdin_open = stdin.is_some();
    let mut stdin = stdin.unwrap_or_else(|| mpsc::channel(1).1);
    if !stdin_open
        && send
            .send(frame(exec_client_frame::Frame::StdinEof(true)))
            .await
            .is_err()
    {
        std::future::pending::<()>().await
    }
    #[cfg(unix)]
    let mut winch = winch;
    #[cfg(not(unix))]
    let _ = winch;
    #[cfg(not(unix))]
    let mut last = size;
    #[cfg(unix)]
    let _ = size;
    loop {
        tokio::select! {
            chunk = stdin.recv(), if stdin_open => match chunk {
                Some(Ok(bytes)) => {
                    if send.send(frame(exec_client_frame::Frame::Stdin(bytes))).await.is_err() {
                        break;
                    }
                }
                end => {
                    stdin_open = false;
                    if tty {
                        if let Some(g) = gone.take() {
                            let _ = g.send(());
                        }
                        continue;
                    }
                    if let Some(Err(e)) = end {
                        stream::say_line(escape_controls(&format!(
                            "Could not read stdin ({e}); the command's input ends here."
                        )))
                        .await;
                    }
                    if send
                        .send(frame(exec_client_frame::Frame::StdinEof(true)))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            },
            _ = async {
                #[cfg(unix)]
                match winch.as_mut() {
                    Some(w) => { w.recv().await; }
                    None => std::future::pending::<()>().await,
                }
                // A Windows console sends no resize event: poll its size.
                #[cfg(not(unix))]
                if tty {
                    tokio::time::sleep(std::time::Duration::from_millis(250)).await
                } else {
                    std::future::pending::<()>().await
                }
            } => {
                let Ok((cols, rows)) = hedra_cli_sandbox::crossterm::terminal::size() else {
                    continue;
                };
                #[cfg(not(unix))]
                {
                    if (cols, rows) == last {
                        continue;
                    }
                    last = (cols, rows);
                }
                let resize = frame(exec_client_frame::Frame::Resize(ExecResize {
                    rows: u32::from(rows),
                    cols: u32::from(cols),
                }));
                if send.send(resize).await.is_err() {
                    break;
                }
            }
        }
    }
    std::future::pending::<()>().await
}

fn frame(f: exec_client_frame::Frame) -> ExecClientFrame {
    ExecClientFrame { frame: Some(f) }
}

/// The `start` frame, refused when over the broker's limit, which it
/// applies to the whole frame as it forwards it, with its own fields added.
fn start_frame(start: ExecStart) -> Result<ExecClientFrame, Failure> {
    let f = frame(exec_client_frame::Frame::Start(start));
    if hedra_cli_sandbox::prost::Message::encoded_len(&f) + BROKER_START_OVERHEAD > MAX_START_BYTES
    {
        return Err(Failure::rejected(
            "The command and its environment are over the 1 MiB a session's first frame can hold.",
        ));
    }
    Ok(f)
}

/// The Connection lost failure, after a fresh read of the sandbox.
pub(crate) async fn lost(api: &Api, id: &str, status: Option<&BrokerStatus>) -> Failure {
    let detail = stream::lost_detail(api, id).await;
    let mut f = Failure::new(
        Class::ConnectionLost,
        format!(
            "Lost the connection to the sandbox{}. {detail}",
            broker_note(status)
        ),
    );
    f.reason = status.and_then(|s| s.reason.clone());
    f
}

/// The broker's own message, which names the cause, in parentheses.
pub(crate) fn broker_note(status: Option<&BrokerStatus>) -> String {
    status
        .map(|s| escape_controls(&s.message))
        .filter(|m| !m.is_empty())
        .map(|m| format!(" ({m})"))
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hedra_cli_sandbox::prost::Message;

    fn start(len: usize) -> ExecStart {
        ExecStart {
            argv: vec!["x".repeat(len)],
            ..Default::default()
        }
    }

    #[test]
    fn the_start_limit_counts_the_whole_frame_and_the_brokers_fields() {
        // The largest argv whose frame leaves exactly the broker's room.
        let limit = MAX_START_BYTES - BROKER_START_OVERHEAD;
        let len = (limit - 16..limit)
            .rev()
            .find(|&n| frame(exec_client_frame::Frame::Start(start(n))).encoded_len() == limit)
            .expect("a frame at the limit");
        assert!(start_frame(start(len)).is_ok());
        // One byte more is refused, though the frame alone still fits.
        assert!(
            frame(exec_client_frame::Frame::Start(start(len + 1))).encoded_len() <= MAX_START_BYTES
        );
        let f = start_frame(start(len + 1)).unwrap_err();
        assert_eq!(f.class, Class::Rejected);
    }
}
