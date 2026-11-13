//! The stream verbs' plumbing: the session minter over the v3 session route,
//! the broker transport settings, output through descriptors of their own,
//! signals and the terminal.

use std::time::{Duration, SystemTime};

use hedra_cli_sandbox::backoff::Backoff;
use hedra_cli_sandbox::session::{MintFuture, Minted, SessionError, SessionMinter};
use hedra_cli_sandbox::stream::calls::{waiting_for, OpenError, Waiting};
use hedra_cli_sandbox::stream::status::{reason, BrokerStatus};
use hedra_cli_sandbox::stream::transport::{self, TransportConfig, TransportError};
use hedra_cli_sdk::api::{
    SandboxEndReason, SandboxFailureReason, SandboxInstanceStatus, SandboxStatus,
    SessionQueryRequest,
};
use tokio::time::Instant;

use super::api::{attempt, Api, CallError, ATTEMPT};
use super::error::{escape_controls, Class, Failure};
use super::lifecycle::retry_wait;
use super::render;

/// How long a mint keeps retrying.
const MINT_RETRY: Duration = Duration::from_secs(60);
/// Between a mint's retries: the cap keeps several tries inside the minute.
const MINT_BACKOFF: Backoff = Backoff::new(Duration::from_secs(1), Duration::from_secs(8));

/// Mints session JWTs for one replica of one sandbox through the v3 route.
pub(crate) struct Minter<'a> {
    pub(crate) api: &'a Api,
    pub(crate) id: String,
    /// The replica the session reaches, from 0.
    pub(crate) replica: i64,
}

impl SessionMinter for Minter<'_> {
    type Error = CallError;

    fn mint(&self) -> MintFuture<'_, CallError> {
        Box::pin(mint(self.api, &self.id, self.replica))
    }
}

/// A minter that owns its client, for a session that outlives a borrow, such
/// as `port-forward`'s.
pub(crate) struct OwnedMinter {
    pub(crate) api: std::sync::Arc<Api>,
    pub(crate) id: String,
    /// The replica the session reaches, from 0.
    pub(crate) replica: i64,
}

impl SessionMinter for OwnedMinter {
    type Error = CallError;

    fn mint(&self) -> MintFuture<'_, CallError> {
        Box::pin(mint(&self.api, &self.id, self.replica))
    }
}

/// `POST /sandboxes/{id}/session?replica=N`, retried on 408, 429, 5xx and
/// transport errors for up to a minute; a second mint is harmless.
async fn mint(api: &Api, id: &str, replica: i64) -> Result<Minted, CallError> {
    let deadline = Instant::now() + MINT_RETRY;
    let mut backoff = MINT_BACKOFF;
    let request = SessionQueryRequest {
        replica: Some(replica),
    };
    loop {
        match attempt(ATTEMPT, api.client.session(id, &request, None)).await {
            Ok(session) => {
                return Ok(Minted {
                    token: session.token,
                    expires_at: SystemTime::from(session.expires_at),
                    date: api.last_date(),
                    broker_url: session.broker_url,
                })
            }
            Err(e) => {
                let wait = retry_wait(&e, backoff.wait());
                if !e.transient() || Instant::now() + wait >= deadline {
                    return Err(e);
                }
                tokio::time::sleep(wait).await;
                backoff.grow();
            }
        }
    }
}

/// The broker transport, from the runtime's HTTP settings. `HEDRA_CLI_PROXY`
/// wins; without it the standard proxy variables apply, in reqwest's order.
pub(crate) fn transport_config(
    http: &fern_cli_sdk::http::HttpConfig,
) -> Result<TransportConfig, Failure> {
    let resolved = http
        .resolve()
        .map_err(|e| Failure::rejected(e.to_string()))?;
    // The first variable that is set decides, even when it is empty, as
    // the REST client's proxy matcher reads them.
    let first_set = |keys: &[&'static str]| {
        keys.iter()
            .find_map(|&k| std::env::var(k).ok().map(|v| (k, v)))
            .filter(|(_, v)| !v.is_empty())
    };
    let (proxy, no_proxy) = match resolved.proxy {
        Some(p) => (Some(("HEDRA_CLI_PROXY", p.url)), p.no_proxy),
        None => (
            first_set(&["HTTPS_PROXY", "https_proxy"])
                .or_else(|| first_set(&["ALL_PROXY", "all_proxy"])),
            first_set(&["NO_PROXY", "no_proxy"]).map(|(_, v)| v),
        ),
    };
    let (proxy_var, proxy) = match proxy {
        Some((var, url)) => (Some(var.to_string()), Some(url)),
        None => (None, None),
    };
    let config = TransportConfig {
        proxy,
        proxy_var,
        no_proxy,
        extra_roots_pem: resolved.extra_root_certs_pem,
        insecure: resolved.insecure_skip_verify,
        connect_timeout: resolved.connect_timeout,
        user_agent: http.user_agent(),
    };
    // With a bypass list the broker may not use the proxy at all, which is
    // known only once the session names the broker, so a proxy it would use
    // is refused when the channel is built instead.
    if config.no_proxy.is_none() {
        transport::check_proxy(&config)
            .map_err(|e| Failure::rejected(escape_controls(&e.to_string())))?;
    }
    Ok(config)
}

/// What a retry before admission prints, on stderr.
pub(crate) fn waiting_line(w: &Waiting, name: &str, proxy: Option<&str>) -> String {
    match w {
        Waiting::Supervisor => {
            format!("`{name}`'s supervisor is not connected yet; retrying.")
        }
        Waiting::Broker(_) => match proxy {
            Some(p) => format!(
                "Could not reach the sandbox broker through the proxy {}; retrying.",
                transport::redact(p)
            ),
            None => format!("Could not reach the sandbox broker; retrying. {SETUP_CLOSE}"),
        },
    }
}

const SETUP_CLOSE: &str = "If this continues, the connection may be closing during setup, \
                           which also happens past the broker's per-network connection limit.";

/// Why the broker could not be reached, for the Unavailable line an open
/// ends with.
fn unreached(proxy: Option<&str>) -> String {
    match proxy {
        Some(p) => format!(
            "The sandbox broker could not be reached through the proxy {}.",
            transport::redact(p)
        ),
        None => SETUP_CLOSE.to_string(),
    }
}

/// The scope a verb needs, named when a broker `permission_denied` names none.
#[derive(Clone, Copy)]
pub(crate) enum Needs {
    Exec,
    Read,
    Write,
}

impl Needs {
    fn scope(self) -> &'static str {
        match self {
            Needs::Exec => "sandboxes:exec",
            Needs::Read => "sandboxes:read",
            Needs::Write => "sandboxes:write",
        }
    }
}

/// The session scope a broker `permission_denied` names in its message
/// ("token lacks the sandbox:exec scope"), without the `sandbox:` prefix.
/// The broker sends the scope only in the message, so this is the one
/// place to read it.
fn scope_in_message(message: &str) -> Option<&'static str> {
    message
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == ':' || c == '_'))
        .find_map(|word| match word {
            "sandbox:read" => Some("read"),
            "sandbox:write" => Some("write"),
            "sandbox:exec" => Some("exec"),
            _ => None,
        })
}

/// The exit-code class for a broker status. `started` is whether `start`
/// or `open` was sent: after it, any end that is not a reason the table
/// names is a dropped stream.
pub(crate) fn broker_failure(s: &BrokerStatus, name: &str, needs: Needs, started: bool) -> Failure {
    let message = escape_controls(&s.message);
    let class = match s.reason.as_deref() {
        Some(reason::PERMISSION_DENIED) => {
            let scope = scope_in_message(&s.message)
                .map(|session_scope| format!("sandboxes:{session_scope}"))
                .unwrap_or_else(|| needs.scope().to_string());
            let mut f = Failure::new(
                Class::MissingScope,
                super::error::missing_scope_line(&scope),
            );
            f.reason = s.reason.clone();
            return f;
        }
        Some(reason::QUOTA_EXCEEDED) => Class::Limit,
        Some(reason::SANDBOX_DISCONNECTED) if !started => {
            let mut f = Failure::new(Class::NotReady, not_connected_line(name, name));
            f.reason = s.reason.clone();
            return f;
        }
        Some(reason::NOT_FOUND) => Class::NotReady,
        Some(reason::INTERNAL) => Class::Internal,
        Some(reason::INVALID_ARGUMENT) => Class::Rejected,
        Some(reason::RATE_LIMITED) if !started => Class::Limit,
        // A second refusal of a just-minted token is the broker's key
        // cache, not the key.
        Some(reason::UNAUTHENTICATED) if !started => return temporarily_unavailable(s, &message),
        _ if started => Class::ConnectionLost,
        _ => match s.code {
            hedra_cli_sandbox::tonic::Code::Unavailable
            | hedra_cli_sandbox::tonic::Code::DeadlineExceeded
            | hedra_cli_sandbox::tonic::Code::Unauthenticated => {
                return temporarily_unavailable(s, &message)
            }
            hedra_cli_sandbox::tonic::Code::PermissionDenied => {
                return Failure::new(
                    Class::MissingScope,
                    super::error::missing_scope_line(needs.scope()),
                )
            }
            hedra_cli_sandbox::tonic::Code::NotFound => Class::NotReady,
            hedra_cli_sandbox::tonic::Code::ResourceExhausted => Class::Limit,
            _ => Class::Internal,
        },
    };
    let mut f = Failure::new(
        class,
        if message.is_empty() {
            format!("{:?}", s.code)
        } else {
            message
        },
    );
    f.reason = s.reason.clone();
    f
}

/// Unavailable (17), with the broker's escaped `message` when it gave one.
fn temporarily_unavailable(s: &BrokerStatus, message: &str) -> Failure {
    let line = if message.is_empty() {
        "Sandbox sessions are temporarily unavailable; retry shortly.".to_string()
    } else {
        format!("Sandbox sessions are temporarily unavailable ({message}); retry shortly.")
    };
    let mut f = Failure::new(Class::Unavailable, line);
    f.reason = s.reason.clone();
    f
}

fn not_connected_line(name: &str, id: &str) -> String {
    format!(
        "`{name}`'s supervisor is not connected. Run `{}`.",
        render::get_command(id)
    )
}

/// A failure to open a call, classed for its exit code. A mint refused 404,
/// or 400 FAILED_PRECONDITION, for a sandbox just resolved reads the sandbox
/// and reports its state, unless a 400's sandbox still reads active. An
/// active sandbox is judged by `replica`, the one the session was for.
pub(crate) async fn open_failure(
    api: &Api,
    id: &str,
    name: &str,
    replica: i64,
    needs: Needs,
    proxy: Option<&str>,
    e: OpenError<CallError>,
) -> Failure {
    match e {
        OpenError::Refused(s) => {
            let mut f = broker_failure(&s, name, needs, false);
            if s.reason.as_deref() == Some(reason::SANDBOX_DISCONNECTED) {
                // The name as typed may be one of several sandboxes' over
                // time; the id names the one the session was minted for.
                f.message = not_connected_line(name, id);
            }
            let unreachable = hedra_cli_sandbox::stream::status::retry_before_admission(&s)
                && matches!(waiting_for(&s), Waiting::Broker(_));
            if f.class == Class::Unavailable && unreachable {
                f.then(escape_controls(&unreached(proxy)))
            } else {
                f
            }
        }
        OpenError::Session(SessionError::Transport(TransportError::Config(m))) => {
            Failure::rejected(escape_controls(&m))
        }
        OpenError::Session(SessionError::Transport(TransportError::Refused(m))) => {
            Failure::rejected(format!(
                "Could not connect to the sandbox broker: {}.",
                escape_controls(&m)
            ))
        }
        OpenError::Session(SessionError::Transport(TransportError::Connect(m))) => Failure::new(
            Class::Unavailable,
            format!(
                "Sandbox sessions are temporarily unavailable ({}); retry shortly. {}",
                escape_controls(&m),
                escape_controls(&unreached(proxy))
            ),
        ),
        OpenError::Session(SessionError::Mint(e)) => {
            // A 404 says only that the session route found no live replica, so
            // the sandbox's own state names why. A 400 FAILED_PRECONDITION says
            // either that the sandbox is not active, where its state and end
            // reason name why, or that the replica's instance has stopped while
            // the sandbox is still active, where the route's message is the
            // line. Either way the line is Not ready or stopped (13) and ends
            // with the next step. A failed read keeps the route's message,
            // unless the read answers 404 too.
            let Some(h) = e.http() else {
                return e.failure(Some(name));
            };
            let precondition = h.is(400, "FAILED_PRECONDITION");
            if h.status != 404 && !precondition {
                return e.failure(Some(name));
            }
            // A read that also answers 404 is the line: the sandbox is gone,
            // and a `get` would say so too. Any other failed read keeps the
            // route's message and adds a step, since the message alone
            // names nothing to do.
            let s = match attempt(ATTEMPT, api.client.get(id, None)).await {
                Ok(s) => s,
                Err(r) if r.http().is_some_and(|h| h.status == 404) => {
                    return r.failure(Some(name));
                }
                Err(_) => return e.failure(Some(name)).then(get_line(id)),
            };
            // An active sandbox whose lease was refused (`unpaid`) still runs
            // until the refusal's grace ends, and a paid lease clears it, so
            // an active sandbox is judged by its replica, as `not_running`
            // judges it.
            if s.status != SandboxStatus::Active {
                return Failure::new(Class::NotReady, state_line(&s, name, replica))
                    .then(next_step(&s, replica));
            }
            match (exited_line(&s, name, replica), precondition) {
                // The route's message already says the instance stopped, so
                // only a failure reason adds to it.
                (Some(_), true)
                    if replica_at(&s, replica).is_some_and(|i| i.failure_reason.is_none()) =>
                {
                    e.failure(Some(name)).then(ended_step(&s, replica))
                }
                (Some(exited), true) => e.failure(Some(name)).then(exited),
                (Some(exited), false) => Failure::new(Class::NotReady, exited),
                (None, true) => e.failure(Some(name)).then(get_line(id)),
                (None, false) => {
                    Failure::new(Class::NotReady, state_line(&s, name, replica)).then(get_line(id))
                }
            }
        }
    }
}

/// The Not ready or stopped (13) failure for a sandbox read as not running:
/// stopping, ended, failed, or active with `replica` stopped or failed. None
/// while it is still being created, or in a status hedra-cli does not know, or
/// `replica` is running, starting or missing: those may yet run. An active
/// sandbox whose lease was refused (`unpaid`) still runs until the refusal's
/// grace ends, and a paid lease clears it, so it is judged by its replica.
pub(crate) fn not_running(
    s: &hedra_cli_sdk::api::Sandbox,
    name: &str,
    replica: i64,
) -> Option<Failure> {
    let stopped = matches!(
        s.status,
        SandboxStatus::Stopping | SandboxStatus::Ended | SandboxStatus::Failed
    );
    if stopped {
        return Some(
            Failure::new(Class::NotReady, state_line(s, name, replica)).then(next_step(s, replica)),
        );
    }
    if s.status != SandboxStatus::Active {
        return None;
    }
    exited_line(s, name, replica).map(|exited| Failure::new(Class::NotReady, exited))
}

/// The refusal for a replica `s` does not have, before any stream opens: one
/// past every instance the read lists and at or past its `replicas`, as the
/// session route bounds `replica` by the instances it lists. An instance the
/// platform started to replace a lost one is listed beside it, so a sandbox
/// can list more instances than its `replicas`, and each is reachable. One
/// under `replicas` but not listed yet may still be placed, and the verbs
/// wait for it as for one starting. Replica 0, which every sandbox has and
/// every verb reached before `--replica`, is never refused here.
pub(crate) fn missing_replica(
    s: &hedra_cli_sdk::api::Sandbox,
    name: &str,
    replica: i64,
) -> Option<Failure> {
    let listed = s.instances.as_deref().unwrap_or_default();
    if replica == 0 || listed.iter().any(|i| i.replica == replica) {
        return None;
    }
    let replicas = s.replicas.max(1);
    let count = listed.len() as i64;
    if replica < replicas.max(count) {
        return None;
    }
    let range = |n: i64| match n {
        1 => "replica 0".to_string(),
        n => format!("0 to {}", n - 1),
    };
    let has = if count > replicas {
        format!(
            "it has {replicas} replica{} and lists {count} instances, {}",
            if replicas == 1 { "" } else { "s" },
            range(count)
        )
    } else {
        format!(
            "it has {replicas} replica{}, {}",
            if replicas == 1 { "" } else { "s" },
            range(replicas)
        )
    };
    Some(Failure::rejected(escape_controls(&format!(
        "`{name}` has no replica {replica}: {has}."
    ))))
}

/// Who a line names for `replica`: the sandbox's instance for replica 0, as
/// a one-replica sandbox reads, else that replica.
pub(crate) fn whose(name: &str, replica: i64) -> String {
    match replica {
        0 => format!("`{name}`'s instance"),
        n => format!("Replica {n} of `{name}`"),
    }
}

/// What an active sandbox's stopped `replica` means, then the next step.
/// None unless the read shows that replica stopped or failed: a replica
/// missing, starting or in a state hedra-cli does not know may yet run.
/// A stopped replica is not always a command that exited: an idle or TTL
/// stop reads the same while the sandbox is still active.
fn exited_line(s: &hedra_cli_sdk::api::Sandbox, name: &str, replica: i64) -> Option<String> {
    let instance = replica_at(s, replica).filter(|i| {
        matches!(
            i.status,
            SandboxInstanceStatus::Stopped | SandboxInstanceStatus::Failed
        )
    })?;
    let whose = whose(name, replica);
    let step = ended_step(s, replica);
    let line = match &instance.failure_reason {
        Some(f) => format!("{whose} stopped ({f}). {step}"),
        None => format!("{whose} stopped. {step}"),
    };
    Some(escape_controls(&line))
}

/// The next step for a sandbox that is not active: funds when it was stopped
/// unpaid, a retry while it is still being created, and otherwise what
/// `ended_step` names. Commands name the sandbox by id: once it ends, its
/// name is free for a new sandbox.
pub(crate) fn next_step(s: &hedra_cli_sdk::api::Sandbox, replica: i64) -> String {
    if s.end_reason == Some(SandboxEndReason::Unpaid) {
        return format!(
            "Add funds to the workspace, then create another; `{}` lists the running ones.",
            render::LS_COMMAND
        );
    }
    match s.status {
        SandboxStatus::Creating => check_line(&s.id),
        SandboxStatus::Stopping | SandboxStatus::Ended | SandboxStatus::Failed => {
            ended_step(s, replica)
        }
        _ => get_line(&s.id),
    }
}

/// The next step once `replica` can run nothing more: its output when it
/// started, whatever ended it, since a started instance's start time is kept
/// through any later stop or failure; otherwise what its failure calls for.
pub(crate) fn ended_step(s: &hedra_cli_sdk::api::Sandbox, replica: i64) -> String {
    if replica_at(s, replica).is_some_and(|i| i.started_at.is_some()) {
        return output_line(&s.id, replica);
    }
    // Past replica 0, an active sandbox's other replicas may still run, so
    // creating another is not the step.
    if replica != 0 && s.status == SandboxStatus::Active {
        return get_line(&s.id);
    }
    // That replica's failure first, as the state line reads it; another
    // replica's when it has none, so a full pool still says so.
    let failure = replica_at(s, replica)
        .and_then(|i| i.failure_reason.as_ref())
        .or_else(|| {
            s.instances
                .iter()
                .flatten()
                .find_map(|i| i.failure_reason.as_ref())
        });
    match failure {
        Some(SandboxFailureReason::NoCapacity) => capacity_line(&s.gpu),
        Some(
            SandboxFailureReason::QuotaExceeded
            | SandboxFailureReason::ImagePullFailed
            | SandboxFailureReason::StartFailed
            | SandboxFailureReason::ProcessExited
            | SandboxFailureReason::OomKilled
            | SandboxFailureReason::GpuError
            | SandboxFailureReason::PlatformError
            | SandboxFailureReason::Other
            | SandboxFailureReason::__Unknown(_),
        )
        | None => another_line(),
        // The type is non-exhaustive, so a variant a later regeneration adds
        // reaches here; a never-started instance has nothing to read.
        Some(_) => another_line(),
    }
}

/// "Run `hedra-cli sandbox logs <id>` for its output", with `--replica N`
/// past replica 0.
fn output_line(id: &str, replica: i64) -> String {
    escape_controls(&format!(
        "Run `{}` for its output.",
        render::logs_command(id, replica)
    ))
}

/// For a sandbox that can run nothing more.
fn another_line() -> String {
    format!(
        "Create another, or run `{}` for the running ones.",
        render::LS_COMMAND
    )
}

/// For a sandbox that never started because no GPU of its shape was free.
fn capacity_line(gpu: &str) -> String {
    escape_controls(&format!(
        "No `{gpu}` was free. Run `{}`, then create another.",
        render::GPUS_COMMAND
    ))
}

/// For a sandbox still being created.
fn check_line(id: &str) -> String {
    escape_controls(&format!(
        "Retry once it is running; check on it with `{}`.",
        render::get_command(id)
    ))
}

/// For a sandbox whose state could not be read, or reads as running.
pub(crate) fn get_line(id: &str) -> String {
    escape_controls(&format!("Check on it with `{}`.", render::get_command(id)))
}

/// "`<name>` is <status>.", with the stop reason or `replica`'s failure,
/// or that its instance stopped when an active sandbox's `replica` has, the
/// server's text escaped.
pub(crate) fn state_line(s: &hedra_cli_sdk::api::Sandbox, name: &str, replica: i64) -> String {
    let line = match render::stop_reason(s) {
        Some(reason) => format!("`{name}` is {} ({reason}).", s.status),
        None => match replica_at(s, replica).and_then(|i| i.failure_reason.as_ref()) {
            Some(f) => format!("{} stopped ({f}).", whose(name, replica)),
            // An active sandbox reads as running without this.
            None if s.status == SandboxStatus::Active
                && replica_at(s, replica).is_some_and(|i| {
                    matches!(
                        i.status,
                        SandboxInstanceStatus::Stopped | SandboxInstanceStatus::Failed
                    )
                }) =>
            {
                format!("{} stopped.", whose(name, replica))
            }
            None => format!("`{name}` is {}.", s.status),
        },
    };
    escape_controls(&line)
}

/// The instance at `replica`, when the read lists it.
pub(crate) fn replica_at(
    s: &hedra_cli_sdk::api::Sandbox,
    replica: i64,
) -> Option<&hedra_cli_sdk::api::SandboxInstance> {
    s.instances.iter().flatten().find(|i| i.replica == replica)
}

/// The Connection lost line's second sentence, from a fresh read of the
/// sandbox, judged by `replica`, the one the session was for.
pub(crate) async fn lost_detail(api: &Api, id: &str, replica: i64) -> String {
    let Ok(s) = attempt(ATTEMPT, api.client.get(id, None)).await else {
        return "The sandbox is still running.".to_string();
    };
    let reason = render::stop_reason(&s).map(|r| escape_controls(&r));
    match s.status {
        SandboxStatus::Ended => format!(
            "The sandbox stopped: {}.",
            reason.unwrap_or_else(|| "no reason given".into())
        ),
        SandboxStatus::Stopping => format!(
            "The sandbox is stopping: {}.",
            reason.unwrap_or_else(|| "no reason given".into())
        ),
        _ => instance_stopped(&s, replica)
            .unwrap_or_else(|| "The sandbox is still running.".to_string()),
    }
}

/// "The sandbox's instance stopped (<reason>)." when an active sandbox's
/// `replica` has stopped or failed, naming the replica past 0.
pub(crate) fn instance_stopped(s: &hedra_cli_sdk::api::Sandbox, replica: i64) -> Option<String> {
    let i = replica_at(s, replica).filter(|i| {
        matches!(
            i.status,
            SandboxInstanceStatus::Stopped | SandboxInstanceStatus::Failed
        )
    })?;
    let whose = match replica {
        0 => "The sandbox's instance".to_string(),
        n => format!("The sandbox's replica {n}"),
    };
    Some(match &i.failure_reason {
        Some(f) => format!("{whose} stopped ({}).", escape_controls(&f.to_string())),
        None => format!("{whose} stopped."),
    })
}

// ---------------------------------------------------------------------------
// Output, signals and the terminal (Unix).
// ---------------------------------------------------------------------------

/// What a write to stdout or stderr failing means.
pub(crate) enum WriteEnd {
    /// The reader of a pipe went away.
    Pipe,
    /// The terminal is gone.
    Hangup,
    Failed(std::io::Error),
}

fn classify_write(e: std::io::Error) -> WriteEnd {
    #[cfg(unix)]
    match e.raw_os_error() {
        Some(libc::EPIPE) => return WriteEnd::Pipe,
        Some(libc::EIO) | Some(libc::ENXIO) => return WriteEnd::Hangup,
        _ => {}
    }
    // Windows reports a pipe whose reader left as ERROR_NO_DATA (232).
    #[cfg(windows)]
    if e.raw_os_error() == Some(232) {
        return WriteEnd::Pipe;
    }
    if e.kind() == std::io::ErrorKind::BrokenPipe {
        return WriteEnd::Pipe;
    }
    WriteEnd::Failed(e)
}

/// A duplicate of fd 1 or fd 2, written on a blocking thread, so no write
/// takes std's stdout lock, which the runtime holds for the whole run.
#[cfg(unix)]
pub(crate) struct Out {
    fd: std::sync::Arc<std::os::fd::OwnedFd>,
    /// A write whose caller stopped waiting, finished before the next.
    busy: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
}

/// A duplicate of the standard output or error handle; a console's writes
/// go through `WriteConsoleW`, so non-ASCII text survives its code page.
#[cfg(windows)]
pub(crate) struct Out(hedra_cli_sandbox::console::Output);

#[cfg(windows)]
impl Out {
    pub(crate) fn dup(fd: i32) -> std::io::Result<Self> {
        use hedra_cli_sandbox::console::{Output, Std};
        Ok(Self(Output::dup(if fd == 2 {
            Std::Err
        } else {
            Std::Out
        })?))
    }

    pub(crate) async fn write(&mut self, bytes: &[u8]) -> Result<(), WriteEnd> {
        match self.0.write(bytes).await {
            Ok(()) => Ok(()),
            // A console that went away is a gone terminal, not a pipe.
            Err(e) if self.0.console_gone(&e) => Err(WriteEnd::Hangup),
            Err(e) => Err(classify_write(e)),
        }
    }
}

#[cfg(unix)]
impl Out {
    pub(crate) fn dup(fd: i32) -> std::io::Result<Self> {
        use std::os::fd::FromRawFd;
        // SAFETY: dup returns a new descriptor this process owns.
        let new = unsafe { libc::dup(fd) };
        if new < 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Self {
            // SAFETY: `new` is a valid, owned descriptor.
            fd: std::sync::Arc::new(unsafe { std::os::fd::OwnedFd::from_raw_fd(new) }),
            busy: None,
        })
    }

    pub(crate) async fn write(&mut self, bytes: &[u8]) -> Result<(), WriteEnd> {
        use std::os::fd::AsRawFd;
        let failed = |e: tokio::task::JoinError| WriteEnd::Failed(std::io::Error::other(e));
        if let Some(earlier) = self.busy.as_mut() {
            let _ = earlier.await;
            self.busy = None;
        }
        let fd = self.fd.clone();
        let data = bytes.to_vec();
        let write = self.busy.insert(tokio::task::spawn_blocking(move || {
            write_all_fd(fd.as_raw_fd(), &data)
        }));
        let written = write.await.map_err(failed)?;
        self.busy = None;
        written.map_err(classify_write)
    }
}

/// Writes all of `bytes` to `fd`. A descriptor left `O_NONBLOCK` by another
/// process sharing it is waited on until writable, never switched back to
/// blocking, since the flag is shared.
#[cfg(unix)]
pub(crate) fn write_all_fd(fd: i32, mut bytes: &[u8]) -> std::io::Result<()> {
    while !bytes.is_empty() {
        // SAFETY: `bytes` is a valid buffer of the given length.
        let n = unsafe { libc::write(fd, bytes.as_ptr().cast(), bytes.len()) };
        if n > 0 {
            bytes = &bytes[n as usize..];
            continue;
        }
        if n == 0 {
            return Err(std::io::ErrorKind::WriteZero.into());
        }
        let e = std::io::Error::last_os_error();
        match e.kind() {
            std::io::ErrorKind::Interrupted => {}
            std::io::ErrorKind::WouldBlock => {
                wait_ready(fd, libc::POLLOUT, None);
            }
            _ => return Err(e),
        }
    }
    Ok(())
}

/// Waits until `fd` is ready for `events` (`POLLIN` or `POLLOUT`) or
/// `timeout` passes, and says whether to try the read or write. macOS's
/// `poll` does not take devices such as `/dev/null`, so those wait in
/// `select`.
#[cfg(unix)]
fn wait_ready(fd: i32, events: libc::c_short, timeout: Option<Duration>) -> bool {
    let ms = timeout.map_or(-1, |t| t.as_millis().min(i32::MAX as u128) as libc::c_int);
    let mut p = libc::pollfd {
        fd,
        events,
        revents: 0,
    };
    // SAFETY: polls one descriptor.
    let rc = unsafe { libc::poll(&mut p, 1, ms) };
    if rc == 0 {
        return false;
    }
    if rc > 0 && p.revents & libc::POLLNVAL == 0 {
        return true;
    }
    if rc < 0 && std::io::Error::last_os_error().kind() == std::io::ErrorKind::Interrupted {
        return false;
    }
    if (0..libc::FD_SETSIZE as i32).contains(&fd) {
        // SAFETY: an fd_set holding one descriptor below FD_SETSIZE.
        let rc = unsafe {
            let mut set: libc::fd_set = std::mem::zeroed();
            libc::FD_ZERO(&mut set);
            libc::FD_SET(fd, &mut set);
            let mut tv = timeout.map(|t| libc::timeval {
                tv_sec: t.as_secs() as libc::time_t,
                tv_usec: t.subsec_micros() as libc::suseconds_t,
            });
            let tv = tv
                .as_mut()
                .map_or(std::ptr::null_mut(), |t| t as *mut libc::timeval);
            let (read, write) = if events == libc::POLLIN {
                (&mut set as *mut libc::fd_set, std::ptr::null_mut())
            } else {
                (std::ptr::null_mut(), &mut set as *mut libc::fd_set)
            };
            libc::select(fd + 1, read, write, std::ptr::null_mut(), tv)
        };
        if rc >= 0 {
            return rc > 0;
        }
    }
    // Neither can wait on it: try again shortly rather than spin.
    std::thread::sleep(Duration::from_millis(10));
    true
}

/// Signals a stream verb listens for, leaving out any the process started
/// with ignored, so `nohup` keeps working.
#[cfg(unix)]
pub(crate) struct Signals {
    int: Option<tokio::signal::unix::Signal>,
    term: Option<tokio::signal::unix::Signal>,
    hup: Option<tokio::signal::unix::Signal>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sig {
    Int,
    Term,
    Hup,
    /// Windows' `CTRL_CLOSE_EVENT`: Windows ends the process a few seconds
    /// later.
    #[cfg_attr(not(windows), allow(dead_code))]
    Close,
}

impl Sig {
    pub(crate) fn class(self) -> Class {
        match self {
            Sig::Int => Class::Interrupted,
            Sig::Term => Class::Terminated,
            Sig::Hup | Sig::Close => Class::Hangup,
        }
    }
}

/// Where a verb's signals come from; tests inject their own.
pub(crate) trait NextSig {
    async fn next_sig(&mut self) -> Sig;
}

impl NextSig for Signals {
    async fn next_sig(&mut self) -> Sig {
        self.next().await
    }
}

/// How long hedra-cli waits for one of its own stderr lines to be written.
const LINE_WAIT: Duration = Duration::from_secs(1);
/// How long an exit waits for the lines still queued, its own last line
/// among them, unless a signal came, before or during the wait. It waits so even when an earlier line
/// outlasted `LINE_WAIT`: a writer starved on a busy machine still gets the
/// final line out, and a stderr nobody reads costs the exit this much.
const EXIT_WAIT: Duration = Duration::from_secs(10);

/// Whether a signal has arrived, after which every wait is short: the
/// terminal may be gone, and a Ctrl-C no longer ends the process.
static SIGNALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
/// How often the exit flush looks at `SIGNALLED`.
const FLUSH_SLICE: Duration = Duration::from_millis(50);

/// Sets `flag` (`SIGNALLED`) on each signal `stream` delivers, for as long
/// as the runtime runs: the verb's own `Signals` stops being polled once it
/// returns, and a signal during the exit flush must still shorten it.
fn watch<S, F>(flag: &'static std::sync::atomic::AtomicBool, mut stream: S, mut recv: F)
where
    S: Send + 'static,
    F: for<'a> FnMut(
            &'a mut S,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = bool> + Send + 'a>>
        + Send
        + 'static,
{
    let Ok(rt) = tokio::runtime::Handle::try_current() else {
        return;
    };
    rt.spawn(async move {
        while recv(&mut stream).await {
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    });
}

/// What a written line reports back on.
enum Ack {
    Sync(std::sync::mpsc::SyncSender<()>),
    Async(tokio::sync::oneshot::Sender<()>),
}

/// Whether raw mode is on, so hedra-cli's own lines end in "\r\n".
static RAW: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// hedra-cli's own stderr lines, written in order on a thread of their own,
/// so a stderr nobody reads holds a verb up for a second at most.
struct Lines {
    tx: std::sync::mpsc::SyncSender<(String, Ack)>,
    /// A write has outlasted its wait: later lines are queued, or dropped
    /// once the queue is full, without waiting, until the writer catches up.
    stalled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl Lines {
    /// A writer thread writing each queued line with `write`.
    fn spawn(mut write: impl FnMut(&[u8]) + Send + 'static) -> Self {
        let (tx, rx) = std::sync::mpsc::sync_channel::<(String, Ack)>(64);
        let stalled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let caught_up = stalled.clone();
        std::thread::spawn(move || {
            for (line, ack) in rx {
                // A raw terminal does not return the carriage on a newline.
                let line = if RAW.load(std::sync::atomic::Ordering::SeqCst) {
                    line.replace('\n', "\r\n")
                } else {
                    line
                };
                write(line.as_bytes());
                caught_up.store(false, std::sync::atomic::Ordering::SeqCst);
                match ack {
                    Ack::Sync(a) => {
                        let _ = a.try_send(());
                    }
                    Ack::Async(a) => {
                        let _ = a.send(());
                    }
                }
            }
        });
        Lines { tx, stalled }
    }

    /// Queues `line`, and says whether to wait for it: not when it was
    /// dropped, nor once an earlier write has stalled.
    fn queue(&self, line: String, ack: Ack) -> bool {
        self.tx.try_send((line, ack)).is_ok()
            && !self.stalled.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Waits up to `wait` for every line queued so far to be written,
    /// stalled or not.
    fn flush(&self, wait: Duration) {
        self.flush_until(wait, &SIGNALLED);
    }

    /// `flush`, reading `signalled` for a signal that cuts the wait.
    fn flush_until(&self, wait: Duration, signalled: &std::sync::atomic::AtomicBool) {
        let mut deadline = std::time::Instant::now() + wait;
        // A signal that arrives during the wait cuts it to `LINE_WAIT`, so a
        // Ctrl-C still ends an exit held up by a stderr nobody reads.
        let cut = |deadline: &mut std::time::Instant| {
            if signalled.load(std::sync::atomic::Ordering::SeqCst) {
                *deadline = (*deadline).min(std::time::Instant::now() + LINE_WAIT);
            }
        };
        let (ack, done) = std::sync::mpsc::sync_channel(1);
        // An empty line writes nothing, and its ack comes after every
        // earlier one. A full queue is retried until there is room.
        let mut marker = (String::new(), Ack::Sync(ack));
        loop {
            match self.tx.try_send(marker) {
                Ok(()) => break,
                Err(std::sync::mpsc::TrySendError::Full(m)) => marker = m,
                Err(std::sync::mpsc::TrySendError::Disconnected(_)) => return,
            }
            cut(&mut deadline);
            if std::time::Instant::now() >= deadline {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        loop {
            cut(&mut deadline);
            let left = deadline.saturating_duration_since(std::time::Instant::now());
            if left.is_zero() {
                return;
            }
            match done.recv_timeout(left.min(FLUSH_SLICE)) {
                Ok(()) | Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => return,
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            }
        }
    }

    fn mark_stalled(&self) {
        self.stalled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

fn lines() -> &'static Lines {
    static LINES: std::sync::OnceLock<Lines> = std::sync::OnceLock::new();
    LINES.get_or_init(|| {
        Lines::spawn(|bytes| {
            #[cfg(unix)]
            let _ = write_all_fd(2, bytes);
            #[cfg(not(unix))]
            let _ = std::io::Write::write_all(&mut std::io::stderr(), bytes);
        })
    })
}

/// Starts the stderr writer, so the first line does not wait on a thread
/// being spawned.
pub(crate) fn start_lines() {
    let _ = lines();
}

/// Waits for every queued stderr line, the exit's own last line among them,
/// to be written before an exit: up to `EXIT_WAIT`, or `LINE_WAIT` once a
/// signal has arrived.
pub(crate) fn flush_lines() {
    let wait = if SIGNALLED.load(std::sync::atomic::Ordering::SeqCst) {
        LINE_WAIT
    } else {
        EXIT_WAIT
    };
    lines().flush(wait);
}

/// Writes a line to stderr, waiting at most a second for it and ignoring a
/// failure, since after a signal the terminal may be gone.
pub(crate) fn note(line: impl std::fmt::Display) {
    let (ack, done) = std::sync::mpsc::sync_channel(1);
    if lines().queue(format!("{line}\n"), Ack::Sync(ack)) && done.recv_timeout(LINE_WAIT).is_err() {
        lines().mark_stalled();
    }
}

/// `note`, awaited.
pub(crate) async fn say_line(line: impl std::fmt::Display) {
    let (ack, done) = tokio::sync::oneshot::channel();
    if lines().queue(format!("{line}\n"), Ack::Async(ack))
        && tokio::time::timeout(LINE_WAIT, done).await.is_err()
    {
        lines().mark_stalled();
    }
}

#[cfg(unix)]
fn ignored(sig: libc::c_int) -> bool {
    // SAFETY: reads the current disposition into a zeroed struct.
    unsafe {
        let mut old: libc::sigaction = std::mem::zeroed();
        libc::sigaction(sig, std::ptr::null(), &mut old) == 0 && old.sa_sigaction == libc::SIG_IGN
    }
}

#[cfg(unix)]
impl Signals {
    pub(crate) fn new() -> Self {
        Self::watching(&SIGNALLED)
    }

    /// `new`, with the watchers setting `flag`.
    fn watching(flag: &'static std::sync::atomic::AtomicBool) -> Self {
        use tokio::signal::unix::{signal, SignalKind};
        let listen = |sig: libc::c_int, kind: SignalKind| {
            if ignored(sig) {
                None
            } else {
                signal(kind).ok()
            }
        };
        for (sig, kind) in [
            (libc::SIGINT, SignalKind::interrupt()),
            (libc::SIGTERM, SignalKind::terminate()),
            (libc::SIGHUP, SignalKind::hangup()),
        ] {
            if let Some(stream) = listen(sig, kind) {
                watch(flag, stream, |s| {
                    Box::pin(async move { s.recv().await.is_some() })
                });
            }
        }
        Self {
            int: listen(libc::SIGINT, SignalKind::interrupt()),
            term: listen(libc::SIGTERM, SignalKind::terminate()),
            hup: listen(libc::SIGHUP, SignalKind::hangup()),
        }
    }

    pub(crate) async fn next(&mut self) -> Sig {
        async fn on(s: &mut Option<tokio::signal::unix::Signal>) {
            match s {
                Some(s) => {
                    s.recv().await;
                }
                None => std::future::pending().await,
            }
        }
        let sig = tokio::select! {
            _ = on(&mut self.int) => Sig::Int,
            _ = on(&mut self.term) => Sig::Term,
            _ = on(&mut self.hup) => Sig::Hup,
        };
        SIGNALLED.store(true, std::sync::atomic::Ordering::SeqCst);
        sig
    }
}

/// Ctrl-C, Ctrl-Break and the console closing. The close handler does not
/// return, so Windows ends the process only after the stop has had its few
/// seconds.
#[cfg(windows)]
pub(crate) struct Signals {
    int: Option<tokio::signal::windows::CtrlC>,
    brk: Option<tokio::signal::windows::CtrlBreak>,
    close: Option<tokio::signal::windows::CtrlClose>,
}

#[cfg(windows)]
impl Signals {
    pub(crate) fn new() -> Self {
        use tokio::signal::windows::{ctrl_break, ctrl_c, ctrl_close};
        if let Ok(s) = ctrl_c() {
            watch(&SIGNALLED, s, |s| {
                Box::pin(async move { s.recv().await.is_some() })
            });
        }
        if let Ok(s) = ctrl_break() {
            watch(&SIGNALLED, s, |s| {
                Box::pin(async move { s.recv().await.is_some() })
            });
        }
        Self {
            int: ctrl_c().ok(),
            brk: ctrl_break().ok(),
            close: ctrl_close().ok(),
        }
    }

    pub(crate) async fn next(&mut self) -> Sig {
        let sig = tokio::select! {
            _ = async { match &mut self.int { Some(s) => { s.recv().await; } None => std::future::pending().await } } => Sig::Int,
            _ = async { match &mut self.brk { Some(s) => { s.recv().await; } None => std::future::pending().await } } => Sig::Term,
            _ = async { match &mut self.close { Some(s) => { s.recv().await; } None => std::future::pending().await } } => Sig::Close,
        };
        SIGNALLED.store(true, std::sync::atomic::Ordering::SeqCst);
        sig
    }
}

#[cfg(windows)]
pub(crate) fn ignore_sigpipe() {}

/// A write to a closed pipe exits as a shell reports SIGPIPE, with no line.
#[cfg(windows)]
pub(crate) fn die_by_sigpipe() -> ! {
    std::process::exit(141)
}

/// Which verb asked for a terminal, for the alternative its refusal offers.
#[derive(Clone, Copy)]
pub(crate) enum NoTerminal {
    /// `exec -t`: the same command runs without a terminal.
    ExecT,
    /// `shell`: a command runs without one through `exec`.
    Shell,
    /// `shell --gpu`: no form of it runs without a terminal.
    GpuShell,
}

impl NoTerminal {
    /// The sentence's end: the alternative, if there is one, and the stop.
    fn alternative(self) -> &'static str {
        match self {
            NoTerminal::ExecT => ", or drop -t.",
            NoTerminal::Shell => {
                ", or run a command without one with `hedra-cli sandbox exec <sandbox> -- <command>`."
            }
            NoTerminal::GpuShell => ".",
        }
    }
}

/// Whether stdin and stdout are consoles. Under mintty and Git Bash they are
/// msys pipes, which `IsTerminal` accepts but raw mode cannot drive.
#[cfg(windows)]
pub(crate) fn foreground_terminal(without: NoTerminal) -> Result<(), Failure> {
    use hedra_cli_sandbox::console::{is_console, Std};
    if is_console(Std::In) && is_console(Std::Out) {
        return Ok(());
    }
    Err(Failure::rejected(format!(
        "This needs a console on stdin and stdout: use Windows Terminal or a console window \
         (mintty and Git Bash are not consoles){}",
        without.alternative()
    )))
}

/// Ignores SIGPIPE while a stream is open, so a broker reset ends the stream
/// with `EPIPE` instead of killing the process.
#[cfg(unix)]
pub(crate) fn ignore_sigpipe() {
    // SAFETY: setting a signal's disposition to SIG_IGN.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }
}

/// Ends the process by SIGPIPE, as a shell expects of `cmd | head`.
#[cfg(unix)]
pub(crate) fn die_by_sigpipe() -> ! {
    // SAFETY: restoring the default action and raising the signal.
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        libc::raise(libc::SIGPIPE);
    }
    std::process::exit(141)
}

/// Whether stdin and stdout are terminals and this process is in the
/// terminal's foreground process group.
#[cfg(unix)]
pub(crate) fn foreground_terminal(without: NoTerminal) -> Result<(), Failure> {
    use std::io::IsTerminal;
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        return Err(Failure::rejected(format!(
            "This needs a terminal on stdin and stdout: run it in a terminal{}",
            without.alternative()
        )));
    }
    // SAFETY: plain queries of the controlling terminal and process group.
    let (fg, own) = unsafe { (libc::tcgetpgrp(0), libc::getpgrp()) };
    if fg != own {
        return Err(Failure::rejected(format!(
            "This needs the terminal: run it in the foreground{}",
            without.alternative()
        )));
    }
    Ok(())
}

/// Raw mode, restored when dropped.
#[cfg(windows)]
pub(crate) struct RawMode {
    _console: hedra_cli_sandbox::console::RawConsole,
}

#[cfg(windows)]
impl RawMode {
    pub(crate) fn enter() -> Result<Self, Failure> {
        let raw = hedra_cli_sandbox::console::RawConsole::enter().map_err(|e| {
            Failure::rejected(format!("Could not put the console in raw mode: {e}."))
        })?;
        RAW.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(RawMode { _console: raw })
    }
}

#[cfg(windows)]
impl Drop for RawMode {
    fn drop(&mut self) {
        RAW.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// Raw mode, restored when dropped.
#[cfg(unix)]
pub(crate) struct RawMode;

#[cfg(unix)]
impl RawMode {
    pub(crate) fn enter() -> Result<Self, Failure> {
        hedra_cli_sandbox::crossterm::terminal::enable_raw_mode().map_err(|e| {
            Failure::rejected(format!("Could not put the terminal in raw mode: {e}."))
        })?;
        RAW.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(RawMode)
    }
}

#[cfg(unix)]
impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = hedra_cli_sandbox::crossterm::terminal::disable_raw_mode();
        RAW.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

/// What the stdin reader sends: a read, or the error that ended stdin. The
/// channel closing is stdin ending.
pub(crate) type Input = std::io::Result<Vec<u8>>;

/// Stops the stdin reader when dropped, before its next read, so input typed
/// after a stream ends stays for the shell.
pub(crate) struct StdinReader(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl Drop for StdinReader {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::SeqCst);
    }
}

/// How often a stdin reader waiting for input checks whether to stop.
const STDIN_CHECK: Duration = Duration::from_millis(100);

/// Reads stdin on a thread of its own and sends each read. A Windows console
/// under `-t` is read with `ReadConsoleW`, so keys arrive as UTF-8 and Ctrl-Z
/// as 0x1A.
pub(crate) fn spawn_stdin(tx: tokio::sync::mpsc::Sender<Input>, tty: bool) -> StdinReader {
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stopped = stop.clone();
    let _ = tty;
    std::thread::spawn(move || {
        let stopped = || stopped.load(std::sync::atomic::Ordering::SeqCst);
        #[cfg(windows)]
        if tty {
            // Reads that keep returning nothing mean the console is gone.
            const EMPTY_READS: u32 = 8;
            let mut reader = hedra_cli_sandbox::console::ConsoleReader::new();
            let mut empty = 0;
            loop {
                if stopped() {
                    return;
                }
                if !reader.wait(STDIN_CHECK) {
                    continue;
                }
                if stopped() {
                    return;
                }
                let Ok(bytes) = reader.read() else { return };
                if bytes.is_empty() {
                    empty += 1;
                    if empty == EMPTY_READS {
                        return;
                    }
                    continue;
                }
                empty = 0;
                if tx.blocking_send(Ok(bytes)).is_err() {
                    return;
                }
            }
        }
        let mut buf = vec![0u8; 32 * 1024];
        loop {
            if stopped() {
                return;
            }
            #[cfg(unix)]
            if !wait_ready(0, libc::POLLIN, Some(STDIN_CHECK)) {
                continue;
            }
            if stopped() {
                return;
            }
            match read_stdin(&mut buf) {
                Ok(0) => return,
                Ok(n) => {
                    if tx.blocking_send(Ok(buf[..n].to_vec())).is_err() {
                        return;
                    }
                }
                Err(e)
                    if matches!(
                        e.kind(),
                        std::io::ErrorKind::Interrupted | std::io::ErrorKind::WouldBlock
                    ) => {}
                Err(e) => {
                    let _ = tx.blocking_send(Err(e));
                    return;
                }
            }
        }
    });
    StdinReader(stop)
}

/// One read of fd 0, past std's buffer; a closed fd 0 reads as empty.
#[cfg(unix)]
fn read_stdin(buf: &mut [u8]) -> std::io::Result<usize> {
    // SAFETY: `buf` is a valid buffer of the given length.
    let n = unsafe { libc::read(0, buf.as_mut_ptr().cast(), buf.len()) };
    if n >= 0 {
        return Ok(n as usize);
    }
    let e = std::io::Error::last_os_error();
    if e.raw_os_error() == Some(libc::EBADF) {
        return Ok(0);
    }
    Err(e)
}

#[cfg(not(unix))]
fn read_stdin(buf: &mut [u8]) -> std::io::Result<usize> {
    use std::io::Read;
    std::io::stdin().read(buf)
}

/// Carries a spawned task's panic on in the caller, so it unwinds to the
/// verb's handler as the main future's own would.
pub(crate) fn rethrow(e: tokio::task::JoinError) -> ! {
    match e.try_into_panic() {
        Ok(payload) => std::panic::resume_unwind(payload),
        Err(e) => panic!("a task of hedra-cli's was cancelled: {e}"),
    }
}

/// Test hook (debug builds only): panics when `HEDRA_CLI_TEST_PANIC` names `place`.
pub(crate) fn test_panic(place: &str) {
    #[cfg(debug_assertions)]
    if std::env::var("HEDRA_CLI_TEST_PANIC").as_deref() == Ok(place) {
        panic!("injected panic in the {place}");
    }
    #[cfg(not(debug_assertions))]
    let _ = place;
}

/// `TERM` for the remote pty: names images commonly ship terminfo for pass
/// through, anything else becomes `xterm-256color`.
pub(crate) fn remote_term(local: Option<&str>, explicit: Option<&str>) -> String {
    const KNOWN: [&str; 12] = [
        "xterm",
        "xterm-256color",
        "xterm-color",
        "screen",
        "screen-256color",
        "tmux",
        "tmux-256color",
        "linux",
        "vt100",
        "vt220",
        "ansi",
        "dumb",
    ];
    if let Some(t) = explicit {
        return t.to_string();
    }
    match local {
        Some(t) if KNOWN.contains(&t) => t.to_string(),
        _ => "xterm-256color".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A signal during the exit flush cuts its wait to `LINE_WAIT`: a stderr
    /// nobody reads no longer holds a Ctrl-C'd exit for the full bound.
    #[test]
    fn a_signal_during_the_exit_flush_cuts_its_wait() {
        let lines = super::Lines::spawn(|_| std::thread::sleep(std::time::Duration::from_secs(60)));
        let (ack, _done) = std::sync::mpsc::sync_channel(1);
        assert!(lines.queue("stuck\n".into(), super::Ack::Sync(ack)));
        let signalled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let flag = signalled.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
        });
        let started = std::time::Instant::now();
        lines.flush_until(super::EXIT_WAIT, &signalled);
        let took = started.elapsed();
        assert!(
            took < super::LINE_WAIT + std::time::Duration::from_secs(1),
            "{took:?}"
        );
    }

    /// A writer that has stalled still gets the exit's last line out: the
    /// exit waits for it up to its bound.
    #[test]
    fn an_exit_waits_for_its_last_line_after_the_writer_stalled() {
        let written = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let into = written.clone();
        // A starved writer: each write takes longer than a line's wait.
        let lines = super::Lines::spawn(move |bytes| {
            std::thread::sleep(std::time::Duration::from_millis(1500));
            into.lock().unwrap().extend_from_slice(bytes);
        });
        let (ack, done) = std::sync::mpsc::sync_channel(1);
        assert!(lines.queue("first\n".into(), super::Ack::Sync(ack)));
        assert!(done.recv_timeout(super::LINE_WAIT).is_err());
        lines.mark_stalled();
        let (ack, _done) = std::sync::mpsc::sync_channel(1);
        assert!(!lines.queue(
            "hedra-cli: Internal (1): last\n".into(),
            super::Ack::Sync(ack)
        ));
        lines.flush(super::EXIT_WAIT);
        assert_eq!(
            String::from_utf8_lossy(&written.lock().unwrap()),
            "first\nhedra-cli: Internal (1): last\n"
        );
    }

    /// A signal that arrives once the verb has stopped polling its `Signals`
    /// still marks the process signalled, so the exit flush is cut short. A
    /// flag of its own keeps `SIGNALLED`, which other tests' flushes read,
    /// clear.
    #[cfg(unix)]
    #[tokio::test]
    async fn a_signal_nobody_polls_for_still_marks_the_process_signalled() {
        static FLAG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        let Some(sig) = [libc::SIGHUP, libc::SIGTERM, libc::SIGINT]
            .into_iter()
            .find(|&s| !ignored(s))
        else {
            return;
        };
        let signals = Signals::watching(&FLAG);
        // SAFETY: raising a signal whose handler `Signals::watching` installed.
        _ = unsafe { libc::raise(sig) };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while !FLAG.load(std::sync::atomic::Ordering::SeqCst) {
            assert!(
                std::time::Instant::now() < deadline,
                "the watcher never saw the signal"
            );
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        drop(signals);
    }

    #[test]
    fn term_names_without_common_terminfo_become_xterm_256color() {
        assert_eq!(remote_term(Some("xterm-ghostty"), None), "xterm-256color");
        assert_eq!(remote_term(Some("xterm-kitty"), None), "xterm-256color");
        assert_eq!(remote_term(Some("tmux-256color"), None), "tmux-256color");
        assert_eq!(remote_term(None, None), "xterm-256color");
        assert_eq!(
            remote_term(Some("xterm-kitty"), Some("xterm-kitty")),
            "xterm-kitty"
        );
    }

    fn status(
        code: hedra_cli_sandbox::tonic::Code,
        reason: Option<&str>,
        meta: &[(&str, &str)],
    ) -> BrokerStatus {
        status_saying(code, reason, meta, "broker says")
    }

    fn status_saying(
        code: hedra_cli_sandbox::tonic::Code,
        reason: Option<&str>,
        meta: &[(&str, &str)],
        message: &str,
    ) -> BrokerStatus {
        BrokerStatus {
            code,
            reason: reason.map(str::to_string),
            message: message.into(),
            metadata: meta
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn each_broker_reason_maps_to_its_class() {
        use hedra_cli_sandbox::tonic::Code::*;
        let class =
            |c, r, started| broker_failure(&status(c, r, &[]), "t", Needs::Exec, started).class;
        assert_eq!(
            class(PermissionDenied, Some("permission_denied"), true),
            Class::MissingScope
        );
        assert_eq!(
            class(ResourceExhausted, Some("quota_exceeded"), false),
            Class::Limit
        );
        assert_eq!(
            class(Unavailable, Some("sandbox_disconnected"), false),
            Class::NotReady
        );
        assert_eq!(
            class(Unavailable, Some("sandbox_disconnected"), true),
            Class::ConnectionLost
        );
        assert_eq!(
            class(Unavailable, Some("rate_limited"), true),
            Class::ConnectionLost
        );
        assert_eq!(class(NotFound, Some("not_found"), true), Class::NotReady);
        assert_eq!(class(Internal, Some("internal"), true), Class::Internal);
        assert_eq!(
            class(InvalidArgument, Some("invalid_argument"), true),
            Class::Rejected
        );
        assert_eq!(class(Unavailable, None, false), Class::Unavailable);
        assert_eq!(class(Unauthenticated, None, false), Class::Unavailable);
        assert_eq!(class(DataLoss, None, false), Class::Internal);
        assert_eq!(class(Internal, None, true), Class::ConnectionLost);
        assert_eq!(class(NotFound, None, true), Class::ConnectionLost);
    }

    #[tokio::test]
    async fn an_unreached_broker_names_the_proxy_or_the_per_network_limit_at_the_end() {
        use hedra_cli_sandbox::tonic::Code::*;
        let api = Api::for_tests("http://127.0.0.1:9");
        let fail = |s: BrokerStatus, proxy: Option<&'static str>| {
            let api = &api;
            async move {
                open_failure(
                    api,
                    "sandbox_1",
                    "t",
                    0,
                    Needs::Exec,
                    proxy,
                    OpenError::Refused(s),
                )
                .await
            }
        };
        let f = fail(status(Unavailable, None, &[]), None).await;
        assert_eq!(f.class, Class::Unavailable);
        assert!(
            f.message.contains("per-network connection limit"),
            "{}",
            f.message
        );
        let f = fail(
            status(DeadlineExceeded, None, &[]),
            Some("http://u:secret@proxy.local:3128"),
        )
        .await;
        assert!(
            f.message
                .contains("could not be reached through the proxy http://***@proxy.local:3128"),
            "{}",
            f.message
        );
        assert!(!f.message.contains("secret"), "{}", f.message);
        // A refusal the broker explained is not an unreached broker.
        let f = fail(status(Unauthenticated, None, &[]), None).await;
        assert!(!f.message.contains("per-network"), "{}", f.message);
        // A second `unauthenticated` with its `ErrorInfo` is the broker's
        // key cache too: the same line, naming the broker's message.
        let f = fail(
            status_saying(
                Unauthenticated,
                Some("unauthenticated"),
                &[],
                "invalid session token",
            ),
            None,
        )
        .await;
        assert_eq!(f.class, Class::Unavailable);
        assert_eq!(
            f.message,
            "Sandbox sessions are temporarily unavailable (invalid session token); retry shortly."
        );
        let f = fail(status(Unavailable, Some("sandbox_disconnected"), &[]), None).await;
        assert!(!f.message.contains("per-network"), "{}", f.message);
    }

    #[test]
    fn a_permission_denied_names_the_sessions_scope_as_the_keys() {
        let f = broker_failure(
            &status_saying(
                hedra_cli_sandbox::tonic::Code::PermissionDenied,
                Some("permission_denied"),
                &[],
                "token lacks the sandbox:exec scope",
            ),
            "t",
            Needs::Read,
            false,
        );
        assert!(f.message.contains("`sandboxes:exec`"), "{}", f.message);
    }

    #[test]
    fn a_permission_denied_naming_no_scope_names_the_one_the_call_needs() {
        let f = broker_failure(
            &status_saying(
                hedra_cli_sandbox::tonic::Code::PermissionDenied,
                Some("permission_denied"),
                &[],
                "permission denied",
            ),
            "t",
            Needs::Read,
            false,
        );
        assert!(f.message.contains("`sandboxes:read`"), "{}", f.message);
    }

    /// An active sandbox with `instance` as its one instance, or with none
    /// for null.
    fn active_with(instance: serde_json::Value) -> hedra_cli_sdk::api::Sandbox {
        let instances = if instance.is_null() {
            serde_json::Value::Null
        } else {
            serde_json::json!([instance])
        };
        serde_json::from_value(serde_json::json!({
            "id": "sandbox_1", "name": "box", "gpu": "rtx4090:1", "gpu_model": "rtx4090",
            "replicas": 1, "status": "active", "created_at": "2026-11-03T12:00:00Z",
            "env_keys": [], "url": null,
            "instances": instances,
            "billing": {"usd_per_hour": 1.0, "paid_usd": 0.0, "settled": false},
        }))
        .unwrap()
    }

    /// Replica 0 as the platform reports it: `started` when it ran.
    fn replica(status: &str, failure: Option<&str>, started: bool) -> serde_json::Value {
        let mut i = serde_json::json!({"replica": 0, "status": status});
        if let Some(f) = failure {
            i["failure_reason"] = serde_json::json!(f);
        }
        if started {
            i["started_at"] = serde_json::json!("2026-11-03T12:01:00Z");
        }
        i
    }

    const OUTPUT: &str = "Run `hedra-cli sandbox logs sandbox_1` for its output.";
    const ANOTHER: &str = "Create another, or run `hedra-cli sandbox ls` for the running ones.";
    const CAPACITY: &str =
        "No `rtx4090:1` was free. Run `hedra-cli sandbox gpus`, then create another.";

    #[test]
    fn a_stopped_replica_on_an_active_sandbox_names_the_logs_command() {
        // A stop with no failure may be an exit, an idle stop or a TTL stop.
        let s = active_with(replica("stopped", None, true));
        assert_eq!(
            exited_line(&s, "box", 0).unwrap(),
            format!("`box`'s instance stopped. {OUTPUT}")
        );
        let s = active_with(replica("failed", Some("oom_killed"), true));
        assert_eq!(
            exited_line(&s, "box", 0).unwrap(),
            format!("`box`'s instance stopped (oom_killed). {OUTPUT}")
        );
        // Lost with its node after it ran: the output is there.
        let s = active_with(replica("failed", Some("platform_error"), true));
        assert_eq!(
            exited_line(&s, "box", 0).unwrap(),
            format!("`box`'s instance stopped (platform_error). {OUTPUT}")
        );
        let s = active_with(replica("failed", Some("evil\u{1b}]52;c;aGk=\u{7}"), true));
        let line = exited_line(&s, "box", 0).unwrap();
        assert!(!line.contains('\x1b') && !line.contains('\x07'), "{line:?}");
        // An instance that never started has no output to read.
        let s = active_with(replica("failed", Some("image_pull_failed"), false));
        assert_eq!(
            exited_line(&s, "box", 0).unwrap(),
            format!("`box`'s instance stopped (image_pull_failed). {ANOTHER}")
        );
        let s = active_with(replica("failed", Some("no_capacity"), false));
        assert_eq!(
            exited_line(&s, "box", 0).unwrap(),
            format!("`box`'s instance stopped (no_capacity). {CAPACITY}")
        );
        // A read that shows replica 0 running, starting, missing or in a
        // state hedra-cli does not know says nothing about its command.
        for instance in [
            serde_json::json!({"replica": 0, "status": "running"}),
            serde_json::json!({"replica": 0, "status": "starting"}),
            serde_json::json!({"replica": 0, "status": "paused"}),
            serde_json::json!({"replica": 1, "status": "stopped"}),
            serde_json::json!(null),
        ] {
            assert_eq!(
                exited_line(&active_with(instance.clone()), "box", 0),
                None,
                "{instance}"
            );
        }
    }

    #[test]
    fn every_state_names_a_next_step() {
        let read = |status: &str, end_reason: Option<&str>, instance: serde_json::Value| {
            let mut s = active_with(instance);
            s.status = serde_json::from_value(serde_json::json!(status)).unwrap();
            s.end_reason =
                end_reason.map(|r| serde_json::from_value(serde_json::json!(r)).unwrap());
            s
        };
        let ran = replica("stopped", None, true);
        let never = replica("stopped", None, false);
        let oom = replica("failed", Some("oom_killed"), true);
        let lost = replica("failed", Some("platform_error"), true);
        let unstaged = replica("failed", Some("platform_error"), false);
        let no_capacity = replica("failed", Some("no_capacity"), false);
        let check = "Retry once it is running; \
                     check on it with `hedra-cli sandbox get --sandbox sandbox_1`.";
        let get = "Check on it with `hedra-cli sandbox get --sandbox sandbox_1`.";
        for (s, want) in [
            // Replica 0 started: its output, whatever ended it.
            (read("ended", Some("exited"), ran.clone()), OUTPUT),
            (read("ended", Some("instance_failed"), oom.clone()), OUTPUT),
            (read("ended", Some("instance_failed"), lost), OUTPUT),
            (read("stopping", Some("caller_stopped"), oom), OUTPUT),
            (
                read("stopping", Some("caller_stopped"), ran.clone()),
                OUTPUT,
            ),
            (read("ended", Some("ttl_ended"), ran.clone()), OUTPUT),
            (read("ended", Some("idle_ttl_ended"), ran.clone()), OUTPUT),
            (read("ended", Some("lease_lapsed"), ran.clone()), OUTPUT),
            (read("ended", Some("access_removed"), ran.clone()), OUTPUT),
            (read("ended", None, ran.clone()), OUTPUT),
            // It never started: another, or the GPUs free for a full pool.
            (
                read("ended", Some("caller_stopped"), never.clone()),
                ANOTHER,
            ),
            (read("ended", Some("instance_failed"), unstaged), ANOTHER),
            (
                read("ended", Some("instance_failed"), no_capacity.clone()),
                CAPACITY,
            ),
            (
                read("ended", Some("caller_stopped"), serde_json::json!(null)),
                ANOTHER,
            ),
            (read("failed", None, serde_json::json!(null)), ANOTHER),
            (read("creating", None, serde_json::json!(null)), check),
            (read("active", None, ran.clone()), get),
        ] {
            assert_eq!(
                next_step(&s, 0),
                want,
                "{:?} {:?} {:?}",
                s.status,
                s.end_reason,
                s.instances
            );
        }
        // Replica 0 decides, not another replica's failure: one that ran
        // beside a replica 1 that failed to start still has output.
        let mut s = read("ended", Some("instance_failed"), ran);
        let failed_1 = serde_json::json!({"replica": 1, "status": "failed",
                                          "failure_reason": "start_failed"});
        s.instances
            .as_mut()
            .unwrap()
            .push(serde_json::from_value(failed_1).unwrap());
        assert_eq!(next_step(&s, 0), OUTPUT);
        // A replica 0 that never started beside a replica 1 the pool could
        // not place still names the GPUs.
        let mut s = read("ended", Some("instance_failed"), never.clone());
        let full_1 = serde_json::json!({"replica": 1, "status": "failed",
                                        "failure_reason": "no_capacity"});
        s.instances
            .as_mut()
            .unwrap()
            .push(serde_json::from_value(full_1).unwrap());
        assert_eq!(next_step(&s, 0), CAPACITY);
        // `logs` reads replica 0, so another replica having started gives
        // it no output.
        let mut s = read("ended", Some("caller_stopped"), never.clone());
        let ran_1 = serde_json::json!({"replica": 1, "status": "stopped",
                                       "started_at": "2026-11-03T12:01:00Z"});
        s.instances
            .as_mut()
            .unwrap()
            .push(serde_json::from_value(ran_1).unwrap());
        assert_eq!(next_step(&s, 0), ANOTHER);
        // Unpaid names funds, on a stopping and on an ended sandbox.
        for status in ["stopping", "ended"] {
            let unpaid = next_step(
                &read(status, Some("unpaid"), replica("stopped", None, true)),
                0,
            );
            assert!(unpaid.starts_with("Add funds"), "{status}: {unpaid}");
        }
        // A whole line, as `open_failure` builds it for a stopped sandbox:
        // the name as typed in the prose, the id in the command.
        let s = read("ended", Some("instance_failed"), no_capacity);
        assert_eq!(
            format!("{} {}", state_line(&s, "x", 0), next_step(&s, 0)),
            format!("`x` is ended (instance_failed (no_capacity)). {CAPACITY}")
        );
    }

    #[test]
    fn only_a_running_or_starting_replica_reads_as_running() {
        let stopped = replica("stopped", None, true);
        for instance in [
            serde_json::json!({"replica": 0, "status": "running"}),
            serde_json::json!({"replica": 0, "status": "starting"}),
            serde_json::json!(null),
        ] {
            assert_eq!(not_running(&active_with(instance), "box", 0), None);
        }
        let f = not_running(&active_with(stopped.clone()), "box", 0).unwrap();
        assert_eq!(f.class, Class::NotReady);
        assert_eq!(f.message, format!("`box`'s instance stopped. {OUTPUT}"));
        // A sandbox still being created, or in a status this build does not
        // know, may yet run.
        for status in ["creating", "paused"] {
            let mut s = active_with(stopped.clone());
            s.status = serde_json::from_value(serde_json::json!(status)).unwrap();
            assert_eq!(not_running(&s, "box", 0), None, "{status}");
        }
        // An active sandbox whose lease was refused still runs.
        let mut unpaid = active_with(serde_json::json!({"replica": 0, "status": "running"}));
        unpaid.end_reason = Some(hedra_cli_sdk::api::SandboxEndReason::Unpaid);
        assert_eq!(not_running(&unpaid, "box", 0), None);
        let mut stopped_unpaid = unpaid.clone();
        stopped_unpaid.status = SandboxStatus::Stopping;
        assert!(not_running(&stopped_unpaid, "box", 0)
            .unwrap()
            .message
            .starts_with("`box` is stopping (unpaid). Add funds"));
        let mut ended = active_with(stopped);
        ended.status = SandboxStatus::Ended;
        ended.end_reason = Some(hedra_cli_sdk::api::SandboxEndReason::CallerStopped);
        assert_eq!(
            not_running(&ended, "box", 0).unwrap().message,
            format!("`box` is ended (caller_stopped). {OUTPUT}")
        );
        let mut failed = active_with(serde_json::json!(null));
        failed.status = SandboxStatus::Failed;
        assert_eq!(
            not_running(&failed, "box", 0).unwrap().message,
            format!("`box` is failed. {ANOTHER}")
        );
    }

    /// A two-replica active sandbox whose instances are `instances`.
    fn two_replicas(instances: serde_json::Value) -> hedra_cli_sdk::api::Sandbox {
        let mut s = active_with(serde_json::json!(null));
        s.replicas = 2;
        s.instances = serde_json::from_value(instances).unwrap();
        s
    }

    #[test]
    fn a_replica_the_sandbox_does_not_have_is_refused_naming_how_many_it_has() {
        let s = two_replicas(serde_json::json!([
            {"replica": 0, "status": "running"},
            {"replica": 1, "status": "running"},
        ]));
        assert_eq!(missing_replica(&s, "box", 0), None);
        assert_eq!(missing_replica(&s, "box", 1), None);
        let f = missing_replica(&s, "box", 2).unwrap();
        assert_eq!(f.class, Class::Rejected);
        assert_eq!(
            f.message,
            "`box` has no replica 2: it has 2 replicas, 0 to 1."
        );
        let one = active_with(serde_json::json!({"replica": 0, "status": "running"}));
        assert_eq!(
            missing_replica(&one, "box", 1).unwrap().message,
            "`box` has no replica 1: it has 1 replica, replica 0."
        );
        // Under `replicas` but not listed, or listing none yet: it may still
        // be placed, so it is not refused here.
        let placing = two_replicas(serde_json::json!([{"replica": 0, "status": "running"}]));
        assert_eq!(missing_replica(&placing, "box", 1), None);
        assert_eq!(
            missing_replica(&two_replicas(serde_json::json!(null)), "box", 1),
            None
        );
        assert!(missing_replica(&placing, "box", 2).is_some());
        // An instance started to replace a lost one is listed beside it, past
        // `replicas`, and is reachable as the session route numbers it.
        let replaced = two_replicas(serde_json::json!([
            {"replica": 0, "status": "running"},
            {"replica": 1, "status": "failed", "failure_reason": "platform_error"},
            {"replica": 2, "status": "running"},
        ]));
        assert_eq!(missing_replica(&replaced, "box", 2), None);
        assert_eq!(
            missing_replica(&replaced, "box", 3).unwrap().message,
            "`box` has no replica 3: it has 2 replicas and lists 3 instances, 0 to 2."
        );
        // A read with no `replicas` still has replica 0.
        let mut none = one.clone();
        none.replicas = 0;
        assert_eq!(
            missing_replica(&none, "box", 1).unwrap().message,
            "`box` has no replica 1: it has 1 replica, replica 0."
        );
        // Replica 0 is never refused, whatever the read lists.
        assert_eq!(
            missing_replica(
                &two_replicas(serde_json::json!([{"replica": 1, "status": "running"}])),
                "box",
                0
            ),
            None
        );
    }

    #[test]
    fn a_sandbox_is_judged_by_the_replica_asked_for() {
        let ran_1 = serde_json::json!({"replica": 1, "status": "stopped",
                                       "started_at": "2026-11-03T12:01:00Z"});
        let s = two_replicas(serde_json::json!([
            {"replica": 0, "status": "running"},
            ran_1,
        ]));
        // Replica 0 runs, so a session to it is not refused...
        assert_eq!(not_running(&s, "box", 0), None);
        // ...while replica 1 stopped, and its own output is named.
        let f = not_running(&s, "box", 1).unwrap();
        assert_eq!(f.class, Class::NotReady);
        assert_eq!(
            f.message,
            "Replica 1 of `box` stopped. \
             Run `hedra-cli sandbox logs sandbox_1 --replica 1` for its output."
        );
        assert_eq!(state_line(&s, "box", 1), "Replica 1 of `box` stopped.");
        assert_eq!(state_line(&s, "box", 0), "`box` is active.");
        assert_eq!(
            instance_stopped(&s, 1).as_deref(),
            Some("The sandbox's replica 1 stopped.")
        );
        assert_eq!(instance_stopped(&s, 0), None);
        // The other way round: replica 0 stopped, replica 1 runs.
        let s = two_replicas(serde_json::json!([
            replica("failed", Some("oom_killed"), true),
            {"replica": 1, "status": "running"},
        ]));
        assert_eq!(not_running(&s, "box", 1), None);
        assert_eq!(
            not_running(&s, "box", 0).unwrap().message,
            format!("`box`'s instance stopped (oom_killed). {OUTPUT}")
        );
        // A replica past 0 that never ran, beside one still running, names
        // `get`; once the sandbox has ended, another.
        let mut s = two_replicas(serde_json::json!([
            {"replica": 0, "status": "running"},
            {"replica": 1, "status": "failed", "failure_reason": "image_pull_failed"},
        ]));
        let get = "Check on it with `hedra-cli sandbox get --sandbox sandbox_1`.";
        assert_eq!(
            not_running(&s, "box", 1).unwrap().message,
            format!("Replica 1 of `box` stopped (image_pull_failed). {get}")
        );
        s.status = SandboxStatus::Ended;
        assert_eq!(ended_step(&s, 1), ANOTHER);
    }

    #[test]
    fn server_text_in_a_state_line_and_a_scope_is_escaped() {
        let s: hedra_cli_sdk::api::Sandbox = serde_json::from_value(serde_json::json!({
            "id": "sandbox_1", "name": "box", "gpu": "h100:1", "gpu_model": "h100",
            "replicas": 1, "status": "active", "created_at": "2026-11-03T12:00:00Z",
            "env_keys": [], "url": null,
            "instances": [{"replica": 0, "status": "failed",
                           "failure_reason": "evil\u{1b}]52;c;aGk=\u{7}"}],
            "billing": {"usd_per_hour": 2.5, "paid_usd": 0.0, "settled": false},
        }))
        .unwrap();
        let line = state_line(&s, "box", 0);
        assert!(!line.contains('\x1b') && !line.contains('\x07'), "{line:?}");
        let f = broker_failure(
            &status_saying(
                hedra_cli_sandbox::tonic::Code::PermissionDenied,
                Some("permission_denied"),
                &[],
                "token lacks the sandbox:exec\x1b]52;c;aGk=\x07 scope",
            ),
            "t",
            Needs::Exec,
            false,
        );
        assert!(!f.message.contains('\x1b'), "{:?}", f.message);
    }
}
