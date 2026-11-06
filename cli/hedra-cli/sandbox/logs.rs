//! `logs`, `logs -f` and `attach`: the main process's output, live from the
//! broker's ring buffer while the instance runs, and from the stored copy
//! once it has stopped.

use std::time::Duration;

use hedra_cli_sandbox::pb::hedra::sandbox::v1::{LogChunk, LogsRequest};
use hedra_cli_sandbox::prost_types::Timestamp;
use hedra_cli_sandbox::session::{Session, SessionError};
use hedra_cli_sandbox::stream::calls::{self, Open, OpenError};
use hedra_cli_sandbox::stream::spool::Spool;
use hedra_cli_sandbox::stream::status::{reason, BrokerStatus};
use hedra_cli_sandbox::stream::transport::TransportConfig;
use hedra_cli_sdk::api::prelude::{DateTime, Utc};
use hedra_cli_sdk::api::{Sandbox, SandboxInstanceStatus, SandboxStatus};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio::time::Instant;

use super::api::{attempt, Api, CallError, ATTEMPT};
use super::error::{escape_controls, Class, Failure};
use super::lifecycle::{self, Timing};
use super::names::{self, Target};
use super::render;
use super::stream::{self, Minter, Needs, Out, Sig, Signals, WriteEnd};

/// What `logs -f` holds in memory before it spills to a temporary file.
const MEMORY: usize = 8 * 1024 * 1024;
/// What it holds in all before it stops reading.
const TOTAL: u64 = 1024 * 1024 * 1024;
/// A starting sandbox's wait: the pool's 15 minutes.
const START_BUDGET: Duration = Duration::from_secs(15 * 60);
const MAX_BACKOFF: Duration = Duration::from_secs(30);

enum Ended {
    Done,
    Failed(Failure),
    Signal(Sig),
    Pipe,
}

/// Runs `logs` (`follow` false), or `logs -f` and `attach` (`follow` true),
/// and returns the exit code. Errors go to stderr: stdout is the output.
pub(crate) async fn run(
    api: &Api,
    transport: TransportConfig,
    target: &Target,
    follow: bool,
    json_errors: bool,
) -> i32 {
    stream::ignore_sigpipe();
    let ended = match Out::dup(1) {
        Ok(out) => {
            let mut signals = Signals::new();
            let mut job = Logs {
                api,
                transport,
                name: target.as_str().to_string(),
                follow,
                out: Writer::spawn(out),
                since: None,
                printed: false,
            };
            let ended = tokio::select! {
                e = job.run(target) => e,
                sig = signals.next() => Ended::Signal(sig),
            };
            match ended {
                // A signal exits without waiting on a stdout that may be
                // stalled.
                Ended::Signal(sig) => Ended::Signal(sig),
                // What was handed to the writer is written before the exit,
                // unless a signal comes first.
                ended => tokio::select! {
                    written = job.out.finish() => match (ended, written) {
                        (Ended::Done, Err(end)) => write_end(end),
                        (ended, _) => ended,
                    },
                    sig = signals.next() => Ended::Signal(sig),
                },
            }
        }
        Err(_) => Ended::Failed(Failure::new(
            Class::OutputFailed,
            "Could not open the output.",
        )),
    };
    match ended {
        Ended::Done => 0,
        Ended::Pipe => stream::die_by_sigpipe(),
        // Ctrl-C is how a follow ends.
        Ended::Signal(Sig::Int) if follow => 0,
        Ended::Signal(sig) => sig.class().code(),
        Ended::Failed(f) => {
            f.write_stderr(json_errors);
            f.class.code()
        }
    }
}

struct Logs<'a> {
    api: &'a Api,
    transport: TransportConfig,
    name: String,
    follow: bool,
    out: Writer,
    /// Where the next read resumes: just after the last chunk printed.
    since: Option<Timestamp>,
    printed: bool,
}

/// Where a sandbox is, for reading its output.
enum Phase {
    NeverRan,
    Starting,
    Running,
    Stopped,
}

fn phase(s: &Sandbox) -> Phase {
    let instances = s.instances.as_deref().unwrap_or_default();
    match s.status {
        SandboxStatus::Failed => Phase::NeverRan,
        SandboxStatus::Ended if instances.is_empty() => Phase::NeverRan,
        SandboxStatus::Ended | SandboxStatus::Stopping => Phase::Stopped,
        SandboxStatus::Creating => Phase::Starting,
        _ => match stream::replica0(s).map(|i| &i.status) {
            None | Some(SandboxInstanceStatus::Starting) => Phase::Starting,
            Some(SandboxInstanceStatus::Running) => Phase::Running,
            Some(_) => Phase::Stopped,
        },
    }
}

impl Logs<'_> {
    async fn get(&self, id: &str) -> Result<Sandbox, Failure> {
        attempt(ATTEMPT, self.api.client.get(id, None))
            .await
            .map_err(|e| e.failure(Some(&self.name)))
    }

    async fn run(&mut self, target: &Target) -> Ended {
        let id = match names::resolve(self.api, target).await {
            Ok(id) => id,
            Err(f) => return Ended::Failed(f),
        };
        let session = Session::new(
            Minter {
                api: self.api,
                id: id.clone(),
            },
            self.transport.clone(),
        );
        // Until a call is admitted, an open retries for the 60 s every verb
        // takes; after, a reopen takes the follow's own backoff.
        let mut admitted = false;
        let mut backoff = Duration::from_secs(1);
        let mut first = true;
        loop {
            let read = attempt(ATTEMPT, self.api.client.get(&id, None)).await;
            let s = match read {
                Ok(s) => s,
                // A reopen rides out a read that may succeed later, as it
                // does a dropped stream.
                Err(e) if !first && e.transient() => {
                    stream::note(format_args!(
                        "Could not read `{}` to reopen its output; retrying.",
                        self.name
                    ));
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                    continue;
                }
                Err(e) => return Ended::Failed(e.failure(Some(&self.name))),
            };
            first = false;
            match phase(&s) {
                Phase::NeverRan => {
                    let reason = render::stop_reason(&s)
                        .map(|r| format!(" ({})", escape_controls(&r)))
                        .unwrap_or_default();
                    stream::note(format_args!(
                        "`{}` never started, so it has no output{reason}.",
                        self.name
                    ));
                    return Ended::Done;
                }
                Phase::Stopped => return self.stored_tail(&id, &s).await,
                Phase::Starting if !self.follow => {
                    stream::note(format_args!("`{}` has no output yet.", self.name));
                    return Ended::Done;
                }
                Phase::Starting => {
                    let timing = Timing {
                        ready_budget: START_BUDGET,
                        ..Timing::default()
                    };
                    if let Err(f) = lifecycle::wait_ready(
                        self.api,
                        &id,
                        &self.name,
                        Instant::now(),
                        &timing,
                        lifecycle::Waiter::Reader,
                        // The create's seconds-to-ready line does not apply here.
                        &mut |line: &str| {
                            if !line.contains("after the create") {
                                stream::note(format_args!("{line}"));
                            }
                        },
                    )
                    .await
                    {
                        let s = match self.get(&id).await {
                            Ok(s) => s,
                            Err(_) => return Ended::Failed(f),
                        };
                        return match phase(&s) {
                            // Still starting: the budget ran out.
                            Phase::Starting => Ended::Failed(f),
                            // Replica 0 runs, whatever another replica did.
                            Phase::Running => continue,
                            _ => {
                                stream::note(format_args!("{}", escape_controls(&f.message)));
                                continue;
                            }
                        };
                    }
                    continue;
                }
                Phase::Running => {}
            }

            let request = LogsRequest {
                since: self.since,
                follow: self.follow,
            };
            let name = self.name.clone();
            // The proxy only when the broker does not bypass it.
            let proxy = || session.proxy();
            let on_wait = move |w: &calls::Waiting| {
                stream::note(escape_controls(&stream::waiting_line(
                    w,
                    &name,
                    proxy().as_deref(),
                )));
            };
            let debug = |line: &str| stream::note(escape_controls(line));
            let open = Open {
                retry_for: if !admitted && self.follow {
                    calls::ADMISSION_RETRY
                } else {
                    Duration::ZERO
                },
                on_wait: &on_wait,
                on_debug: self.api.debug.then_some(&debug as _),
                ..Default::default()
            };
            let opened = calls::open_logs(&session, request, &open).await;
            let mut stream = match opened {
                Ok(stream) => {
                    admitted = true;
                    stream
                }
                Err(OpenError::Refused(st))
                    if !self.follow && st.is(reason::SANDBOX_DISCONNECTED) =>
                {
                    return self.stored_tail(&id, &s).await;
                }
                Err(OpenError::Session(SessionError::Mint(e))) if stopped_mint(&e) => {
                    let s = self.get(&id).await.unwrap_or(s);
                    return self.stored_tail(&id, &s).await;
                }
                Err(OpenError::Refused(st))
                    if self.follow
                        && admitted
                        && hedra_cli_sandbox::stream::status::retry_before_admission(&st) =>
                {
                    on_wait(&calls::waiting_for(&st));
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                    continue;
                }
                Err(e) => {
                    return Ended::Failed(
                        stream::open_failure(
                            self.api,
                            &id,
                            &self.name,
                            Needs::Read,
                            proxy().as_deref(),
                            e,
                        )
                        .await,
                    )
                }
            };

            match self.pump(&mut stream, &mut backoff).await {
                Pumped::Eof => return Ended::Done,
                Pumped::End(e) => return e,
                Pumped::Dropped(status) => {
                    if let (true, Some(s)) = (self.api.debug, &status) {
                        debug(&format!("< logs ended: {s}"));
                    }
                    if !self.follow {
                        if status
                            .as_ref()
                            .is_some_and(|s| s.is(reason::SANDBOX_DISCONNECTED))
                        {
                            let s = self.get(&id).await.unwrap_or(s);
                            return self.stored_tail(&id, &s).await;
                        }
                        // The ring buffer replayed; the stream ending is the end.
                        if status.is_none() {
                            return Ended::Done;
                        }
                        let st = status.expect("checked");
                        let f = stream::broker_failure(&st, &self.name, Needs::Read, true);
                        if f.class == Class::ConnectionLost {
                            return Ended::Failed(
                                super::exec::lost(self.api, &id, Some(&st)).await,
                            );
                        }
                        return Ended::Failed(f);
                    }
                    // A follow reopens after its backoff; the next pass reads
                    // the sandbox again, so a stop ends it with the tail.
                    tokio::time::sleep(backoff).await;
                    backoff = (backoff * 2).min(MAX_BACKOFF);
                }
            }
        }
    }

    /// Prints a stream's chunks. Reading runs ahead of a slow stdout into
    /// a spool, so the stream is not held up until the spool is full.
    async fn pump(
        &mut self,
        stream: &mut hedra_cli_sandbox::tonic::Streaming<LogChunk>,
        backoff: &mut Duration,
    ) -> Pumped {
        let mut spool = Spool::new(MEMORY, TOTAL);
        let mut open = true;
        let mut ending = Pumped::Dropped(None);
        loop {
            if !open && spool.is_empty() {
                return ending;
            }
            let room = spool.has_room();
            if open && !room {
                stream::note(format_args!(
                    "Output is arriving faster than it is read; some may be missing."
                ));
                // The broker ends the session; a follow reopens from the last
                // printed chunk once this drains.
                open = false;
                continue;
            }
            let tx = self.out.tx.clone();
            tokio::select! {
                // A failed write ends the writer before the next chunk.
                _ = tx.closed() => {
                    return Pumped::End(match self.out.take_error().await {
                        Some(end) => write_end(end),
                        None => Ended::Done,
                    });
                }
                permit = tx.reserve(), if !spool.is_empty() => {
                    let Ok(permit) = permit else {
                        return Pumped::End(match self.out.take_error().await {
                            Some(end) => write_end(end),
                            None => Ended::Done,
                        });
                    };
                    match spool.pop() {
                        Ok(Some(chunk)) => permit.send(chunk),
                        Ok(None) => {}
                        Err(e) => return Pumped::End(buffer_failed(e)),
                    }
                }
                msg = stream.message(), if open => match msg {
                    Ok(Some(chunk)) => {
                        if chunk.eof {
                            open = false;
                            ending = Pumped::Eof;
                            continue;
                        }
                        if let Some(at) = chunk.at {
                            self.since = Some(next_nanosecond(at));
                        }
                        *backoff = Duration::from_secs(1);
                        self.printed = true;
                        if let Err(e) = spool.push(&chunk.data) {
                            return Pumped::End(buffer_failed(e));
                        }
                    }
                    Ok(None) => open = false,
                    Err(status) => {
                        open = false;
                        ending = Pumped::Dropped(Some(BrokerStatus::from(&status)));
                    }
                },
            }
        }
    }

    /// The stored copy after the last printed chunk, then how the sandbox
    /// ended.
    async fn stored_tail(&mut self, id: &str, s: &Sandbox) -> Ended {
        let since = self.since.map(rfc3339);
        let stored = match self.api.logs(id, since.as_deref()).await {
            Ok(stored) => stored,
            Err(e)
                if e.http()
                    .is_some_and(|h| h.message() == "The sandbox has no instances.") =>
            {
                stream::note(format_args!(
                    "`{}` never started, so it has no output.",
                    self.name
                ));
                return Ended::Done;
            }
            Err(e) => return Ended::Failed(e.failure(Some(&self.name))),
        };
        if self.out.tx.send(stored.body).await.is_err() {
            if let Some(end) = self.out.take_error().await {
                return write_end(end);
            }
        }
        if stored.truncated {
            stream::note(format_args!(
                "Some output was dropped before the first stored line printed here; the stored \
                 copy keeps only its newest part."
            ));
        }
        if self.printed {
            stream::note(format_args!(
                "The live output ended; the rest is from the stored copy, which can miss the \
                 last few seconds."
            ));
        }
        if self.follow {
            stream::note(format_args!("{}", stream::state_line(s, &self.name)));
        }
        Ended::Done
    }
}

/// Writes stdout on a task of its own, in order.
struct Writer {
    tx: mpsc::Sender<Vec<u8>>,
    task: Option<JoinHandle<Result<(), WriteEnd>>>,
}

impl Writer {
    fn spawn(mut out: Out) -> Self {
        let (tx, mut rx) = mpsc::channel::<Vec<u8>>(1);
        let task = tokio::spawn(async move {
            while let Some(chunk) = rx.recv().await {
                stream::test_panic("task");
                out.write(&chunk).await?;
            }
            Ok(())
        });
        Self {
            tx,
            task: Some(task),
        }
    }

    /// The write failure that ended the task, once it has. A panic in the
    /// task carries on here.
    async fn take_error(&mut self) -> Option<WriteEnd> {
        match self.task.take()?.await {
            Ok(Err(end)) => Some(end),
            Ok(Ok(())) => None,
            Err(e) => stream::rethrow(e),
        }
    }

    /// Waits for everything sent to be written.
    async fn finish(&mut self) -> Result<(), WriteEnd> {
        let (closed, _) = mpsc::channel(1);
        drop(std::mem::replace(&mut self.tx, closed));
        match self.task.take() {
            Some(task) => task.await.unwrap_or_else(|e| stream::rethrow(e)),
            None => Ok(()),
        }
    }
}

fn write_end(end: WriteEnd) -> Ended {
    match end {
        WriteEnd::Pipe => Ended::Pipe,
        WriteEnd::Hangup => Ended::Signal(Sig::Hup),
        WriteEnd::Failed(e) => Ended::Failed(Failure::new(
            Class::OutputFailed,
            format!("Could not write the output: {e}."),
        )),
    }
}

fn buffer_failed(e: std::io::Error) -> Ended {
    Ended::Failed(Failure::new(
        Class::OutputFailed,
        format!("Could not buffer the output: {e}."),
    ))
}

enum Pumped {
    /// The ring buffer was replayed (`logs` without `-f`).
    Eof,
    /// The stream ended: by the broker, the network, or a full spool.
    Dropped(Option<BrokerStatus>),
    End(Ended),
}

/// A mint that can never succeed again: the sandbox or its replica stopped.
fn stopped_mint(e: &CallError) -> bool {
    e.http()
        .is_some_and(|h| h.status == 404 || h.is(400, "FAILED_PRECONDITION"))
}

/// The instant one nanosecond after `at`.
pub(crate) fn next_nanosecond(at: Timestamp) -> Timestamp {
    if at.nanos >= 999_999_999 {
        Timestamp {
            seconds: at.seconds + 1,
            nanos: 0,
        }
    } else {
        Timestamp {
            seconds: at.seconds,
            nanos: at.nanos + 1,
        }
    }
}

/// RFC 3339 in UTC with nanoseconds, as the logs route's `since` takes it.
pub(crate) fn rfc3339(at: Timestamp) -> String {
    DateTime::<Utc>::from_timestamp(at.seconds, at.nanos.max(0) as u32)
        .unwrap_or_default()
        .format("%Y-%m-%dT%H:%M:%S%.9fZ")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resume_is_one_nanosecond_after_the_last_chunk() {
        assert_eq!(
            next_nanosecond(Timestamp {
                seconds: 5,
                nanos: 7
            }),
            Timestamp {
                seconds: 5,
                nanos: 8
            }
        );
        assert_eq!(
            next_nanosecond(Timestamp {
                seconds: 5,
                nanos: 999_999_999
            }),
            Timestamp {
                seconds: 6,
                nanos: 0
            }
        );
    }

    #[test]
    fn since_is_rfc3339_with_nanoseconds_in_utc() {
        assert_eq!(
            rfc3339(Timestamp {
                seconds: 1_793_750_400,
                nanos: 1
            }),
            "2026-11-04T00:00:00.000000001Z"
        );
    }
}
