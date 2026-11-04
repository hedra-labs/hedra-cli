//! Loading `.env` (ENG-12596).
//!
//! The CLI is routinely run inside repositories its user did not write, and
//! `dotenvy` finds the nearest `.env` in the working directory or a parent.
//! A `.env` that could set any variable could point requests and credentials
//! at another host, add trusted roots, or name a program for the CLI to run,
//! through variables the libraries it links read, which no list of keys to
//! refuse can cover. So a `.env` may set only this CLI's own variables,
//! `HEDRA_*`, and `NO_COLOR`, compared in any case, and of those not the
//! transport, TLS, pager and log-file settings in [`DENIED_CLI_SUFFIXES`] or
//! the login endpoint overrides in [`DENIED_KEYS`]. Credentials and output
//! preferences (`HEDRA_API_KEY`, `HEDRA_ENV`, `HEDRA_CLI_OUTPUT`) still load.
//! A variable the process environment already holds is left as it is, as
//! `dotenvy::dotenv()` does.
//!
//! The runtime has a `.env` loader of its own, called from `CliApp::run`
//! (`load_dotenv_filtered`, src/lib.rs). It admits every key outside a short
//! deny-list, matched in upper case only, and src/lib.rs is generated, so it
//! cannot be changed here. `custom::register` therefore loads `.env` with
//! [`load`] and runs the command itself through `CliApp::try_run_from`, which
//! skips the runtime's loader; see `custom::register`.

use std::io::Write;

/// The prefix of this CLI's own variables.
const VENDOR_PREFIX: &str = "HEDRA_";

/// The prefix of the runtime's settings for this binary (`HEDRA_CLI` from the
/// binary name `hedra-cli`, plus `_`).
const CLI_PREFIX: &str = "HEDRA_CLI_";

/// Keys outside [`VENDOR_PREFIX`] that a `.env` may set.
const ALSO_ALLOWED: [&str; 1] = ["NO_COLOR"];

/// `HEDRA_` keys a `.env` may not set: they choose where `auth login` sends
/// the browser and the authorization code (`auth.rs`, `endpoint_override`).
const DENIED_KEYS: [&str; 2] = ["HEDRA_AUTH_AUTHORIZE_URL", "HEDRA_AUTH_TOKEN_URL"];

/// Suffixes of `HEDRA_CLI_…` settings a `.env` may not set: the runtime's own
/// deny-list (`DOTENV_DENIED_SUFFIXES`, src/lib.rs), plus `_LOG_FILE`, which
/// chooses where logs are written, and, defensively, `_COMPLETE`, which a
/// runtime with dynamic shell completion would read to turn an invocation
/// into a completion request.
const DENIED_CLI_SUFFIXES: [&str; 12] = [
    "BASE_URL",
    "PAGER",
    "PROXY",
    "NO_PROXY",
    "INSECURE",
    "INSECURE_SKIP_VERIFY",
    "CA_BUNDLE",
    "EXTRA_CA_CERTS",
    "ALLOW_CROSS_HOST_REDIRECTS",
    "ALLOW_CROSS_HOST_PAGINATION",
    "LOG_FILE",
    "COMPLETE",
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

/// Whether a `.env` may set `key`.
pub(crate) fn key_is_allowed(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    if ALSO_ALLOWED.contains(&key.as_str()) {
        return true;
    }
    if !key.starts_with(VENDOR_PREFIX) || DENIED_KEYS.contains(&key.as_str()) {
        return false;
    }
    !key.strip_prefix(CLI_PREFIX)
        .is_some_and(|setting| DENIED_CLI_SUFFIXES.contains(&setting))
}

/// Load the nearest `.env`, setting each key [`key_is_allowed`] admits and
/// the process environment does not already hold. Returns the keys it
/// refused, in file order. A missing or unreadable `.env` loads nothing.
pub(crate) fn load() -> Vec<String> {
    let Some(contents) = nearest_dotenv().and_then(|path| std::fs::read(path).ok()) else {
        return Vec::new();
    };
    load_from(&contents)
}

/// [`load`] for the contents of one `.env` file. A value holding a NUL byte,
/// which the environment cannot store, is skipped.
fn load_from(contents: &[u8]) -> Vec<String> {
    // `dotenvy::dotenv()` strips a UTF-8 byte order mark, which Windows
    // editors write, but `dotenvy`'s iterators do not, and the first key
    // would then fail to parse and be dropped.
    let contents = contents.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(contents);
    let mut ignored = Vec::new();
    for (key, value) in dotenvy::from_read_iter(contents).flatten() {
        if std::env::var_os(&key).is_some() {
            continue;
        }
        if !key_is_allowed(&key) {
            ignored.push(key);
            continue;
        }
        if value.contains('\0') {
            // The environment cannot hold a NUL, and `set_var` would panic.
            tracing::warn!(key = %key, "ignoring {key} from .env: its value holds a NUL byte");
            continue;
        }
        // Runs first in `custom::register`, before any other thread starts.
        std::env::set_var(&key, value);
    }
    ignored
}

/// The `.env` file `dotenvy::dotenv()` would load: the first one in the
/// working directory or a parent.
fn nearest_dotenv() -> Option<std::path::PathBuf> {
    let cwd = std::env::current_dir().ok()?;
    cwd.ancestors()
        .map(|dir| dir.join(".env"))
        .find(|path| path.is_file())
}

/// Report the keys [`load`] refused: each as a warning at `HEDRA_CLI_LOG=warn`,
/// and the ones meant for this CLI in one line on stderr.
///
/// Call after logging is initialized, or the warnings are dropped.
pub(crate) fn report_ignored(ignored: &[String]) {
    for key in ignored {
        tracing::warn!(
            key = %key,
            "ignoring {key} from .env: a .env file may set only this CLI's own \
             variables and NO_COLOR, other than its transport, TLS, pager, \
             log-file and login endpoint settings. Export it from your shell instead."
        );
    }
    if let Some(line) = ignored_keys_line(ignored) {
        // Not `eprintln!`, which panics where writing to a closed stderr
        // fails instead of raising SIGPIPE, as on Windows.
        let _ = writeln!(std::io::stderr(), "{line}");
    }
}

/// The stderr line naming the ignored keys meant for this CLI, or `None` when
/// there are none.
///
/// A project's `.env` usually holds keys for other programs, such as
/// `DATABASE_URL`; naming those on every run would be noise, so the line
/// lists only ignored `HEDRA_` keys and the keys in [`REPORTED_FOREIGN_KEYS`].
fn ignored_keys_line(ignored: &[String]) -> Option<String> {
    let keys: Vec<&str> = ignored
        .iter()
        .map(String::as_str)
        .filter(|key| is_reported(key))
        .collect();
    if keys.is_empty() {
        return None;
    }
    Some(format!(
        "hedra-cli: ignored from .env (export them from your shell instead): {}",
        keys.join(", ")
    ))
}

fn is_reported(key: &str) -> bool {
    let key = key.to_ascii_uppercase();
    key.starts_with(VENDOR_PREFIX) || REPORTED_FOREIGN_KEYS.contains(&key.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Keys a `.env` must not set: keys outside the allow-list, including
    /// the proxy, trust-store and pager variables, a prefix that only looks
    /// like the vendor's, the denied `HEDRA_CLI_` settings and the login
    /// endpoint overrides, in several cases each.
    const DENIED: [&str; 36] = [
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
        "HEDRA_CLI_COMPLETE",
        "HEDRA_AUTH_AUTHORIZE_URL",
        "hedra_auth_token_url",
    ];

    /// Keys a `.env` may set: credentials and preferences, in any case, a
    /// `HEDRA_CLI_` key that only starts like a denied setting, and `NO_COLOR`.
    const ALLOWED: [&str; 9] = [
        "HEDRA_API_KEY",
        "Hedra_Api_Key",
        "HEDRA_ENV",
        "HEDRA_CLI_OUTPUT",
        "HEDRA_CLI_LOG",
        "HEDRA_CLI_RETRIES",
        "HEDRA_CLI_LOG_FILE_DIR_HINT",
        "HEDRA_AUTH_SOMETHING_ELSE",
        "NO_COLOR",
    ];

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
        const PREFIX: &str = "HEDRA_DOTENV_LOAD_TEST_";
        let allowed = format!("{PREFIX}ALLOWED");
        let exported = format!("{PREFIX}EXPORTED");
        let denied = ["dotenv_load_test_foreign", "Hedra_Cli_Pager"];
        let dir = tempfile::tempdir().unwrap();
        let dir = dir.path().canonicalize().unwrap();
        let mut lines = vec![
            format!("{allowed}=from-dotenv"),
            format!("{exported}=from-dotenv"),
        ];
        lines.extend(denied.iter().map(|key| format!("{key}=from-dotenv")));
        std::fs::write(dir.join(".env"), lines.join("\n")).unwrap();
        std::env::set_var(&exported, "from-shell");

        let saved_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        let ignored = load();
        std::env::set_current_dir(saved_cwd).unwrap();

        let allowed_value = std::env::var(&allowed);
        let exported_value = std::env::var(&exported);
        let denied_values: Vec<_> = denied.iter().map(std::env::var_os).collect();
        std::env::remove_var(&allowed);
        std::env::remove_var(&exported);

        assert_eq!(ignored, denied);
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
        const KEY: &str = "HEDRA_DOTENV_BOM_TEST";
        let ignored =
            load_from(format!("\u{feff}{KEY}=from-dotenv\nDOTENV_BOM_TEST_FOREIGN=x\n").as_bytes());
        let value = std::env::var(KEY);
        std::env::remove_var(KEY);
        assert_eq!(value.as_deref(), Ok("from-dotenv"));
        assert_eq!(ignored, ["DOTENV_BOM_TEST_FOREIGN"]);
    }

    #[test]
    #[serial_test::serial]
    fn a_value_holding_a_nul_is_skipped() {
        const KEY: &str = "HEDRA_DOTENV_NUL_TEST";
        let ignored = load_from(format!("{KEY}=a\u{0}b\n").as_bytes());
        let value = std::env::var_os(KEY);
        assert!(ignored.is_empty());
        assert!(value.is_none());
    }

    #[test]
    fn the_stderr_line_names_only_keys_meant_for_this_cli() {
        let ignored: Vec<String> = [
            "DATABASE_URL",
            "https_proxy",
            "HEDRA_CLI_BASE_URL",
            "ssl_cert_dir",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        assert_eq!(
            ignored_keys_line(&ignored).as_deref(),
            Some(
                "hedra-cli: ignored from .env (export them from your shell instead): \
                 https_proxy, HEDRA_CLI_BASE_URL, ssl_cert_dir"
            )
        );
        assert_eq!(ignored_keys_line(&["DATABASE_URL".to_string()]), None);
        assert_eq!(ignored_keys_line(&[]), None);
    }
}
