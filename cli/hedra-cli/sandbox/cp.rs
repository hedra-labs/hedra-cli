//! `cp`: files and directories in and out of a sandbox, as tar over `Exec`.

use std::ffi::OsStr;
use std::sync::Arc;

use hedra_cli_sandbox::cp::{self, CopyError, CopyReport, Direction, Progress};
use hedra_cli_sandbox::session::Session;
use hedra_cli_sandbox::stream::status::BrokerStatus;
use hedra_cli_sandbox::stream::transport::TransportConfig;

use super::api::Api;
use super::error::{escape_controls, Class, Failure};
use super::exec::{self, Ended};
use super::names::{self, Target};
use super::stream::{self, Minter, Needs, Out, Sig, Signals, WriteEnd};

/// Whether `part` names a sandbox: a valid name or a `sandbox_` id.
fn is_sandbox(part: &str) -> bool {
    names::is_sandbox_name(part) || names::is_sandbox_id(part)
}

/// Runs `cp` and returns the exit code: 0 for a whole copy, the remote
/// `tar`'s when it fails a copy in and hedra-cli refused nothing, and 255
/// for hedra-cli's own failures, a copy out whose `tar` failed among them.
pub(crate) async fn run(
    api: &Api,
    transport: TransportConfig,
    src: &OsStr,
    dst: &OsStr,
    json_errors: bool,
) -> i32 {
    stream::ignore_sigpipe();
    let ended = run_inner(api, transport, src, dst).await;
    exec::finish(ended, json_errors).await
}

/// A copy planned before any request.
pub(crate) enum Plan {
    In(cp::CopyIn),
    Out(cp::CopyOut),
}

/// Checks `cp`'s arguments and plans the copy, with no request and nothing
/// written; `--dry-run` stops here.
pub(crate) fn plan(src: &OsStr, dst: &OsStr) -> Result<(Target, Plan), Failure> {
    let spec =
        cp::parse_args(src, dst, is_sandbox).map_err(|m| Failure::rejected(escape_controls(&m)))?;
    let target = Target::parse(&spec.sandbox)?;
    let plan = match spec.direction {
        Direction::In => cp::plan_copy_in(&spec).map(Plan::In),
        Direction::Out => cp::plan_copy_out(&spec).map(Plan::Out),
    };
    Ok((target, plan.map_err(Failure::rejected)?))
}

async fn run_inner(api: &Api, transport: TransportConfig, src: &OsStr, dst: &OsStr) -> Ended {
    // Everything that can be refused locally is refused before a request.
    let (target, mut plan) = match plan(src, dst) {
        Ok(p) => p,
        Err(f) => return Ended::Failed(f),
    };
    if let Plan::Out(p) = &mut plan {
        if let Err(m) = p.prepare() {
            return Ended::Failed(Failure::rejected(m));
        }
    }
    copy(api, transport, &target, plan).await
}

async fn copy(api: &Api, transport: TransportConfig, target: &Target, plan: Plan) -> Ended {
    let name = target.as_str().to_string();
    let mut signals = Signals::new();
    let id = tokio::select! {
        r = names::resolve(api, target) => match r {
            Ok(id) => id,
            Err(f) => return discard(plan, Ended::Failed(f)),
        },
        sig = signals.next() => return discard(plan, Ended::Signal(sig)),
    };
    let session = Session::new(
        Minter {
            api,
            id: id.clone(),
        },
        transport.clone(),
    );
    let call = match exec::open_call(api, &session, &id, &name, &mut signals).await {
        Ok(call) => call,
        Err(ended) => return discard(plan, ended),
    };

    let progress = Arc::new(Progress::default());
    // What a copy out created, for a signal to remove.
    let mut created = match &plan {
        Plan::Out(p) => Some(p.clone()),
        Plan::In(_) => None,
    };
    // Dropping the copy on a signal drops both halves of the call, which
    // cancels the remote tar.
    let report: CopyReport<hedra_cli_sandbox::tonic::Status> = tokio::select! {
        r = async {
            match plan {
                Plan::In(p) => cp::copy_in(p, call.send, call.recv, progress.clone()).await,
                Plan::Out(p) => cp::copy_out(p, call.send, call.recv, progress.clone()).await,
            }
        } => r,
        sig = signals.next() => {
            if let Some(p) = &mut created {
                // The extractor, still running, begins no other entry, so
                // whether it began one says whether it wrote anything.
                // An entry it began may still be landing, so the parents
                // stay and the line says where.
                if progress.cancel() {
                    stream::note(format!(
                        "A partial copy was left at {}.",
                        escape_controls(&p.target().display().to_string())
                    ));
                } else {
                    p.discard();
                }
            }
            return Ended::Signal(sig);
        }
    };
    // A failed copy out leaves what it wrote, and says where, also when a
    // signal ends `cp` while it reports the failure.
    let partial = report.partial.as_deref();
    let signalled = |sig: Sig| {
        if let Some(p) = partial {
            stream::note(format!("A partial copy was left at {p}."));
        }
        Ended::Signal(sig)
    };
    // The copy has ended, so a signal or a failed write here leaves what it
    // wrote in place.
    if !report.remote_stderr.is_empty() {
        match relay_stderr(&report.remote_stderr, &mut signals).await {
            Some(Ended::Signal(sig)) => return signalled(sig),
            Some(ended) => return ended,
            None => {}
        }
    }
    for w in &report.warnings {
        stream::note(w);
    }
    let with_partial = |f: Failure| match partial {
        Some(p) => f.then(format!("A partial copy was left at {p}.")),
        None => f,
    };
    let status = match report.result {
        Ok(code) => return Ended::Exit(code),
        Err(CopyError::Rejected(m)) => {
            let m = match partial {
                Some(p) => format!("{m}; a partial copy was left at {p}"),
                None => m,
            };
            return Ended::Failed(Failure::rejected(m));
        }
        Err(CopyError::Call(status)) => {
            let s = BrokerStatus::from(&status);
            if api.debug {
                stream::note(escape_controls(&format!("< exec ended: {s}")));
            }
            let f = stream::broker_failure(&s, &name, Needs::Exec, true);
            if f.class != Class::ConnectionLost {
                return Ended::Failed(with_partial(f));
            }
            Some(s)
        }
        Err(CopyError::Ended) => None,
    };
    match exec::lost_unless_signalled(api, &id, status.as_ref(), &mut signals).await {
        Ok(f) => Ended::Failed(with_partial(stopped_at(f, &report.stopped_at))),
        Err(sig) => signalled(sig),
    }
}

/// Writes the sandbox's stderr through a duplicate of fd 2, as `exec` writes
/// a command's, so a failed write ends `cp` as it ends `exec`: Output
/// failed, a closed pipe by SIGPIPE, a gone terminal as a hangup. A signal
/// still ends a write that waits on a stderr nobody reads.
async fn relay_stderr(bytes: &[u8], signals: &mut Signals) -> Option<Ended> {
    let mut err_out = match Out::dup(2) {
        Ok(o) => o,
        Err(e) => return Some(Ended::Failed(exec::output_failed(e))),
    };
    let text = escape_lines(bytes);
    let written = tokio::select! {
        w = err_out.write(text.as_bytes()) => w,
        sig = signals.next() => return Some(Ended::Signal(sig)),
    };
    match written {
        Ok(()) => None,
        Err(WriteEnd::Pipe) => Some(Ended::Pipe),
        Err(WriteEnd::Hangup) => Some(Ended::Signal(Sig::Hup)),
        Err(WriteEnd::Failed(e)) => Some(Ended::Failed(exec::output_failed(e))),
    }
}

/// Removes what a copy out created for itself, when it ends before its call.
fn discard(plan: Plan, ended: Ended) -> Ended {
    if let Plan::Out(mut p) = plan {
        p.discard();
    }
    ended
}

/// The sandbox's stderr with each line's control characters escaped, since
/// it carries names from the sandbox.
fn escape_lines(b: &[u8]) -> String {
    String::from_utf8_lossy(b)
        .split_inclusive('\n')
        .map(|l| match l.strip_suffix('\n') {
            Some(l) => format!("{}\n", escape_controls(l)),
            None => escape_controls(l),
        })
        .collect()
}

/// Names the entry the copy stopped at, when it knows.
fn stopped_at(f: Failure, at: &Option<String>) -> Failure {
    match at {
        Some(entry) => f.then(format!("The copy stopped at {}.", escape_controls(entry))),
        None => f,
    }
}
