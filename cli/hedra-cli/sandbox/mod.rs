//! The sandbox stream verbs: `exec`, `shell` (and `shell --gpu`), `logs` and
//! `attach`, which hold a stream open to the sandbox through the broker. Each
//! is grafted into the generated `sandbox` group beside the generated control
//! commands (`ls`, `get`, `create`, `stop`, …), which this module does not
//! touch.
//!
//! Unless `HEDRA_CLI_SANDBOX_PREVIEW=1`, the verbs are not grafted at all, so
//! no help, completion script or typo suggestion names them, and
//! [`exit_if_unreleased`] refuses a command line that runs one.

mod api;
mod error;
mod exec;
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
const PREVIEW_ENV: &str = "HEDRA_CLI_SANDBOX_PREVIEW";

/// The verbs this module grafts into the generated group. A test asserts that
/// no generated leaf of the group shares one of these names, since a graft
/// replaces a generated leaf of the same name without a warning.
pub(crate) const VERBS: [&str; 4] = ["exec", "shell", "logs", "attach"];

pub(crate) fn preview_enabled() -> bool {
    std::env::var(PREVIEW_ENV).is_ok_and(|v| v == "1")
}

const EXIT_CODES: &str = "\
Exit codes:
   0  success (exec and shell: the command's own exit code)
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
 130  interrupted
 141  the output was closed
 143  terminated
 255  exec, shell: hedra-cli failed, not the command";

/// The stream verbs' commands, each grafted under `sandbox` with its handler,
/// or none without the preview. A hidden clap command still appears in the
/// runtime's completion scripts, in clap's "a similar subcommand exists" tip
/// and in `sandbox help <verb>`, so the verbs are left out of the tree.
pub(crate) fn commands() -> Vec<(Command, fern_cli_sdk::app::CliCommandHandler)> {
    if !preview_enabled() {
        return Vec::new();
    }
    stream_commands()
}

fn stream_commands() -> Vec<(Command, fern_cli_sdk::app::CliCommandHandler)> {
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
    ]
}

/// Without the preview, refuses a command line that runs a stream verb,
/// sending nothing, before the runtime would answer it with an unknown
/// subcommand. `exec` and `shell` exit 255 under the 255 rule, since their
/// other codes are the remote command's; `logs` and `attach` exit with the
/// Rejected request code. `args` is the command line without the program name.
pub(crate) fn exit_if_unreleased(args: &[String]) {
    if preview_enabled() {
        return;
    }
    let Some(verb) = stream_verb_in(args) else {
        return;
    };
    let f = Failure::rejected(format!(
        "The sandbox stream commands ({}) are not released yet.",
        VERBS.join(", ")
    ));
    let code = match verb {
        "exec" | "shell" => 255,
        _ => f.class.code(),
    };
    f.write_stderr(fern_cli_sdk::formatter::explicit_machine_format(args));
    std::process::exit(code);
}

/// The stream verb `args` runs, when its first two positional arguments are
/// `sandbox` and one of [`VERBS`]. Global flags before them are skipped, with
/// their values.
fn stream_verb_in(args: &[String]) -> Option<&'static str> {
    let mut positionals: Vec<&str> = Vec::new();
    let mut args = args.iter().map(String::as_str);
    while positionals.len() < 2 {
        let arg = args.next()?;
        if arg == "--" {
            return None;
        }
        if !arg.starts_with('-') || arg == "-" {
            positionals.push(arg);
            continue;
        }
        let takes_value = super::dotenv::VALUE_FLAGS.contains(&arg)
            || super::dotenv::SHORT_VALUE_FLAGS.contains(&arg)
            || arg == "--base-url";
        if takes_value {
            let _ = args.next();
        }
    }
    match positionals.as_slice() {
        [GROUP, verb] => VERBS.iter().copied().find(|v| v == verb),
        _ => None,
    }
}

fn sandbox_arg() -> Arg {
    Arg::new("sandbox")
        .value_name("SANDBOX")
        .required(true)
        .help("The sandbox's name, or its `sandbox_` id")
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
        .arg(term_arg())
}

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
}

fn attach_command() -> Command {
    Command::new("attach")
        .about("Follow a sandbox's main process output; the same as logs -f")
        .after_long_help(EXIT_CODES)
        .arg(sandbox_arg())
}

/// A verb's handler: the preview gate, then the verb, then its exit code.
/// Every verb exits from here, since its stdout is the sandbox's output and
/// its errors go to stderr.
fn dispatch(verb: &str, m: &ArgMatches, ctx: &dyn std::any::Any) -> Result<(), CliError> {
    let ctx = ctx
        .downcast_ref::<AppContext>()
        .ok_or_else(|| CliError::Validation("sandbox: unexpected binding context type".into()))?;
    let args: Vec<String> = std::env::args().collect();
    let json_errors = fern_cli_sdk::formatter::explicit_machine_format(&args);
    let flag = |name: &str| {
        m.try_get_one::<bool>(name)
            .ok()
            .flatten()
            .copied()
            .unwrap_or(false)
    };
    let fail = |f: Failure, code: i32| -> ! {
        f.write_stderr(json_errors);
        let _ = std::io::stdout().flush();
        std::process::exit(code);
    };
    if flag("dry-run") {
        if let Err(f) = dry_run(verb, m) {
            let code = f.class.code();
            fail(f, code);
        }
        std::process::exit(0);
    }
    let api = Api::new(ctx, flag("debug"));
    let code = match verb {
        "logs" | "attach" => {
            let follow = verb == "attach" || m.get_flag("follow");
            match target(m).and_then(|t| {
                stream::transport_config(ctx.http_config()).map(|transport| (t, transport))
            }) {
                Ok((t, transport)) => {
                    unwinding(|| run_async(logs::run(&api, transport, &t, follow, json_errors)))
                }
                Err(f) => {
                    let code = f.class.code();
                    fail(f, code);
                }
            }
        }
        "shell" if m.get_one::<String>("gpu").is_some() => match gpu_shell_args(m, ctx) {
            Ok((shell, transport)) => {
                unwinding(|| run_async(gpu_shell::run(&api, transport, shell, json_errors)))
            }
            Err(f) => fail(f, 255),
        },
        _ => match stream_verb(verb, m, ctx) {
            Ok((target, exec, transport)) => {
                unwinding(|| run_async(exec::run(&api, transport, &target, exec, json_errors)))
            }
            Err(f) => fail(f, 255),
        },
    };
    let _ = std::io::stdout().flush();
    std::process::exit(code);
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
        }
    }
    if verb == "exec" {
        exec_env(m)?;
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
    };
    Ok((shell, stream::transport_config(ctx.http_config())?))
}

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
            no_stdin: false,
            env: HashMap::new(),
            term,
        }
    } else {
        exec::Exec {
            argv: m
                .get_many::<String>("command")
                .expect("required")
                .cloned()
                .collect(),
            tty: m.get_flag("tty"),
            no_stdin: m.get_flag("no-stdin"),
            env: exec_env(m)?,
            term,
        }
    };
    let transport = stream::transport_config(ctx.http_config())?;
    Ok((target, exec, transport))
}

/// Runs a stream verb, turning a panic into exit 255 with its message
/// printed after the terminal is restored. The default hook would print it
/// while the terminal is raw, where each line starts where the last ended.
fn unwinding(verb: impl FnOnce() -> i32) -> i32 {
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
            let mut f = Failure::new(Class::OutputFailed, error::escape_controls(&text));
            if let Some((name, id)) = gpu_shell::created() {
                f = f.then(format!(
                    "`{name}` may still be running; stop it with `{}`.",
                    render::stop_command(&id)
                ));
            }
            stream::note(format_args!("hedra-cli: Panic (255): {}", f.message));
            255
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
        "`{gpu}` is not a GPU model and count such as h100:1. Run `hedra-cli sandbox gpus`."
    )))
}

/// How long each read before `shell --gpu`'s create may take.
const PRE_CREATE_READ: std::time::Duration = std::time::Duration::from_secs(5);

/// The warnings from the two reads before `shell --gpu`'s create: on any
/// failure or after `read`, a read's line is skipped and the create goes
/// ahead.
async fn warnings_before_create(api: &Api, gpu: &str, read: std::time::Duration) -> Vec<String> {
    let (model, count) = gpu.split_once(':').unwrap_or((gpu, "1"));
    let count: i64 = count.parse().unwrap_or(1);
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

    /// A graft replaces a generated leaf of the same name without a
    /// warning, so a regeneration that adds a `sandbox` operation named like a
    /// stream verb fails here instead of losing the generated command.
    #[test]
    fn no_generated_sandbox_command_shares_a_stream_verbs_name() {
        let spec: serde_json::Value =
            serde_json::from_str(include_str!("../openapi0.json")).expect("the embedded spec");
        let mut generated = Vec::new();
        for item in spec["paths"].as_object().expect("paths").values() {
            for op in item.as_object().into_iter().flat_map(|o| o.values()) {
                let group = op.get("x-fern-sdk-group-name").and_then(|g| g.as_array());
                let in_sandbox = group.is_some_and(|g| g.len() == 1 && g[0] == GROUP);
                if let (true, Some(name)) = (
                    in_sandbox,
                    op.get("x-fern-sdk-method-name").and_then(|n| n.as_str()),
                ) {
                    generated.push(name.to_string());
                }
            }
        }
        assert!(generated.iter().any(|n| n == "get"), "{generated:?}");
        for verb in VERBS {
            assert!(
                !generated.iter().any(|n| n == verb),
                "`sandbox {verb}` is generated too"
            );
        }
        let grafted: Vec<String> = stream_commands()
            .iter()
            .map(|(c, _)| c.get_name().to_string())
            .collect();
        assert_eq!(grafted, VERBS);
    }

    #[test]
    fn a_stream_verb_is_found_after_global_flags_and_their_values() {
        let args = |line: &str| {
            line.split_whitespace()
                .map(String::from)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            stream_verb_in(&args("sandbox exec train -- true")),
            Some("exec")
        );
        assert_eq!(
            stream_verb_in(&args(
                "--format json -p work --base-url x sandbox logs train"
            )),
            Some("logs")
        );
        assert_eq!(
            stream_verb_in(&args("--debug sandbox attach train")),
            Some("attach")
        );
        assert_eq!(stream_verb_in(&args("sandbox get train")), None);
        assert_eq!(stream_verb_in(&args("sandbox")), None);
        assert_eq!(stream_verb_in(&args("--format sandbox exec")), None);
        assert_eq!(stream_verb_in(&args("-- sandbox exec")), None);
    }

    #[test]
    fn gpus_follow_the_create_routes_pattern() {
        for ok in ["h100:1", "h100:8", "a100:4", "h100:99", "a-1:2"] {
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
            let lines = warnings_before_create(&api, "h100:1", Duration::from_secs(5)).await;
            assert_eq!(lines.len(), 2, "{lines:?}");
            assert!(lines[0].contains("No h100 is free"), "{lines:?}");
            assert!(lines[1].contains("0 GPUs left"), "{lines:?}");
        }

        #[tokio::test(flavor = "multi_thread")]
        async fn reads_past_their_timeout_are_skipped_without_waiting() {
            let server = server(Duration::from_secs(3)).await;
            let api = Api::for_tests(&server.uri());
            let started = Instant::now();
            let lines = warnings_before_create(&api, "h100:1", Duration::from_millis(200)).await;
            assert!(lines.is_empty(), "{lines:?}");
            assert!(started.elapsed() < Duration::from_secs(1));
        }
    }
}
