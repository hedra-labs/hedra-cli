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
#[path = "views.rs"]
mod views;
#[path = "workspaces.rs"]
mod workspaces;

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
    // it here too is safe: dotenvy never overrides existing process env,
    // and the runtime's later call just becomes a no-op.
    let _ = dotenvy::dotenv();

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

    let app = app
        .login_flow(auth::EnvPkceLoginFlow::new())
        .command(workspaces::command(), Box::new(workspaces::dispatch));

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
    let doc = load_openapi_spec(include_str!("openapi0.json"), "hedra-cli").ok()?;
    let cli = commands::build_cli(&doc)
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
