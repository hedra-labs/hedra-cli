//! Tests for the runtime's `.env` filter, the allow-list plus the generated
//! deny-lists, through `load_dotenv_filtered`.
//!
//! The filter lives in the generated `src/lib.rs`, where regeneration would
//! drop any test added beside it, so its tests live here (`tests/` is
//! .fernignore-protected). They fail on the generated tree and pass once
//! `.github/scripts/patch-dotenv-filter.py` has run, which makes them the
//! oracle for that script's Rust (ENG-12596). Keep them once ENG-12663 lands
//! upstream: they still test the behaviour.
//!
//! One test in its own file, so the working-directory change it needs runs in
//! a process no other test shares.

/// Keys a `.env` must not set, in several cases each: keys outside the
/// allow-list, including the names the generated deny-list already lists, a
/// prefix that only resembles the vendor's, and the `HEDRA_CLI_` settings the
/// deny-list refuses although they start with `HEDRA_`.
const DENIED: [&str; 31] = [
    // Outside the allow-list.
    "SOME_LIBRARY_SETTING",
    "some_library_setting",
    "Another_Tool_Option",
    "DATABASE_URL",
    "RUST_LOG",
    "rust_log",
    "PATH",
    "random_key",
    // Outside the allow-list and in the generated deny-list.
    "PAGER",
    "pager",
    "HTTPS_PROXY",
    "https_proxy",
    "Https_Proxy",
    "All_Proxy",
    "no_proxy",
    "SSL_CERT_FILE",
    "ssl_cert_file",
    "CURL_CA_BUNDLE",
    "Requests_Ca_Bundle",
    // A prefix that only looks like the vendor's.
    "HEDRAX_API_KEY",
    // Inside the allow-list, refused by the deny-list.
    "HEDRA_CLI_BASE_URL",
    "Hedra_Cli_Base_Url",
    "hedra_cli_insecure",
    "HEDRA_CLI_PROXY",
    "HEDRA_CLI_PAGER",
    "Hedra_Cli_Pager",
    "HEDRA_CLI_CA_BUNDLE",
    "HEDRA_CLI_ALLOW_CROSS_HOST_REDIRECTS",
    "HEDRA_CLI_LOG_FILE",
    "hedra_cli_log_file",
    "Hedra_Cli_Log_File",
];

/// Keys a `.env` may set: the CLI's credentials and preferences, in any case,
/// and `NO_COLOR`.
const ALLOWED: [&str; 8] = [
    "HEDRA_API_KEY",
    "Hedra_Api_Key",
    "HEDRA_ENV",
    "HEDRA_CLI_OUTPUT",
    "HEDRA_CLI_LOG",
    "NO_COLOR",
    "HEDRA_CLI_DOTENV_FILTER_TEST_ALLOWED",
    "HEDRA_CLI_LOG_FILE_DOTENV_FILTER_TEST",
];

#[test]
fn the_loader_drops_denied_keys_in_any_case_and_keeps_allowed_ones() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();
    let mut lines: Vec<String> = DENIED
        .iter()
        .map(|key| format!("{key}=from-dotenv"))
        .collect();
    lines.extend(ALLOWED.iter().map(|key| format!("{key}=from-dotenv")));
    std::fs::write(dir.join(".env"), lines.join("\n")).unwrap();
    let unset_before: Vec<&str> = DENIED
        .iter()
        .copied()
        .filter(|key| std::env::var_os(key).is_none())
        .collect();
    let allowed_unset_before: Vec<&str> = ALLOWED
        .iter()
        .copied()
        .filter(|key| std::env::var_os(key).is_none())
        .collect();

    let saved_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();
    let ignored = fern_cli_sdk::load_dotenv_filtered("hedra-cli");
    std::env::set_current_dir(saved_cwd).unwrap();

    assert_eq!(ignored, DENIED);
    for key in allowed_unset_before {
        assert_eq!(std::env::var(key).as_deref(), Ok("from-dotenv"), "{key}");
    }
    for key in unset_before {
        assert!(std::env::var_os(key).is_none(), "{key} was set from .env");
    }
}
