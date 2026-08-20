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
// directory, keeping them at cli/hedra-cli/ next to this file. All three are
// .fernignore-protected like custom.rs itself.
#[path = "active_key.rs"]
mod active_key;
#[path = "auth.rs"]
mod auth;
#[path = "keyring_cache.rs"]
mod keyring_cache;
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
        app
    } else {
        app.transform_response(&["models", "list"], reshape_model_list_table)
    }
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
    let probe = || -> Option<OutputFormat> {
        let doc = load_openapi_spec(include_str!("openapi0.json"), "hedra-cli").ok()?;
        let cli = commands::build_cli(&doc).ignore_errors(true);
        let matches = cli.try_get_matches_from(std::env::args_os()).ok()?;
        OutputPipeline::from_matches(&matches, "hedra-cli")
            .ok()
            .map(|p| p.format)
    };
    probe().unwrap_or_else(|| {
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
