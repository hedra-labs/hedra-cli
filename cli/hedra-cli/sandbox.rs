//! Preview command scaffold for ENG-11519, following diffusion#8975.
//!
//! No transport or value-reading path until the generated sandbox SDK and
//! hedra-compute#664 -> #668 -> #669 are ready. Keep the refusal ahead of
//! stdin, files, credentials, and HTTP, including --debug and --dry-run.

use clap::{Arg, ArgAction, ArgMatches, Command};
use fern_cli_sdk::error::CliError;

const PREVIEW_ENV: &str = "HEDRA_CLI_SANDBOX_PREVIEW";

fn preview_enabled() -> bool {
    std::env::var(PREVIEW_ENV).as_deref() == Ok("1")
}

pub(crate) fn command() -> Command {
    Command::new("sandbox")
        .about("GPU sandbox commands (preview)")
        .hide(!preview_enabled())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("secrets")
                .about("Manage workspace secrets (not available yet)")
                .long_about(
                    "Workspace secrets shared by sandboxes in the active workspace. \
                     This preview only reserves the command interface: all operations \
                     fail without reading values or sending requests, even with --dry-run.",
                )
                .subcommand_required(true)
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("set")
                        .about("Store or replace a workspace secret (not available yet)")
                        .arg(name_arg())
                        .arg(
                            Arg::new("from-file")
                                .long("from-file")
                                .value_name("PATH")
                                .value_parser(clap::value_parser!(std::path::PathBuf))
                                .help("Reserved file input; no file or stdin is read yet"),
                        ),
                )
                .subcommand(
                    Command::new("ls").about("List workspace secret metadata (not available yet)"),
                )
                .subcommand(
                    Command::new("rm")
                        .about("Delete a workspace secret (not available yet)")
                        .arg(name_arg())
                        .arg(
                            Arg::new("yes").long("yes").action(ArgAction::SetTrue).help(
                                "Reserved noninteractive confirmation; nothing is deleted yet",
                            ),
                        ),
                ),
        )
}

fn name_arg() -> Arg {
    Arg::new("name")
        .value_name("NAME")
        .required(true)
        .help("Workspace secret name (environment variable name)")
}

pub(crate) fn dispatch(_: &ArgMatches, _: &dyn std::any::Any) -> Result<(), CliError> {
    if !preview_enabled() {
        return Err(CliError::Validation(format!(
            "Sandbox commands are in preview. Set {PREVIEW_ENV}=1 to inspect the command interface."
        )));
    }
    // Use the existing structured error path while the shared sandbox error
    // classes from diffusion#8975 are pending. Never report a stub as success.
    Err(CliError::Validation(
        "Secrets are not available yet.".to_string(),
    ))
}
