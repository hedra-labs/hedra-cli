//! Custom command handlers.
//!
//! This file is yours to edit — add it to `.fernignore` so
//! `fern generate` will never overwrite your changes.
//!
//! The generated `main.rs` calls `custom::register(app)` at
//! startup, composing your commands into the CLI at compile time.
//!
//! Each handler receives an `AppContext`. Use `super::sdk::client(ctx)`
//! to get a fully-wired SDK client that inherits the CLI's auth,
//! retries, TLS, and global headers. Use `super::sdk::block_on(future)`
//! to run async SDK calls from synchronous handler context.
//! Types are available via `hedra_cli_sdk::api::*`.

// Hand-written sibling modules, declared here rather than in main.rs
// (which is regenerated). `#[path]` resolves relative to this file's
// directory, keeping them at cli/hedra-cli/ next to this file. All of them are
// .fernignore-protected like custom.rs itself.
#[path = "active_key.rs"]
mod active_key;
#[path = "auth.rs"]
mod auth;
#[path = "keyring_cache.rs"]
mod keyring_cache;
#[path = "media_refs.rs"]
mod media_refs;
#[path = "views.rs"]
mod views;
#[path = "workspaces.rs"]
mod workspaces;

use std::io::Write;

use fern_cli_sdk::app::CliApp;
use fern_cli_sdk::error::CliError;
use fern_cli_sdk::formatter::{OutputFormat, OutputPipeline};
use fern_cli_sdk::openapi::{commands, load_openapi_spec};
use serde_json::Value;

/// Register custom commands on the CLI app builder.
///
/// Called from `main.rs` during startup.
pub fn register(app: CliApp) -> CliApp {
    // The runtime only loads .env inside run() — after this function has
    // returned — so any env read below would miss .env-set values. Loading
    // it here too is safe: the filtered loader never overrides existing
    // process env, and the runtime's later call sets nothing new. It is the
    // runtime's own filtered loader, so a `.env` cannot set here a key the
    // runtime refuses from one.
    //
    // The login endpoint overrides start with `HEDRA_`, so the runtime's
    // allow-list admits them and this load sets a `.env` value for them;
    // auth.rs then ignores it, comparing against what was set before.
    //
    // Ignored keys meant for this CLI are named once on stderr; see
    // `ignored_dotenv_keys_line`. The runtime's own report is a tracing
    // warning, shown only when `HEDRA_CLI_LOG` asks for warnings, and it
    // cannot know about the endpoint overrides.
    let endpoint_override_keys_in_env = auth::endpoint_override_keys_in_env();
    let ignored = fern_cli_sdk::load_dotenv_filtered("hedra-cli");
    let ignored_overrides =
        auth::ignore_endpoint_override_from_dotenv(&endpoint_override_keys_in_env);
    if let Some(line) = ignored_dotenv_keys_line(&ignored, &ignored_overrides) {
        // Not `eprintln!`, which panics when stderr is a closed pipe: this
        // runs before the runtime resets SIGPIPE handling.
        let _ = writeln!(std::io::stderr(), "{line}");
    }

    // Credential-store stack, outermost first. Must be installed before
    // anything resolves a credential; nothing in the SDK installs a store
    // eagerly, and this runs before CliApp::run, so it wins the slot.
    //
    // The KeyAuth projection sits outside the memo so the workspace-map read
    // it performs is cached like any other. The reverse order would memoise
    // a derived value that a later write to the map could not invalidate.
    fern_cli_sdk::auth::set_active_store(active_key::project(keyring_cache::memoize(
        fern_cli_sdk::auth::auto_store(),
    )));

    // One knob: HEDRA_ENV=staging retargets the data plane too, unless an
    // explicit HEDRA_CLI_BASE_URL / --base-url says otherwise.
    auth::derive_base_url_from_hedra_env();

    // `--input.source-image asset_…` / `https://…` as shorthand for the
    // MediaRef object. Request-side, so it applies in every output format.
    media_refs::install();

    let app = app
        .login_flow(auth::EnvPkceLoginFlow::new())
        .command(workspaces::command(), Box::new(workspaces::dispatch));

    // The runtime builds its commands from the embedded spec, which has no
    // `/sandboxes` paths, so no `sandboxes` group appears today. A spec that
    // had them would add a generated group, which stays out of `--help`.
    // Hidden only when present, since `CliApp::hide`
    // panics on a missing command, and before the table-only return below,
    // since it applies in every format. The hide keeps the group out of
    // `--help` and `man`; shell completion, `--format json --help`,
    // `--schema` and `generate-skills` would still list it or its operations,
    // a mistyped command can still be answered with a did-you-mean suggestion
    // of it, and `sandboxes …` itself still runs.
    let app = hide_generated_sandboxes(app, embedded_cli());

    if resolve_format_from_argv() != OutputFormat::Table {
        return app;
    }

    // Table-only response reshaping. Every hook below exists because the
    // generic table renderer drops fields, and none of them may run for
    // machine-readable formats — json/yaml/csv/jsonl output stays
    // byte-identical to the server's, so scripts are unaffected.
    //
    // `["jobs", "*"]` is one registration rather than ~85: the jobs group
    // carries `submit-<model>` leaves for every model in the catalog, and
    // `views::jobs` selects the view from the operation path.
    //
    // The remaining four groups get the generic array de-hijacker. They
    // are named individually rather than globbed across the whole CLI so
    // that adding a resource is a deliberate choice — `views::dehijack`
    // passes list envelopes through untouched, but a hook that silently
    // covered every future endpoint would be harder to reason about.
    app.transform_response(&["models", "list"], reshape_model_list_table)
        .transform_response(&["jobs", "*"], views::jobs)
        .transform_response(&["keys", "*"], views::defuse_arrays)
        .transform_response(&["webhooks", "*"], views::defuse_arrays)
        .transform_response(&["log-drains", "*"], views::defuse_arrays)
        .transform_response(&["billing", "*"], views::defuse_arrays)
}

/// Keys outside `HEDRA_` that the runtime's generated `.env` deny-list names
/// (`DOTENV_DENIED_BARE` in src/lib.rs, which is private to the runtime): a
/// `.env` that sets one was most likely meant to configure this CLI.
const REPORTED_FOREIGN_KEYS: [&str; 8] = [
    "PAGER",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "SSL_CERT_FILE",
    "CURL_CA_BUNDLE",
    "REQUESTS_CA_BUNDLE",
];

/// The stderr line naming the ignored `.env` keys meant for this CLI, or
/// `None` when there are none.
///
/// A project's `.env` usually holds keys for other programs, such as
/// `DATABASE_URL`; naming those on every run, with advice to export them,
/// would be noise. So the line lists only ignored keys starting with
/// `HEDRA_`, the proxy, trust-store and pager keys in
/// [`REPORTED_FOREIGN_KEYS`], and the ignored login endpoint overrides, all
/// compared in any case. It also leaves out a key the process environment
/// already holds: the exported value applies, so there is nothing to tell the
/// user. The runtime's tracing warning still lists every ignored key at
/// `HEDRA_CLI_LOG=warn`.
fn ignored_dotenv_keys_line(ignored: &[String], ignored_overrides: &[&str]) -> Option<String> {
    let keys: Vec<&str> = ignored
        .iter()
        .map(String::as_str)
        .filter(|key| is_reported_dotenv_key(key) && std::env::var_os(key).is_none())
        .chain(ignored_overrides.iter().copied())
        .collect();
    if keys.is_empty() {
        return None;
    }
    Some(format!(
        "hedra-cli: ignored from .env (export them from your shell instead): {}",
        keys.join(", ")
    ))
}

/// Whether an ignored `.env` key belongs in [`ignored_dotenv_keys_line`].
fn is_reported_dotenv_key(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    key.starts_with("HEDRA_") || REPORTED_FOREIGN_KEYS.contains(&key.as_str())
}

/// The command group the generator would build from `/sandboxes`.
const GENERATED_SANDBOXES: &str = "sandboxes";

/// The command tree the runtime builds from the embedded spec, built once:
/// the spec is over a megabyte, and both the hide above and the format probe
/// below read the tree on every start. None when the spec does not parse.
fn embedded_cli() -> Option<&'static clap::Command> {
    static CLI: std::sync::OnceLock<Option<clap::Command>> = std::sync::OnceLock::new();
    CLI.get_or_init(|| cli_from_spec(include_str!("openapi0.json")))
        .as_ref()
}

fn cli_from_spec(spec: &str) -> Option<clap::Command> {
    load_openapi_spec(spec, "hedra-cli")
        .ok()
        .map(|doc| commands::build_cli(&doc))
}

/// `app` with the generated `sandboxes` group hidden when `cli`, the tree the
/// runtime builds from the same spec, has one.
fn hide_generated_sandboxes(app: CliApp, cli: Option<&clap::Command>) -> CliApp {
    if cli.is_some_and(|cli| has_command(cli, GENERATED_SANDBOXES)) {
        app.hide(&[GENERATED_SANDBOXES])
    } else {
        app
    }
}

/// Whether `cli` has a top-level `name` command.
fn has_command(cli: &clap::Command, name: &str) -> bool {
    cli.find_subcommand(name).is_some()
}

async fn reshape_model_list_table(mut v: Value, _: Vec<String>) -> Result<Value, CliError> {
    const COLUMNS: [&str; 4] = ["id", "name", "modality", "description"];

    if let Some(Value::Array(models)) = v.get_mut("data") {
        for model in models {
            if let Value::Object(fields) = model {
                let mut source = std::mem::take(fields);
                *fields = COLUMNS
                    .into_iter()
                    .filter_map(|key| source.remove(key).map(|value| (key.to_owned(), value)))
                    .collect();
            }
        }
    }

    Ok(v)
}

fn resolve_format_from_argv() -> OutputFormat {
    resolve_format(std::env::args_os())
}

/// The format the hooks below gate on, resolved from `argv` alone.
///
/// Split from [`resolve_format_from_argv`] so the flag handling can be
/// tested against a synthetic command line: the fallback arm reads the
/// environment and the TTY, but the probe arm — the one that decides
/// whether a hook runs at all — does not, so every case that reaches it
/// is deterministic under `cargo test`.
fn resolve_format<I, T>(argv: I) -> OutputFormat
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    probe_format(argv).unwrap_or_else(|| {
        // parse failed (--help, custom command, exotic argv): env/TTY default
        use std::io::IsTerminal;
        std::env::var("HEDRA_CLI_OUTPUT")
            .ok()
            .and_then(|v| OutputFormat::parse(&v).ok())
            .unwrap_or(if std::io::stdout().is_terminal() {
                OutputFormat::Table
            } else {
                OutputFormat::Json
            })
    })
}

fn probe_format<I, T>(argv: I) -> Option<OutputFormat>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let cli = embedded_cli()?
        .clone()
        .ignore_errors(true)
        // `--human` is the other way to ask for a table, and it is
        // declared on the `CliApp` root rather than on the binding
        // root `build_cli` returns — so this probe CLI does not know
        // the flag, `ignore_errors` swallows it, and
        // `OutputPipeline::from_matches` falls through to the
        // env/TTY default. Piping `--human` therefore resolved to
        // Json and skipped every hook below, which is precisely the
        // case the flag exists for: a human reading a table through
        // a pager or a redirect. Re-declaring it here is enough —
        // `from_matches` consults `human` only when `--format` is
        // absent, so a real `--format` still wins.
        .arg(
            clap::Arg::new("human")
                .long("human")
                .action(clap::ArgAction::SetTrue)
                .global(true),
        );
    let matches = cli.try_get_matches_from(argv).ok()?;
    OutputPipeline::from_matches(&matches, "hedra-cli")
        .ok()
        .map(|p| p.format)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The embedded spec with one `/sandboxes` operation added, as a Fern CLI
    /// that builds the embedded spec from the generator's spec would write it.
    fn spec_with_sandboxes() -> String {
        let mut spec: Value = serde_json::from_str(include_str!("openapi0.json")).unwrap();
        spec["paths"]["/sandboxes"] = serde_json::json!({
            "get": {
                "tags": ["Sandboxes"],
                "summary": "List sandboxes",
                "operationId": "sandboxes_list",
                "responses": {"200": {"description": "Your sandboxes."}}
            }
        });
        spec.to_string()
    }

    /// Fails on the regeneration the hide exists for: an embedded spec that
    /// gains `/sandboxes` paths. The hide still covers that case, so update
    /// this test when it fails.
    #[test]
    fn the_embedded_spec_has_no_generated_sandboxes_group() {
        let cli = embedded_cli().expect("the embedded spec parses");
        assert!(!has_command(cli, GENERATED_SANDBOXES));
    }

    #[test]
    fn a_spec_with_sandbox_routes_has_the_generated_group() {
        let cli = cli_from_spec(&spec_with_sandboxes()).expect("the spec parses");
        assert!(has_command(&cli, GENERATED_SANDBOXES));
    }

    /// `--help` from an app whose embedded spec has the sandbox routes.
    fn help_from_spec_with_sandboxes(hide: bool, format: &[&str]) -> String {
        let spec: &'static str = Box::leak(spec_with_sandboxes().into_boxed_str());
        let app = CliApp::new("hedra-cli")
            .binding(fern_cli_sdk::openapi::OpenApiBinding::new().spec(spec));
        let app = if hide {
            hide_generated_sandboxes(app, cli_from_spec(spec).as_ref())
        } else {
            app
        };
        let mut out = Vec::new();
        let args: Vec<&str> = std::iter::once("hedra-cli")
            .chain(format.iter().copied())
            .chain(["--help"])
            .collect();
        let _ = app.try_run_from_with_output(args, &mut out);
        String::from_utf8_lossy(&out).into_owned()
    }

    #[test]
    fn the_hide_keeps_a_generated_sandboxes_group_out_of_help() {
        let shown = help_from_spec_with_sandboxes(false, &[]);
        assert!(
            shown.contains("sandboxes"),
            "help without the hide:\n{shown}"
        );
        // Human help only: `--format json --help` prints the runtime's spec
        // schema, which lists every operation whatever clap's hidden flag says.
        for format in [&[][..], &["--format", "table"][..]] {
            let help = help_from_spec_with_sandboxes(true, format);
            assert!(help.contains("jobs"), "help with {format:?}:\n{help}");
            assert!(!help.contains("sandboxes"), "help with {format:?}:\n{help}");
        }
    }

    #[test]
    fn the_hide_is_a_no_op_without_a_generated_group() {
        let app = CliApp::new("hedra-cli").binding(
            fern_cli_sdk::openapi::OpenApiBinding::new().spec(include_str!("openapi0.json")),
        );
        let app = hide_generated_sandboxes(app, embedded_cli());
        let mut out = Vec::new();
        let _ = app.try_run_from_with_output(["hedra-cli", "--help"], &mut out);
        assert!(String::from_utf8_lossy(&out).contains("jobs"));
    }

    /// `REPORTED_FOREIGN_KEYS` copies the runtime's private `DOTENV_DENIED_BARE`;
    /// a regeneration that changes that list fails here.
    #[test]
    fn reported_foreign_keys_match_the_runtime_bare_deny_list() {
        let lib = include_str!("../../src/lib.rs");
        let start = lib
            .find("const DOTENV_DENIED_BARE: &[&str] = &[")
            .expect("src/lib.rs declares DOTENV_DENIED_BARE");
        let end = start + lib[start..].find("];").expect("DOTENV_DENIED_BARE ends");
        let bare: Vec<&str> = lib[start..end]
            .lines()
            .skip(1)
            .map(str::trim)
            .filter(|line| line.starts_with('"'))
            .map(|line| line.trim_end_matches(',').trim_matches('"'))
            .collect();
        assert_eq!(bare, REPORTED_FOREIGN_KEYS);
    }

    #[test]
    fn the_hide_is_a_no_op_when_the_spec_does_not_parse() {
        let app = CliApp::new("hedra-cli").binding(
            fern_cli_sdk::openapi::OpenApiBinding::new().spec(include_str!("openapi0.json")),
        );
        let app = hide_generated_sandboxes(app, None);
        let mut out = Vec::new();
        let _ = app.try_run_from_with_output(["hedra-cli", "--help"], &mut out);
        assert!(String::from_utf8_lossy(&out).contains("jobs"));
    }

    #[test]
    fn an_unparseable_spec_builds_no_command_tree() {
        assert!(cli_from_spec("not: [a spec").is_none());
    }

    fn argv(args: &[&str]) -> Vec<String> {
        std::iter::once("hedra-cli")
            .chain(args.iter().copied())
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn human_asks_for_a_table() {
        // The regression this guards: `--human` is declared on the
        // `CliApp` root, not on the binding root the probe is built
        // from, so it used to be swallowed by `ignore_errors` and the
        // resolved format fell back to the env/TTY default. Piped —
        // which is the only time anyone types `--human` — that meant
        // Json, and every table view was skipped. `jobs get --human`
        // rendered its `logs` array and dropped the other ten fields.
        assert_eq!(
            probe_format(argv(&["jobs", "get", "--job-id", "job_x", "--human"])),
            Some(OutputFormat::Table)
        );
    }

    #[test]
    fn human_asks_for_a_table_on_a_streaming_leaf_too() {
        assert_eq!(
            probe_format(argv(&["jobs", "stream", "--job-id", "job_x", "--human"])),
            Some(OutputFormat::Table)
        );
    }

    #[test]
    fn an_explicit_format_still_wins() {
        // `from_matches` consults `human` only when `--format` is
        // absent. The real root marks the two `conflicts_with`, so this
        // combination never reaches a request — the point is that
        // re-declaring `--human` on the probe cannot hijack a format
        // the caller named.
        assert_eq!(
            probe_format(argv(&[
                "jobs", "get", "--job-id", "job_x", "--format", "json"
            ])),
            Some(OutputFormat::Json)
        );
        assert_eq!(
            probe_format(argv(&[
                "jobs", "get", "--job-id", "job_x", "--format", "json", "--human"
            ])),
            Some(OutputFormat::Json)
        );
    }

    #[test]
    fn format_table_resolves_without_the_human_flag() {
        assert_eq!(
            probe_format(argv(&[
                "jobs", "stream", "--job-id", "job_x", "--format", "table"
            ])),
            Some(OutputFormat::Table)
        );
    }
}
