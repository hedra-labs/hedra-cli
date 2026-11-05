//! Loading `.env`.
//!
//! The CLI is routinely run inside repositories its user did not write, and
//! `dotenvy` finds the nearest `.env` in the working directory or a parent.
//! A `.env` that could set any variable could point requests and credentials
//! at another host, add trusted roots, or name a program for the CLI to run,
//! through variables the libraries it links read, which no list of keys to
//! refuse can cover. The same holds inside this CLI's own `HEDRA_` prefix: a
//! later runtime can start reading a new `HEDRA_CLI_…` setting. So a `.env`
//! may set only the exact keys in [`ALLOWED_KEYS`] and the `HEDRA_CLI_…`
//! settings in [`ALLOWED_CLI_SETTINGS`], compared in any case and set under
//! their upper-case name, which is the one the CLI reads: credentials,
//! the target environment, output and logging preferences, retries and
//! timeouts. Every other key is refused. A variable the process environment
//! already holds, as written or in upper case, is left as it is, as
//! `dotenvy::dotenv()` does.
//!
//! The runtime has a `.env` loader of its own, called from `CliApp::run`
//! (`load_dotenv_filtered`, src/lib.rs). It admits every key outside a short
//! deny-list, matched in upper case only, and src/lib.rs is generated, so it
//! cannot be changed here. `custom::register` therefore loads `.env` with
//! [`load`] and runs the command itself through `CliApp::try_run_from`, which
//! skips the runtime's loader; see `custom::register`.

use std::io::Write;

/// The prefix of the runtime's settings for this binary (`HEDRA_CLI` from the
/// binary name `hedra-cli`, plus `_`).
const CLI_PREFIX: &str = "HEDRA_CLI_";

/// Keys outside [`CLI_PREFIX`] that a `.env` may set.
const ALLOWED_KEYS: [&str; 3] = ["HEDRA_API_KEY", "HEDRA_ENV", "NO_COLOR"];

/// The `HEDRA_CLI_…` settings a `.env` may set, by the name after
/// [`CLI_PREFIX`]. Each is one the runtime reads for this binary:
/// `_OUTPUT` (src/formatter.rs), `_LOG` (src/logging.rs), `_RETRIES`
/// (src/openapi/binding.rs), `_TIMEOUT_SECS` and `_CONNECT_TIMEOUT_SECS`
/// (src/http.rs) and `_USER_AGENT_SUFFIX` (src/user_agent.rs).
/// `tests/startup_contract.rs` scans the runtime's non-test source for
/// setting names it builds from the prefix as `{prefix}_NAME` or `{}_NAME` in
/// a format string or as a `"_NAME"` argument of `scoped(`, plus the
/// runtime's own `.env` deny-list, and fails on a name that is neither here
/// nor on its list of settings refused on purpose. A name built another way
/// is not seen, and stays refused.
const ALLOWED_CLI_SETTINGS: [&str; 6] = [
    "OUTPUT",
    "LOG",
    "RETRIES",
    "TIMEOUT_SECS",
    "CONNECT_TIMEOUT_SECS",
    "USER_AGENT_SUFFIX",
];

/// The data-plane base URL setting. A `.env` that sets it fails a command
/// that would send a request to the default API (see [`base_url_refusal`]).
const BASE_URL_KEY: &str = "HEDRA_CLI_BASE_URL";

/// Global flags that take a value, as the next argument or after `=`, for
/// [`sends_request_to_default_target`] (`--base-url` is handled on its own).
const VALUE_FLAGS: [&str; 4] = ["--format", "--user-agent-suffix", "--query", "--profile"];

/// Short global flags that take a value, also attached (`-pNAME`).
const SHORT_VALUE_FLAGS: [&str; 1] = ["-p"];

/// Global flags that take no value.
const BOOLEAN_FLAGS: [&str; 5] = ["--dry-run", "--human", "--quiet", "-q", "--debug"];

/// Flags that make any command print something locally instead of sending a
/// request.
const LOCAL_ONLY_FLAGS: [&str; 7] = [
    "--help",
    "-h",
    "--version",
    "-V",
    "--schema",
    "--spec",
    "--spec-raw",
];

/// Ignored keys outside `HEDRA_` that are named on stderr, compared in any
/// case: a `.env` that sets a proxy, trust-store or pager variable most likely
/// meant it for this CLI.
const REPORTED_FOREIGN_KEYS: [&str; 9] = [
    "PAGER",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "NO_PROXY",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "CURL_CA_BUNDLE",
    "REQUESTS_CA_BUNDLE",
];

/// The keys [`load`] did not set, in file order.
#[derive(Debug, Default, PartialEq)]
pub(crate) struct IgnoredKeys {
    /// Keys a `.env` may not set.
    pub(crate) refused: Vec<String>,
    /// Allowed keys whose value holds a NUL byte, which the environment
    /// cannot store.
    pub(crate) nul_valued: Vec<String>,
}

/// Whether a `.env` may set `key`.
pub(crate) fn key_is_allowed(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    ALLOWED_KEYS.contains(&key.as_str())
        || key
            .strip_prefix(CLI_PREFIX)
            .is_some_and(|setting| ALLOWED_CLI_SETTINGS.contains(&setting))
}

/// Load the nearest `.env`, setting each key [`key_is_allowed`] admits, under
/// its upper-case name, unless the process environment already holds it as
/// written or in upper case. Returns the keys it did
/// not set for those reasons. A missing or unreadable `.env` loads nothing.
pub(crate) fn load() -> IgnoredKeys {
    let Some(contents) = nearest_dotenv().and_then(|path| std::fs::read(path).ok()) else {
        return IgnoredKeys::default();
    };
    load_from(&contents)
}

/// [`load`] for the contents of one `.env` file.
fn load_from(contents: &[u8]) -> IgnoredKeys {
    // `dotenvy::dotenv()` strips a UTF-8 byte order mark, which Windows
    // editors write, but `dotenvy`'s iterators do not, and the first key
    // would then fail to parse and be dropped.
    let contents = contents.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(contents);
    let mut ignored = IgnoredKeys::default();
    for (key, value) in dotenvy::from_read_iter(contents).flatten() {
        let upper = key.to_ascii_uppercase();
        // This CLI's own keys, allowed or refused, are read under their
        // upper-case name, so only that counts as exported. Other programs'
        // keys are left alone in the spelling the shell exports.
        let exported = if key_is_allowed(&key) || upper.starts_with("HEDRA_") {
            std::env::var_os(&upper).is_some()
        } else {
            std::env::var_os(&key).is_some()
        };
        if exported {
            continue;
        }
        if !key_is_allowed(&key) {
            ignored.refused.push(key);
            continue;
        }
        if value.contains('\0') {
            // The environment cannot hold a NUL, and `set_var` would panic.
            // Reported by `report_ignored`, once logging is up.
            ignored.nul_valued.push(key);
            continue;
        }
        // Runs first in `custom::register`, before any other thread starts.
        std::env::set_var(upper, value);
    }
    ignored
}

/// The `.env` file `dotenvy::dotenv()` would load: the first one in the
/// working directory or a parent. Like `dotenvy`, it stops at the first
/// `.env` it cannot inspect for a reason other than its absence.
fn nearest_dotenv() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    for dir in cwd.ancestors() {
        let path = dir.join(".env");
        match std::fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => return Some(path),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return None,
        }
    }
    None
}

/// Report the keys [`load`] did not set: each as a warning at
/// `HEDRA_CLI_LOG=warn`, and, unless `name_refused_keys` is false (the caller
/// is about to fail with the [`base_url_refusal`] error instead), one line on
/// stderr naming the refused keys meant for this CLI.
///
/// Call after logging is initialized, or the warnings are dropped.
pub(crate) fn report_ignored(ignored: &IgnoredKeys, name_refused_keys: bool) {
    for key in &ignored.refused {
        tracing::warn!(
            key = %key,
            "ignoring {key} from .env: a .env file cannot set it. \
             If you set it yourself, set it in your shell."
        );
    }
    for key in &ignored.nul_valued {
        tracing::warn!(key = %key, "ignoring {key} from .env: its value holds a NUL byte");
    }
    if !name_refused_keys {
        return;
    }
    if let Some(line) = ignored_keys_line(&ignored.refused) {
        // Not `eprintln!`, which panics where writing to a closed stderr
        // fails instead of raising SIGPIPE, as on Windows.
        let _ = writeln!(std::io::stderr(), "{line}");
    }
}

/// The error message when the `.env` set the data-plane base URL and the
/// command would otherwise send a request to the default API, or `None`.
///
/// A refused base URL would send such a command, with the stored credential,
/// to the default API instead of the host the `.env` named, so it fails
/// instead. A command that sends nothing, or whose target the user chose
/// (`--base-url`, or `HEDRA_ENV` exported from the shell, which
/// `hedra_env_exported` says), still runs. `args` excludes the program name.
pub(crate) fn base_url_refusal(
    ignored: &IgnoredKeys,
    args: &[String],
    hedra_env_exported: bool,
) -> Option<String> {
    let key = ignored
        .refused
        .iter()
        .find(|key| key.eq_ignore_ascii_case(BASE_URL_KEY))?;
    if hedra_env_exported || !sends_request_to_default_target(args) {
        return None;
    }
    Some(format!(
        "a .env file sets {key}, which a .env file cannot set, so the command did not run \
         rather than send a request to an API other than the one that file names. To run it, \
         set the base URL you want in your shell (for example \
         {BASE_URL_KEY}=https://api.hedra.com/v3) or pass --base-url, or run it from outside \
         the directory that holds the .env file."
    ))
}

/// Whether the command line `args` (without the program name) may send a
/// request to the base URL the environment chooses: not when it asks only
/// for help, the version, a schema or spec, shell completion, a man page or
/// the exit-code table, runs `auth logout` or `auth status`, or passes
/// `--base-url`. Anything this cannot tell apart counts as sending a request:
/// a flag before the command that is not a known global flag, and a bare
/// command group, which prints help for most groups but sends a request for
/// `workspaces`, so is refused too.
fn sends_request_to_default_target(args: &[String]) -> bool {
    let mut positionals: Vec<&str> = Vec::new();
    let mut args = args.iter().map(String::as_str);
    while let Some(arg) = args.next() {
        if arg == "--" {
            positionals.extend(args.by_ref());
            break;
        }
        if !arg.starts_with('-') || arg == "-" {
            positionals.push(arg);
            continue;
        }
        let (flag, inline_value) = match arg.split_once('=') {
            Some((flag, _)) => (flag, true),
            None => (arg, false),
        };
        if flag == "--base-url" || LOCAL_ONLY_FLAGS.contains(&flag) {
            return false;
        }
        // `-pNAME`: a short value flag with its value attached.
        let attached_short_value = SHORT_VALUE_FLAGS
            .iter()
            .any(|short| flag.len() > short.len() && flag.starts_with(short));
        if VALUE_FLAGS.contains(&flag) || SHORT_VALUE_FLAGS.contains(&flag) {
            if !inline_value {
                let _ = args.next();
            }
            continue;
        }
        if attached_short_value || BOOLEAN_FLAGS.contains(&flag) {
            continue;
        }
        // An unknown flag before the command may take a value that would
        // then read as the command, so it cannot be classified. After the
        // command it belongs to that command, and only the command matters.
        if positionals.is_empty() {
            return true;
        }
    }
    !matches!(
        positionals.as_slice(),
        [] | ["help" | "completion" | "man" | "errors", ..] | ["auth", "logout" | "status", ..]
    )
}

/// The stderr line naming the ignored keys meant for this CLI, each once, or
/// `None` when there are none.
///
/// A project's `.env` usually holds keys for other programs, such as
/// `DATABASE_URL`; naming those on every run would be noise, so the line
/// lists only ignored `HEDRA_` keys and the keys in [`REPORTED_FOREIGN_KEYS`].
fn ignored_keys_line(ignored: &[String]) -> Option<String> {
    let mut keys: Vec<&str> = Vec::new();
    for key in ignored
        .iter()
        .map(String::as_str)
        .filter(|key| is_reported(key))
    {
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    if keys.is_empty() {
        return None;
    }
    Some(format!(
        "hedra-cli: a .env file cannot set these, so they were ignored: {}. \
         If you set them yourself, set them in your shell.",
        keys.join(", ")
    ))
}

fn is_reported(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    key.starts_with("HEDRA_") || REPORTED_FOREIGN_KEYS.contains(&key.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Keys a `.env` must not set: keys outside the allow-list, including
    /// the proxy, trust-store and pager variables, a prefix that only looks
    /// like the vendor's, `HEDRA_` keys that are not on the list (known or
    /// not), and the login endpoint overrides, in several cases each.
    const DENIED: [&str; 42] = [
        "SOME_LIBRARY_SETTING",
        "some_library_setting",
        "Another_Tool_Option",
        "DATABASE_URL",
        "RUST_LOG",
        "PATH",
        "BROWSER",
        "OPENSSL_CONF",
        "LESSOPEN",
        "PAGER",
        "pager",
        "HTTPS_PROXY",
        "https_proxy",
        "Https_Proxy",
        "All_Proxy",
        "no_proxy",
        "SSL_CERT_FILE",
        "ssl_cert_dir",
        "CURL_CA_BUNDLE",
        "Requests_Ca_Bundle",
        "HEDRAX_API_KEY",
        "HEDRA_API_KEY_FILE",
        "HEDRA_ENVIRONMENT",
        "HEDRA_SOMETHING",
        "HEDRA_CLI_BASE_URL",
        "Hedra_Cli_Base_Url",
        "hedra_cli_insecure",
        "HEDRA_CLI_INSECURE_SKIP_VERIFY",
        "HEDRA_CLI_PROXY",
        "HEDRA_CLI_NO_PROXY",
        "HEDRA_CLI_PAGER",
        "HEDRA_CLI_CA_BUNDLE",
        "HEDRA_CLI_EXTRA_CA_CERTS",
        "HEDRA_CLI_ALLOW_CROSS_HOST_REDIRECTS",
        "HEDRA_CLI_ALLOW_CROSS_HOST_PAGINATION",
        "hedra_cli_log_file",
        "HEDRA_CLI_LOG_FILE_DIR_HINT",
        "HEDRA_CLI_COMPLETE",
        "HEDRA_CLI_PROFILE",
        "HEDRA_AUTH_AUTHORIZE_URL",
        "hedra_auth_token_url",
        "HEDRA_AUTH_SOMETHING_ELSE",
    ];

    /// Keys a `.env` may set: credentials, the target environment and
    /// preferences, in any case.
    const ALLOWED: [&str; 12] = [
        "HEDRA_API_KEY",
        "Hedra_Api_Key",
        "HEDRA_ENV",
        "hedra_env",
        "HEDRA_CLI_OUTPUT",
        "HEDRA_CLI_LOG",
        "HEDRA_CLI_RETRIES",
        "HEDRA_CLI_TIMEOUT_SECS",
        "Hedra_Cli_Connect_Timeout_Secs",
        "HEDRA_CLI_USER_AGENT_SUFFIX",
        "NO_COLOR",
        "no_color",
    ];

    /// The current values of `keys`, which are then removed from the
    /// environment, for [`restore_env`].
    fn take_env(keys: &[&str]) -> Vec<(String, Option<std::ffi::OsString>)> {
        keys.iter()
            .map(|key| {
                let value = std::env::var_os(key);
                std::env::remove_var(key);
                (key.to_string(), value)
            })
            .collect()
    }

    fn restore_env(saved: Vec<(String, Option<std::ffi::OsString>)>) {
        for (key, value) in saved {
            match value {
                Some(value) => std::env::set_var(key, value),
                None => std::env::remove_var(key),
            }
        }
    }

    #[test]
    fn denied_keys_are_refused() {
        for key in DENIED {
            assert!(!key_is_allowed(key), "{key} must not be settable from .env");
        }
    }

    #[test]
    fn credentials_and_preferences_are_allowed() {
        for key in ALLOWED {
            assert!(key_is_allowed(key), "{key} must be settable from .env");
        }
    }

    #[test]
    #[serial_test::serial]
    fn the_loader_sets_allowed_keys_and_refuses_the_rest() {
        const ALLOWED_KEY: &str = "HEDRA_CLI_CONNECT_TIMEOUT_SECS";
        const EXPORTED_KEY: &str = "HEDRA_CLI_TIMEOUT_SECS";
        let saved = take_env(&[ALLOWED_KEY, EXPORTED_KEY]);
        let denied = ["dotenv_load_test_foreign", "Hedra_Cli_Pager"];
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path().canonicalize().unwrap();
        let mut lines = vec![
            format!("{ALLOWED_KEY}=from-dotenv"),
            format!("{EXPORTED_KEY}=from-dotenv"),
        ];
        lines.extend(denied.iter().map(|key| format!("{key}=from-dotenv")));
        std::fs::write(dir.join(".env"), lines.join("\n")).unwrap();
        std::env::set_var(EXPORTED_KEY, "from-shell");

        let saved_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let ignored = load();
        std::env::set_current_dir(saved_cwd).unwrap();

        let allowed_value = std::env::var(ALLOWED_KEY);
        let exported_value = std::env::var(EXPORTED_KEY);
        let denied_values: Vec<_> = denied.iter().map(std::env::var_os).collect();
        restore_env(saved);

        assert_eq!(ignored.refused, denied);
        assert!(ignored.nul_valued.is_empty());
        assert_eq!(allowed_value.as_deref(), Ok("from-dotenv"));
        assert_eq!(exported_value.as_deref(), Ok("from-shell"));
        assert!(
            denied_values.iter().all(Option::is_none),
            "{denied_values:?}"
        );
    }

    #[test]
    #[serial_test::serial]
    fn a_byte_order_mark_does_not_drop_the_first_key() {
        const KEY: &str = "HEDRA_CLI_USER_AGENT_SUFFIX";
        let saved = take_env(&[KEY]);
        let ignored =
            load_from(format!("\u{feff}{KEY}=from-dotenv\nDOTENV_BOM_TEST_FOREIGN=x\n").as_bytes());
        let value = std::env::var(KEY);
        restore_env(saved);
        assert_eq!(value.as_deref(), Ok("from-dotenv"));
        assert_eq!(ignored.refused, ["DOTENV_BOM_TEST_FOREIGN"]);
    }

    #[test]
    #[serial_test::serial]
    fn a_value_holding_a_nul_is_skipped_and_returned() {
        const KEY: &str = "HEDRA_CLI_RETRIES";
        let saved = take_env(&[KEY]);
        let ignored = load_from(format!("{KEY}=a\u{0}b\n").as_bytes());
        let value = std::env::var_os(KEY);
        restore_env(saved);
        assert!(ignored.refused.is_empty());
        assert_eq!(ignored.nul_valued, [KEY]);
        assert!(value.is_none());
    }

    #[test]
    fn the_stderr_line_names_only_keys_meant_for_this_cli_once_each() {
        let ignored: Vec<String> = [
            "DATABASE_URL",
            "https_proxy",
            "HEDRA_CLI_BASE_URL",
            "ssl_cert_dir",
            "https_proxy",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(
            ignored_keys_line(&ignored).as_deref(),
            Some(
                "hedra-cli: a .env file cannot set these, so they were ignored: \
                 https_proxy, HEDRA_CLI_BASE_URL, ssl_cert_dir. \
                 If you set them yourself, set them in your shell."
            )
        );
        assert_eq!(ignored_keys_line(&["DATABASE_URL".to_string()]), None);
        assert_eq!(ignored_keys_line(&[]), None);
    }

    fn args(line: &str) -> Vec<String> {
        line.split_whitespace().map(String::from).collect()
    }

    #[test]
    fn a_refused_base_url_in_any_case_is_an_error() {
        let refused = |keys: &[&str]| IgnoredKeys {
            refused: keys.iter().map(|key| key.to_string()).collect(),
            nul_valued: Vec::new(),
        };
        let list = args("models list");
        let error = base_url_refusal(&refused(&["PAGER", "Hedra_Cli_Base_Url"]), &list, false)
            .expect("a refused base URL is an error");
        assert!(error.contains("sets Hedra_Cli_Base_Url,"), "{error}");
        assert!(error.contains("outside the directory"), "{error}");
        assert_eq!(
            base_url_refusal(&refused(&["HEDRA_CLI_BASE_URL"]), &list, true),
            None,
            "an exported HEDRA_ENV chose the target"
        );
        assert_eq!(
            base_url_refusal(&refused(&["HEDRA_CLI_PROXY"]), &list, false),
            None
        );
        assert_eq!(
            base_url_refusal(&IgnoredKeys::default(), &list, false),
            None
        );
    }

    #[test]
    fn only_commands_that_may_send_a_request_to_the_default_target_are_refused() {
        for line in [
            "models list",
            "--format json models list",
            "--format table jobs get --job-id job_x",
            "jobs get --job-id help",
            "auth login",
            "workspaces list",
            "--dry-run models list",
            "-q --debug models list",
            "--query data models list",
            "-p help models list",
            "--profile help models list",
            "--profile=work models list",
            "-p=work models list",
            "-pwork models list",
            "--some-new-flag help models list",
            "--some-new-flag=x auth status",
            "jobs",
            "workspaces",
        ] {
            assert!(sends_request_to_default_target(&args(line)), "{line}");
        }
        for line in [
            "",
            "--help",
            "-h",
            "models list --help",
            "jobs get -h",
            "--version",
            "-V",
            "help",
            "help models",
            "completion zsh",
            "--format json completion bash",
            "auth logout",
            "auth status",
            "--base-url https://example.test/v3 models list",
            "models list --base-url=https://example.test/v3",
            "--profile p auth status",
            "-p p auth status",
            "--profile=p auth logout",
            "-pp auth status",
            "--format=json --query data help",
            "--human -q --debug --dry-run completion zsh",
            "models list --schema",
            "--spec",
            "--spec-raw",
            "man",
            "errors",
            "auth status --some-command-flag x",
        ] {
            assert!(!sends_request_to_default_target(&args(line)), "{line}");
        }
    }

    #[test]
    #[serial_test::serial]
    fn a_refused_hedra_key_counts_as_exported_only_in_upper_case() {
        const KEY: &str = "hedra_cli_base_url";
        let saved = take_env(&[KEY, "HEDRA_CLI_BASE_URL"]);
        std::env::set_var(KEY, "from-shell");
        let ignored = load_from(format!("{KEY}=from-dotenv\n").as_bytes());
        restore_env(saved);
        assert_eq!(ignored.refused, [KEY]);
    }
}
