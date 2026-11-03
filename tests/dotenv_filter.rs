//! Tests for the runtime's `.env` deny-list, through `load_dotenv_filtered`.
//!
//! The deny-list lives in the generated `src/lib.rs`, where regeneration would
//! drop any test added beside it, so its tests live here (`tests/` is
//! .fernignore-protected). They fail on the generated tree and pass once
//! `.github/scripts/patch-dotenv-denylist.py` has run, which makes them the
//! oracle for that script's Rust (ENG-12596; delete with the script when
//! ENG-12663 lands upstream).
//!
//! One test in its own file, so the working-directory change it needs runs in
//! a process no other test shares.

/// Keys a `.env` must not set: upper-case keys the generated list does not
/// name, and listed keys in other cases.
const DENIED: [&str; 7] = [
    "SSL_CERT_DIR",
    "ssl_cert_dir",
    "https_proxy",
    "Https_Proxy",
    "All_Proxy",
    "hedra_cli_insecure",
    "Hedra_Cli_Base_Url",
];

/// A key the filter must still let through.
const ALLOWED: &str = "HEDRA_CLI_DOTENV_FILTER_TEST_ALLOWED";

#[test]
fn the_loader_drops_denied_keys_in_any_case_and_keeps_allowed_ones() {
    let dir = tempfile::tempdir().unwrap();
    let dir = dir.path().canonicalize().unwrap();
    let mut lines: Vec<String> = DENIED
        .iter()
        .map(|key| format!("{key}=from-dotenv"))
        .collect();
    lines.push(format!("{ALLOWED}=from-dotenv"));
    std::fs::write(dir.join(".env"), lines.join("\n")).unwrap();
    let unset_before: Vec<&str> = DENIED
        .iter()
        .copied()
        .filter(|key| std::env::var_os(key).is_none())
        .collect();

    let saved_cwd = std::env::current_dir().unwrap();
    std::env::set_current_dir(&dir).unwrap();
    let ignored = fern_cli_sdk::load_dotenv_filtered("hedra-cli");
    std::env::set_current_dir(saved_cwd).unwrap();

    assert_eq!(ignored, DENIED);
    assert_eq!(std::env::var(ALLOWED).as_deref(), Ok("from-dotenv"));
    for key in unset_before {
        assert!(std::env::var_os(key).is_none(), "{key} was set from .env");
    }
}
