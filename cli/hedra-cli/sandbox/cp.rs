//! `cp`: files and directories in and out of a sandbox, as tar over `Exec`.

use std::sync::Arc;

use hedra_cli_sandbox::cp::{self, CopyError, CopyReport, Direction, Progress};
use hedra_cli_sandbox::session::Session;
use hedra_cli_sandbox::stream::calls::{self, Open, Waiting};
use hedra_cli_sandbox::stream::status::BrokerStatus;
use hedra_cli_sandbox::stream::transport::TransportConfig;

use super::api::Api;
use super::error::{escape_controls, Class, Failure};
use super::exec::{self, Ended};
use super::names::{self, Target};
use super::stream::{self, Minter, Needs, Signals};

/// Whether `part` names a sandbox: a valid name or a `sandbox_` id.
fn is_sandbox(part: &str) -> bool {
    names::is_sandbox_name(part) || names::is_sandbox_id(part)
}

/// Runs `cp` and returns the exit code: the remote `tar`'s on a copy in,
/// 0 for a whole copy out, 255 for hedra-cli's own failures.
pub(crate) async fn run(
    api: &Api,
    transport: TransportConfig,
    src: &str,
    dst: &str,
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
pub(crate) fn plan(src: &str, dst: &str) -> Result<(Target, Plan), Failure> {
    let spec =
        cp::parse_args(src, dst, is_sandbox).map_err(|m| Failure::rejected(escape_controls(&m)))?;
    let target = Target::parse(&spec.sandbox)?;
    let plan = match spec.direction {
        Direction::In => cp::plan_copy_in(&spec).map(Plan::In),
        Direction::Out => cp::plan_copy_out(&spec).map(Plan::Out),
    };
    Ok((target, plan.map_err(Failure::rejected)?))
}

async fn run_inner(api: &Api, transport: TransportConfig, src: &str, dst: &str) -> Ended {
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
    let proxy = transport.proxy.clone();
    let on_wait = |w: &Waiting| {
        stream::note(escape_controls(&stream::waiting_line(
            w,
            &name,
            proxy.as_deref(),
        )));
    };
    let open = Open {
        on_wait: &on_wait,
        ..Default::default()
    };
    let call = tokio::select! {
        r = calls::open_exec(&session, &open) => match r {
            Ok(call) => call,
            Err(e) => {
                let f = stream::open_failure(api, &id, &name, Needs::Exec, proxy.as_deref(), e).await;
                return discard(plan, Ended::Failed(f));
            }
        },
        sig = signals.next() => return discard(plan, Ended::Signal(sig)),
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
                p.discard();
            }
            return Ended::Signal(sig);
        }
    };
    if !report.remote_stderr.is_empty() {
        eprint!("{}", escape_lines(&report.remote_stderr));
    }
    for w in &report.warnings {
        stream::note(w);
    }
    match report.result {
        Ok(code) => Ended::Exit(code),
        Err(CopyError::Rejected(m)) => Ended::Failed(Failure::rejected(m)),
        Err(CopyError::Call(status)) => {
            let s = BrokerStatus::from(&status);
            let f = stream::broker_failure(&s, &name, Needs::Exec, true);
            if f.class == Class::ConnectionLost {
                Ended::Failed(stopped_at(
                    exec::lost(api, &id, Some(&s)).await,
                    &report.stopped_at,
                ))
            } else {
                Ended::Failed(f)
            }
        }
        Err(CopyError::Ended) => Ended::Failed(stopped_at(
            exec::lost(api, &id, None).await,
            &report.stopped_at,
        )),
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
