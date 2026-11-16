//! The sandbox stream verbs: `exec`, `shell` (and `shell --gpu`), `logs`,
//! `attach`, `cp` and `port-forward`, which hold a stream open to the sandbox
//! through the broker. Each is grafted into the generated `sandbox` group
//! beside the generated control commands (`ls`, `get`, `create`, `stop`, …),
//! which this module does not touch.

mod api;
mod cp;
mod error;
mod exec;
mod forward;
mod gpu_shell;
mod lifecycle;
mod logs;
mod names;
mod render;
mod stream;

use std::collections::HashMap;
use std::io::Write;

use clap::{Arg, ArgAction, ArgMatches, Command};
use fern_cli_sdk::error::CliError;
use fern_cli_sdk::openapi::AppContext;

use api::{attempt, Api};
use error::{Class, Failure};
use names::Target;

/// The generated group the verbs are grafted into.
pub(crate) const GROUP: &str = "sandbox";

const EXIT_CODES: &str = "\
Exit codes:
   0  success (exec, shell and cp: the command's own exit code)
   1  Hedra returned an error
   2  not signed in
   3  usage error
  10  the key lacks a sandbox scope
  11  no such sandbox
  12  sandboxes are not enabled for this workspace
  13  the sandbox is not ready, or stopped
  14  no GPU of that model is free
  15  the wallet cannot pay
  16  a limit was reached
  17  Hedra or the network is unavailable; retry
  18  the connection was lost
  19  the request was refused
 129  hangup
 130  interrupted (logs -f, attach and port-forward end on Ctrl-C and exit 0)
 141  the output was closed
 143  terminated
 255  the output failed, or hedra-cli panicked; exec, shell, cp: any failure of hedra-cli's own, not the command's";

/// The stream verbs' commands, each grafted under `sandbox` with its handler.
pub(crate) fn commands() -> Vec<(Command, fern_cli_sdk::app::CliCommandHandler)> {
    vec![
        (exec_command(), Box::new(|m, ctx| dispatch("exec", m, ctx))),
        (
            shell_command(),
            Box::new(|m, ctx| dispatch("shell", m, ctx)),
        ),
        (logs_command(), Box::new(|m, ctx| dispatch("logs", m, ctx))),
        (
            attach_command(),
            Box::new(|m, ctx| dispatch("attach", m, ctx)),
        ),
        (cp_command(), Box::new(|m, ctx| dispatch("cp", m, ctx))),
        (
            port_forward_command(),
            Box::new(|m, ctx| dispatch("port-forward", m, ctx)),
        ),
    ]
}

fn sandbox_arg() -> Arg {
    Arg::new("sandbox")
        .value_name("SANDBOX")
        .required(true)
        .help("The sandbox's name, or its `sandbox_` id")
}

/// `--replica N`: which instance of a sandbox with several replicas a verb
/// reaches. Every verb reached replica 0 before the flag, and still does.
fn replica_arg() -> Arg {
    Arg::new("replica")
        .long("replica")
        .value_name("N")
        .value_parser(clap::value_parser!(i64).range(0..))
        .default_value("0")
        .help(
            "The replica to reach, from 0; `hedra-cli sandbox get --sandbox <SANDBOX>` lists them",
        )
}

/// `--replica`'s value: 0 when the command has none.
fn replica(m: &ArgMatches) -> i64 {
    m.try_get_one::<i64>("replica")
        .ok()
        .flatten()
        .copied()
        .unwrap_or(0)
}

fn term_arg() -> Arg {
    Arg::new("term")
        .long("term")
        .value_name("TERM")
        .help("The TERM to give the remote terminal; by default yours, or xterm-256color when the image may not know it")
}

fn exec_command() -> Command {
    Command::new("exec")
        .about("Run a command in a sandbox, relaying its stdin, stdout, stderr and exit code")
        .after_long_help(EXIT_CODES)
        .arg(sandbox_arg())
        .arg(
            Arg::new("tty")
                .short('t')
                .long("tty")
                .action(ArgAction::SetTrue)
                .help("Run the command on a terminal, with this terminal in raw mode"),
        )
        .arg(
            Arg::new("no-stdin")
                .short('n')
                .long("no-stdin")
                .action(ArgAction::SetTrue)
                .conflicts_with("tty")
                .help("Send no input: the command sees end of input at once"),
        )
        .arg(
            Arg::new("env")
                .long("env")
                .value_name("KEY=VALUE")
                .action(ArgAction::Append)
                .help("An environment variable for the command; repeatable"),
        )
        .arg(term_arg())
        .arg(replica_arg())
        .arg(
            Arg::new("command")
                .value_name("COMMAND")
                .num_args(1..)
                .last(true)
                .required(true)
                .help("The command and its arguments, after --; run without a shell"),
        )
}

fn shell_command() -> Command {
    Command::new("shell")
        .about("Open an interactive shell in a sandbox, or with --gpu in a new one that stops when the shell ends")
        .after_long_help(EXIT_CODES)
        .arg(sandbox_arg().required(false).required_unless_present("gpu"))
        .arg(
            Arg::new("gpu")
                .long("gpu")
                .value_name("MODEL:COUNT")
                .conflicts_with("sandbox")
                .requires("image")
                .help("Create a sandbox with this GPU model and count, such as h100:1, for this shell"),
        )
        .arg(
            Arg::new("image")
                .long("image")
                .value_name("REF")
                .requires("gpu")
                .help("With --gpu: the container image to run"),
        )
        .arg(
            Arg::new("max-duration")
                .long("max-duration")
                .value_name("DURATION")
                .requires("gpu")
                .help("With --gpu: stop the sandbox this long after it is created, such as 8h; 24h by default"),
        )
        .arg(
            Arg::new("replicas")
                .long("replicas")
                .value_name("N")
                .requires("gpu")
                .value_parser(clap::value_parser!(i64).range(1..=MAX_REPLICAS))
                .help("With --gpu: how many instances to create, at most your workspace's sandbox quota; 1 by default. Every one stops with the shell"),
        )
        .arg(term_arg())
        .arg(replica_arg().help(
            "The replica to reach, from 0; with --gpu, below --replicas, else \
             `hedra-cli sandbox get --sandbox <SANDBOX>` lists them",
        ))
}

/// The most replicas the create route takes.
const MAX_REPLICAS: i64 = 64;

fn logs_command() -> Command {
    Command::new("logs")
        .about("Print a sandbox's main process output; -f keeps following it")
        .after_long_help(EXIT_CODES)
        .arg(sandbox_arg())
        .arg(
            Arg::new("follow")
                .short('f')
                .long("follow")
                .action(ArgAction::SetTrue)
                .help("Keep printing new output until Ctrl-C, through the sandbox's stop"),
        )
        .arg(replica_arg())
}

fn attach_command() -> Command {
    Command::new("attach")
        .about("Follow a sandbox's main process output; the same as logs -f")
        .after_long_help(EXIT_CODES)
        .arg(sandbox_arg())
        .arg(replica_arg())
}

fn cp_command() -> Command {
    const EXAMPLES: &str = "Examples:\n  hedra-cli sandbox cp ./data train:/work/data\n  \
                            hedra-cli sandbox cp train:/work/out ./out";
    Command::new("cp")
        .about("Copy files and directories in or out of a sandbox; one side is <SANDBOX>:<PATH>")
        .long_about(
            "Copy a file or a directory into or out of a sandbox. One side is <SANDBOX>:<PATH>, \
             a path in the sandbox, where a relative path is relative to the image's working \
             directory; the other is a local path. Directories copy without a flag.\n\n\
             As scp -r and docker cp do, an existing directory receives the source under the \
             source's own name; otherwise the destination is the new name, and a missing \
             parent is created.\n\n\
             A source path that ends in /. copies the directory's contents, and a destination \
             that ends in / has to be an existing directory. A symlinked source copies what it \
             links to.\n\n\
             The copy runs over exec and tar, so the image needs sh and tar; GNU tar also lets \
             hedra-cli check that a copy out arrived whole. A copy out that fails part way \
             leaves what it wrote in place and names it.",
        )
        .after_help(EXAMPLES)
        .after_long_help(format!("{EXAMPLES}\n\n{EXIT_CODES}"))
        .arg(
            Arg::new("source")
                .value_name("SOURCE")
                .required(true)
                .value_parser(clap::value_parser!(std::ffi::OsString))
                .help("A local path, or <SANDBOX>:<PATH>"),
        )
        .arg(
            Arg::new("destination")
                .value_name("DESTINATION")
                .required(true)
                .value_parser(clap::value_parser!(std::ffi::OsString))
                .help("A local path, or <SANDBOX>:<PATH>"),
        )
        .arg(replica_arg())
}

fn port_forward_command() -> Command {
    Command::new("port-forward")
        .about("Forward a local port to a port in a sandbox until Ctrl-C")
        .after_long_help(EXIT_CODES)
        .arg(sandbox_arg())
        .arg(
            Arg::new("ports")
                .value_name("LOCAL:REMOTE")
                .required(true)
                .help(
                    "The local and sandbox ports, or one port for both; local 0 picks a free one",
                ),
        )
        .arg(replica_arg())
}

/// A verb's handler: the verb, then its exit code.
/// Every verb exits from here, since its stdout is the sandbox's output and
/// its errors go to stderr.
fn dispatch(verb: &str, m: &ArgMatches, ctx: &dyn std::any::Any) -> Result<(), CliError> {
    let ctx = ctx
        .downcast_ref::<AppContext>()
        .ok_or_else(|| CliError::Validation("sandbox: unexpected binding context type".into()))?;
    // `std::env::args` panics on an argument that is not valid Unicode, such
    // as a local `cp` path, which clap passes through as bytes.
    let args: Vec<String> = std::env::args_os()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let json_errors = fern_cli_sdk::formatter::explicit_machine_format(&args);
    let flag = |name: &str| {
        m.try_get_one::<bool>(name)
            .ok()
            .flatten()
            .copied()
            .unwrap_or(false)
    };
    stream::start_lines();
    let fail = |f: Failure, code: i32| -> ! {
        f.write_stderr(json_errors);
        let _ = std::io::stdout().flush();
        stream::flush_lines();
        std::process::exit(code);
    };
    if flag("dry-run") {
        if let Err(f) = dry_run(verb, m) {
            let code = refusal_code(verb, &f);
            fail(f, code);
        }
        stream::flush_lines();
        std::process::exit(0);
    }
    // The runtime's client panics on settings it cannot build, printing a
    // proxy URL with its credentials; refuse them first, naming no value.
    if ctx.http_config().build_client().is_err() {
        let f = Failure::rejected(unusable_http_settings());
        let code = refusal_code(verb, &f);
        fail(f, code);
    }
    let api = std::sync::Arc::new(Api::new(ctx, flag("debug")));
    let code = match verb {
        "cp" => {
            let arg = |n: &str| m.get_one::<std::ffi::OsString>(n).expect("required");
            match stream::transport_config(ctx.http_config()) {
                Ok(transport) => unwinding(json_errors, || {
                    run_async(cp::run(
                        &api,
                        transport,
                        arg("source"),
                        arg("destination"),
                        replica(m),
                        json_errors,
                    ))
                }),
                Err(f) => fail(f, 255),
            }
        }
        "port-forward" => {
            let api = api.clone();
            let ports = m.get_one::<String>("ports").expect("required").clone();
            match target(m).and_then(|t| {
                stream::transport_config(ctx.http_config()).map(|transport| (t, transport))
            }) {
                Ok((t, transport)) => unwinding(json_errors, || {
                    run_async(forward::run(
                        api,
                        transport,
                        &t,
                        &ports,
                        replica(m),
                        json_errors,
                    ))
                }),
                Err(f) => {
                    let code = f.class.code();
                    fail(f, code);
                }
            }
        }
        "logs" | "attach" => {
            let follow = verb == "attach" || m.get_flag("follow");
            match target(m).and_then(|t| {
                stream::transport_config(ctx.http_config()).map(|transport| (t, transport))
            }) {
                Ok((t, transport)) => unwinding(json_errors, || {
                    run_async(logs::run(
                        &api,
                        transport,
                        &t,
                        follow,
                        replica(m),
                        json_errors,
                    ))
                }),
                Err(f) => {
                    let code = f.class.code();
                    fail(f, code);
                }
            }
        }
        "shell" if m.get_one::<String>("gpu").is_some() => match gpu_shell_args(m, ctx) {
            Ok((shell, transport)) => unwinding(json_errors, || {
                run_async(gpu_shell::run(&api, transport, shell, json_errors))
            }),
            Err(f) => fail(f, 255),
        },
        _ => match stream_verb(verb, m, ctx) {
            Ok((target, exec, transport)) => unwinding(json_errors, || {
                run_async(exec::run(&api, transport, &target, exec, json_errors))
            }),
            Err(f) => fail(f, 255),
        },
    };
    let _ = std::io::stdout().flush();
    stream::flush_lines();
    std::process::exit(code);
}

/// The exit code for a refusal before anything is sent: 255 on `exec`,
/// `shell` and `cp`, which keep 255 for their own failures, else the class's.
fn refusal_code(verb: &str, f: &Failure) -> i32 {
    match verb {
        "exec" | "shell" | "cp" => 255,
        _ => f.class.code(),
    }
}

/// The variables an unusable HTTP client may have been built from.
const HTTP_SETTINGS: [&str; 8] = [
    "HEDRA_CLI_PROXY",
    "HTTPS_PROXY",
    "https_proxy",
    "ALL_PROXY",
    "all_proxy",
    "HEDRA_CLI_CA_BUNDLE",
    "HEDRA_CLI_EXTRA_CA_CERTS",
    "SSL_CERT_FILE",
];

/// The refusal for HTTP settings the client cannot be built from. It names
/// the variables, never their values, which can hold credentials.
fn unusable_http_settings() -> String {
    let set: Vec<&str> = HTTP_SETTINGS
        .into_iter()
        .filter(|k| std::env::var_os(k).is_some_and(|v| !v.is_empty()))
        .collect();
    if set.is_empty() {
        "hedra-cli's HTTP settings cannot be used.".to_string()
    } else {
        format!(
            "hedra-cli's HTTP settings cannot be used; check {}.",
            set.join(", ")
        )
    }
}

/// `--dry-run`: checks the arguments and sends nothing.
fn dry_run(verb: &str, m: &ArgMatches) -> Result<(), Failure> {
    if m.try_get_one::<String>("sandbox").ok().flatten().is_some() {
        target(m)?;
    }
    if verb == "shell" {
        if let Some(gpu) = m.get_one::<String>("gpu") {
            check_gpu(gpu)?;
            if let Some(raw) = m.get_one::<String>("max-duration") {
                render::parse_duration(raw).map_err(Failure::rejected)?;
            }
            check_shell_replica(m)?;
        }
    }
    if verb == "exec" {
        exec_env(m)?;
    }
    if verb == "cp" {
        let arg = |n: &str| m.get_one::<std::ffi::OsString>(n).expect("required");
        cp::plan(arg("source"), arg("destination"))?;
    }
    if verb == "port-forward" {
        forward::parse_ports(m.get_one::<String>("ports").expect("required"))?;
    }
    eprintln!("Sent no request (--dry-run).");
    Ok(())
}

/// `shell --gpu`'s arguments, checked before any request.
fn gpu_shell_args(
    m: &ArgMatches,
    ctx: &AppContext,
) -> Result<
    (
        gpu_shell::GpuShell,
        hedra_cli_sandbox::stream::transport::TransportConfig,
    ),
    Failure,
> {
    let gpu = m.get_one::<String>("gpu").expect("checked").clone();
    check_gpu(&gpu)?;
    let ttl_seconds = match m.get_one::<String>("max-duration") {
        Some(raw) => render::parse_duration(raw).map_err(Failure::rejected)?,
        None => gpu_shell::DEFAULT_MAX_DURATION,
    };
    let shell = gpu_shell::GpuShell {
        gpu,
        image: m
            .get_one::<String>("image")
            .expect("required by --gpu")
            .clone(),
        ttl_seconds,
        term: m.get_one::<String>("term").cloned(),
        replicas: m.get_one::<i64>("replicas").copied(),
        replica: check_shell_replica(m)?,
    };
    let transport = stream::transport_config(ctx.http_config())?;
    // With a bypass list the other verbs refuse an unusable proxy once the
    // session names the broker; the create comes first here and is paid
    // for, so the proxy is checked before it. A bypass list that covers the
    // broker the session names in practice lets it through.
    if !hedra_cli_sandbox::stream::transport::bypasses(transport.no_proxy.as_deref(), BROKER_HOST) {
        hedra_cli_sandbox::stream::transport::check_proxy(&transport)
            .map_err(|e| Failure::rejected(error::escape_controls(&e.to_string())))?;
    }
    Ok((shell, transport))
}

/// `shell --gpu`'s `--replica`, refused before the create unless it is below
/// `--replicas`, so no sandbox is paid for that the shell cannot open in.
fn check_shell_replica(m: &ArgMatches) -> Result<i64, Failure> {
    let replicas = m.get_one::<i64>("replicas").copied().unwrap_or(1);
    let replica = replica(m);
    if replica >= replicas {
        return Err(Failure::rejected(format!(
            "--replica {replica} is not among the {replicas} replica{} --gpu creates; \
             they are numbered from 0, so raise --replicas or lower --replica.",
            if replicas == 1 { "" } else { "s" }
        )));
    }
    Ok(replica)
}

/// The broker host the session route names, so `shell --gpu` can tell before
/// its create whether a bypass list covers it. The route names its broker
/// only in a session, which needs the sandbox the create makes, so the
/// production host stands in. Against another API, a bypass list that covers
/// only that API's broker can see an unusable proxy refused before the create
/// that the session would have bypassed: the check errs before the payment,
/// not after.
const BROKER_HOST: &str = "broker.sandbox.hedra.com";

/// `exec`'s `--env` pairs.
fn exec_env(m: &ArgMatches) -> Result<HashMap<String, String>, Failure> {
    let mut env = HashMap::new();
    for pair in m.get_many::<String>("env").into_iter().flatten() {
        match pair.split_once('=') {
            Some((k, v)) if !k.is_empty() => {
                env.insert(k.to_string(), v.to_string());
            }
            _ => {
                return Err(Failure::rejected(format!(
                    "--env takes KEY=VALUE, with a key; got `{}`.",
                    pair.split('=').next().unwrap_or_default()
                )))
            }
        }
    }
    Ok(env)
}

/// `exec` and `shell`'s arguments, checked before any request.
fn stream_verb(
    verb: &str,
    m: &ArgMatches,
    ctx: &AppContext,
) -> Result<
    (
        Target,
        exec::Exec,
        hedra_cli_sandbox::stream::transport::TransportConfig,
    ),
    Failure,
> {
    let target = target(m)?;
    let term = m.get_one::<String>("term").cloned();
    let exec = if verb == "shell" {
        exec::Exec {
            argv: exec::shell_argv(),
            tty: true,
            shell: true,
            no_stdin: false,
            env: HashMap::new(),
            term,
            replica: replica(m),
        }
    } else {
        exec::Exec {
            argv: m
                .get_many::<String>("command")
                .expect("required")
                .cloned()
                .collect(),
            tty: m.get_flag("tty"),
            shell: false,
            no_stdin: m.get_flag("no-stdin"),
            env: exec_env(m)?,
            term,
            replica: replica(m),
        }
    };
    let transport = stream::transport_config(ctx.http_config())?;
    Ok((target, exec, transport))
}

/// Runs a stream verb, turning a panic into exit 255 with its message
/// printed after the terminal is restored. The default hook would print it
/// while the terminal is raw, where each line starts where the last ended.
fn unwinding(json_errors: bool, verb: impl FnOnce() -> i32) -> i32 {
    use std::sync::{Arc, Mutex};
    let message: Arc<Mutex<Option<String>>> = Arc::default();
    let stored = message.clone();
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if let Ok(mut m) = stored.lock() {
            m.get_or_insert_with(|| info.to_string());
        }
    }));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(verb));
    std::panic::set_hook(previous);
    match outcome {
        Ok(code) => code,
        Err(_) => {
            // The raw-mode guard dropped while unwinding, so the terminal is
            // cooked again here.
            let text = message
                .lock()
                .ok()
                .and_then(|m| m.clone())
                .unwrap_or_else(|| "hedra-cli panicked".to_string());
            let mut f = Failure::new(Class::Panic, error::escape_controls(&text));
            if let Some((name, id)) = gpu_shell::created() {
                f = f.then(format!(
                    "`{name}` may still be running; stop it with `{}`.",
                    render::stop_command(&id)
                ));
            }
            f.write_stderr(json_errors);
            f.class.code()
        }
    }
}

fn run_async<T>(fut: impl std::future::Future<Output = T>) -> T {
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
}

fn target(m: &ArgMatches) -> Result<Target, Failure> {
    Target::parse(m.get_one::<String>("sandbox").expect("required"))
}

/// `<gpu model>:<count>`, as the create route's `gpu` takes it:
/// `^[a-z0-9][a-z0-9-]*:[1-9][0-9]?$`.
fn gpu_ok(s: &str) -> bool {
    let Some((model, count)) = s.split_once(':') else {
        return false;
    };
    let model_ok = model
        .bytes()
        .next()
        .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && model
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    let count_ok = (1..=2).contains(&count.len())
        && count.bytes().all(|b| b.is_ascii_digit())
        && !count.starts_with('0');
    model_ok && count_ok
}

fn check_gpu(gpu: &str) -> Result<(), Failure> {
    if gpu_ok(gpu) {
        return Ok(());
    }
    Err(Failure::rejected(format!(
        "`{gpu}` is not a GPU model and count such as h100:1. Run `{}`.",
        render::GPUS_COMMAND
    )))
}

/// How long each read before `shell --gpu`'s create may take.
const PRE_CREATE_READ: std::time::Duration = std::time::Duration::from_secs(5);

/// The warnings from the two reads before `shell --gpu`'s create: on any
/// failure or after `read`, a read's line is skipped and the create goes
/// ahead. The quota counts every one of the `replicas` instances' GPUs.
async fn warnings_before_create(
    api: &Api,
    gpu: &str,
    replicas: i64,
    read: std::time::Duration,
) -> Vec<String> {
    let (model, count) = gpu.split_once(':').unwrap_or((gpu, "1"));
    let count: i64 = count
        .parse::<i64>()
        .unwrap_or(1)
        .saturating_mul(replicas.max(1));
    let (gpus, quota) = tokio::join!(
        attempt(read, api.client.gpus(None)),
        attempt(read, api.client.quota(None)),
    );
    let mut lines = Vec::new();
    if let Ok(gpus) = gpus {
        if gpus
            .data
            .iter()
            .any(|g| g.gpu_model == model && g.available == Some(false))
        {
            lines.push(format!(
                "No {model} is free right now, so the sandbox may wait up to 15 minutes for one."
            ));
        }
    }
    if let Ok(quota) = quota {
        if count > quota.gpu_quota_remaining {
            lines.push(format!(
                "Your workspace has {} GPU{} left in its quota, so the create may be refused.",
                quota.gpu_quota_remaining,
                if quota.gpu_quota_remaining == 1 {
                    ""
                } else {
                    "s"
                }
            ));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The verbs this module grafts into the generated group, in
    /// [`commands`]'s order.
    const VERBS: [&str; 6] = ["exec", "shell", "logs", "attach", "cp", "port-forward"];

    /// `SSL_CERT_FILE` feeds the trust store the client is built with, so a
    /// bad one is among the settings the refusal names.
    #[test]
    fn the_http_settings_named_include_ssl_cert_file() {
        assert!(HTTP_SETTINGS.contains(&"SSL_CERT_FILE"));
    }

    /// The generated command tree, built from the embedded spec as the
    /// runtime builds it.
    fn generated_tree() -> Command {
        let doc =
            fern_cli_sdk::openapi::load_openapi_spec(include_str!("../openapi0.json"), "hedra-cli")
                .expect("the embedded spec");
        fern_cli_sdk::openapi::commands::build_cli(&doc)
    }

    /// A graft replaces a generated subcommand of the same name, a group as
    /// well as a leaf, without a warning, so a regeneration that adds one
    /// named like a stream verb fails here instead of losing it.
    #[test]
    fn no_generated_sandbox_command_shares_a_stream_verbs_name() {
        let cli = generated_tree();
        let group = cli
            .find_subcommand(GROUP)
            .expect("the generated sandbox group");
        assert!(group.find_subcommand("get").is_some());
        for verb in VERBS {
            assert!(
                group.find_subcommand(verb).is_none(),
                "`sandbox {verb}` is generated too"
            );
        }
        let grafted: Vec<String> = commands()
            .iter()
            .map(|(c, _)| c.get_name().to_string())
            .collect();
        assert_eq!(grafted, VERBS);
    }

    /// Each generated command a recovery hint names parses in the generated
    /// tree, so a regeneration that renames it or its flag fails here.
    #[test]
    fn every_hinted_command_parses_in_the_generated_tree() {
        let cli = generated_tree();
        for hint in render::hinted_commands() {
            let args: Vec<&str> = hint.split_whitespace().collect();
            assert!(
                cli.clone().try_get_matches_from(&args).is_ok(),
                "`{hint}` does not parse"
            );
        }
    }

    /// The code the help documents for `class`. The match has no wildcard,
    /// so a new class does not compile until it is given a code here, and
    /// added to `Class::ALL`.
    fn documented_code(class: Class) -> i32 {
        match class {
            Class::Internal => 1,
            Class::NotSignedIn => 2,
            Class::MissingScope => 10,
            Class::NotFound => 11,
            Class::NotEnabled => 12,
            Class::NotReady => 13,
            Class::NoCapacity => 14,
            Class::Payment => 15,
            Class::Limit => 16,
            Class::Unavailable => 17,
            Class::ConnectionLost => 18,
            Class::Rejected => 19,
            Class::Hangup => 129,
            Class::Interrupted => 130,
            Class::Terminated => 143,
            Class::OutputFailed | Class::Panic => 255,
        }
    }

    /// Every class's code, the public contract each verb's `--help` prints,
    /// and that the help table lists each one.
    #[test]
    fn each_class_exits_with_its_code_and_the_help_table_lists_it() {
        for class in Class::ALL {
            let code = documented_code(class);
            assert_eq!(class.code(), code, "{class:?}");
            let listed = EXIT_CODES
                .lines()
                .any(|l| l.split_whitespace().next() == Some(&code.to_string()));
            assert!(
                listed,
                "exit code {code} ({class:?}) is not in the help table"
            );
        }
    }

    #[test]
    fn gpus_follow_the_create_routes_pattern() {
        for ok in ["h100:1", "h100:8", "h100:4", "h100:99", "a-1:2"] {
            assert!(gpu_ok(ok), "{ok}");
        }
        for bad in [
            "h100", "h100:0", "h100:100", "H100:1", "-h:1", ":1", "h100:01", "h100:x",
        ] {
            assert!(!gpu_ok(bad), "{bad}");
        }
    }

    #[test]
    fn shell_takes_a_sandbox_or_gpu_and_image() {
        let parse = |args: &[&str]| {
            shell_command()
                .try_get_matches_from(std::iter::once("shell").chain(args.iter().copied()))
        };
        assert!(parse(&["train"]).is_ok());
        assert!(parse(&["--gpu", "h100:1", "--image", "i"]).is_ok());
        assert!(parse(&[]).is_err());
        assert!(parse(&["--gpu", "h100:1"]).is_err());
        assert!(parse(&["train", "--gpu", "h100:1", "--image", "i"]).is_err());
        assert!(parse(&["train", "--max-duration", "8h"]).is_err());
    }

    /// Every stream verb takes `--replica N`, replica 0 by default, and says
    /// so in its help; a negative replica is a usage error.
    #[test]
    fn every_stream_verb_takes_a_replica_defaulting_to_zero() {
        let args: [(&str, &[&str]); 6] = [
            ("exec", &["train", "--", "true"]),
            ("shell", &["train"]),
            ("logs", &["train"]),
            ("attach", &["train"]),
            ("cp", &["./a", "train:/b"]),
            ("port-forward", &["train", "8080"]),
        ];
        for (command, (verb, rest)) in commands().into_iter().map(|(c, _)| c).zip(args) {
            assert_eq!(command.get_name(), verb);
            let parse = |extra: &[&str]| {
                command.clone().try_get_matches_from(
                    std::iter::once(verb)
                        .chain(extra.iter().copied())
                        .chain(rest.iter().copied()),
                )
            };
            assert_eq!(replica(&parse(&[]).unwrap()), 0, "{verb}");
            assert_eq!(replica(&parse(&["--replica", "3"]).unwrap()), 3, "{verb}");
            assert!(parse(&["--replica", "-1"]).is_err(), "{verb}");
            assert!(parse(&["--replica", "x"]).is_err(), "{verb}");
            let help = command.clone().render_long_help().to_string();
            assert!(help.contains("--replica <N>"), "{verb}: {help}");
            assert!(
                help.contains("The replica to reach, from 0"),
                "{verb}: {help}"
            );
        }
    }

    #[test]
    fn shell_takes_replicas_only_with_gpu_and_a_replica_below_it() {
        let parse = |args: &[&str]| {
            shell_command()
                .try_get_matches_from(std::iter::once("shell").chain(args.iter().copied()))
        };
        let gpu = ["--gpu", "h100:1", "--image", "i"];
        assert!(parse(&["train", "--replicas", "2"]).is_err());
        let with = |extra: &[&str]| parse(&[&gpu[..], extra].concat()).unwrap();
        assert_eq!(with(&[]).get_one::<i64>("replicas"), None);
        assert_eq!(
            with(&["--replicas", "2"]).get_one::<i64>("replicas"),
            Some(&2)
        );
        for bad in ["0", "65", "-1"] {
            assert!(
                parse(&[&gpu[..], &["--replicas", bad]].concat()).is_err(),
                "{bad}"
            );
        }
        assert_eq!(check_shell_replica(&with(&[])).unwrap(), 0);
        assert_eq!(
            check_shell_replica(&with(&["--replicas", "2", "--replica", "1"])).unwrap(),
            1
        );
        let f = check_shell_replica(&with(&["--replica", "1"])).unwrap_err();
        assert_eq!(f.class, Class::Rejected);
        assert!(
            f.message.contains("the 1 replica --gpu creates"),
            "{}",
            f.message
        );
        let f = check_shell_replica(&with(&["--replicas", "2", "--replica", "2"])).unwrap_err();
        assert!(f.message.contains("the 2 replicas"), "{}", f.message);
        let help = shell_command().render_long_help().to_string();
        assert!(help.contains("--replicas <N>"), "{help}");
    }

    #[test]
    fn exec_refuses_an_env_pair_without_a_key() {
        let m = exec_command()
            .try_get_matches_from(["exec", "train", "--env", "=v", "--", "true"])
            .unwrap();
        assert_eq!(exec_env(&m).unwrap_err().class, Class::Rejected);
    }

    mod before_a_create {
        use super::*;
        use serde_json::json;
        use std::time::Duration;
        use tokio::time::Instant;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        async fn server(delay: Duration) -> MockServer {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/gpus"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(json!({"data": [{"gpu_model": "h100", "gpu_memory_gb": 80,
                            "usd_per_gpu_hour": 2.5, "available": false}]}))
                        .set_delay(delay),
                )
                .mount(&server)
                .await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/quota"))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_body_json(json!({"gpu_quota_remaining": 0}))
                        .set_delay(delay),
                )
                .mount(&server)
                .await;
            server
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn prompt_reads_give_both_warnings() {
            let server = server(Duration::ZERO).await;
            let api = Api::for_tests(&server.uri());
            let lines = warnings_before_create(&api, "h100:1", 1, Duration::from_secs(5)).await;
            assert_eq!(lines.len(), 2, "{lines:?}");
            assert!(lines[0].contains("No h100 is free"), "{lines:?}");
            assert!(lines[1].contains("0 GPUs left"), "{lines:?}");
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn reads_past_their_timeout_are_skipped_without_waiting() {
            let server = server(Duration::from_secs(3)).await;
            let api = Api::for_tests(&server.uri());
            let started = Instant::now();
            let lines = warnings_before_create(&api, "h100:1", 1, Duration::from_millis(200)).await;
            assert!(lines.is_empty(), "{lines:?}");
            assert!(started.elapsed() < Duration::from_secs(1));
        }

        /// The quota counts every replica's GPUs: two left holds one
        /// two-GPU instance, but not two of them.
        #[tokio::test(flavor = "multi_thread")]
        async fn the_quota_warning_counts_every_replica() {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/sandboxes/quota"))
                .respond_with(
                    ResponseTemplate::new(200).set_body_json(json!({"gpu_quota_remaining": 2})),
                )
                .mount(&server)
                .await;
            let api = Api::for_tests(&server.uri());
            let read = Duration::from_secs(5);
            assert!(warnings_before_create(&api, "h100:2", 1, read)
                .await
                .is_empty());
            let lines = warnings_before_create(&api, "h100:1", 4, read).await;
            assert_eq!(lines.len(), 1, "{lines:?}");
            assert!(lines[0].contains("2 GPUs left"), "{lines:?}");
            assert_eq!(
                warnings_before_create(&api, "h100:2", 2, read).await.len(),
                1
            );
        }
    }

    /// `--replicas`' range is the create route's, so a regeneration that
    /// moves it fails here.
    #[test]
    fn max_replicas_is_the_create_routes_maximum() {
        let spec: serde_json::Value =
            serde_json::from_str(include_str!("../openapi0.json")).unwrap();
        let replicas =
            &spec["components"]["schemas"]["CreateSandboxRequest"]["properties"]["replicas"];
        assert_eq!(replicas["maximum"], MAX_REPLICAS, "{replicas}");
        assert_eq!(replicas["minimum"], 1, "{replicas}");
    }
}
