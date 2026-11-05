//! Checks on the generated code `custom::register` depends on.
//!
//! `register` runs the command itself through `CliApp::try_run_from` and
//! exits, so the runtime's own `.env` loader in `CliApp::run` never runs. That
//! holds only while the generated `main.rs` calls `register` and then `run`,
//! while `run`'s setup is the one `register` copies, and while nothing else
//! in the runtime loads `.env`. A regeneration that changes any of these
//! fails here, so the copy in `register` is checked again before the
//! regeneration lands. The same goes for the settings the runtime reads from
//! the environment: each must be one `dotenv.rs` allows from a `.env` or one
//! this file lists as refused on purpose.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry).

use std::path::{Path, PathBuf};

const MAIN_RS: &str = include_str!("../cli/hedra-cli/main.rs");
const APP_RS: &str = include_str!("../src/app.rs");
const LIB_RS: &str = include_str!("../src/lib.rs");
const USER_AGENT_RS: &str = include_str!("../src/user_agent.rs");
const DOTENV_RS: &str = include_str!("../cli/hedra-cli/dotenv.rs");

/// `HEDRA_CLI_…` settings the runtime reads that a `.env` may not set, by
/// the name after the prefix. Everything not in `dotenv.rs`'s
/// `ALLOWED_CLI_SETTINGS` is refused anyway; this list records that each
/// known one was refused on purpose, so a setting a regeneration adds fails
/// [`every_runtime_setting_is_classified`] until someone decides.
const REFUSED_CLI_SETTINGS: [&str; 14] = [
    // Where requests go, through what, and what is trusted.
    "BASE_URL",
    "PROXY",
    "NO_PROXY",
    "INSECURE",
    "INSECURE_SKIP_VERIFY",
    "CA_BUNDLE",
    "EXTRA_CA_CERTS",
    "ALLOW_CROSS_HOST_REDIRECTS",
    "ALLOW_CROSS_HOST_PAGINATION",
    // A program to run, and a directory to write logs to.
    "PAGER",
    "LOG_FILE",
    // Chooses a stored profile, which can carry a base URL. Profiles are not
    // enabled for this binary.
    "PROFILE",
    // Read only for AsyncAPI bindings, which this binary has none of.
    "WS_RESPONSE_TIMEOUT_SECS",
    // Read by a runtime with dynamic shell completion to turn an invocation
    // into a completion request.
    "COMPLETE",
];

/// `source`'s code lines, trimmed, without comments or blank lines.
fn code_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect()
}

/// The lines of the item in `source` whose first line starts with
/// `signature` (after its indentation), through the closing brace at that
/// indentation, with `\r` line endings removed. `None` when there is no
/// such item.
fn item_lines<'a>(source: &'a str, signature: &str) -> Option<Vec<&'a str>> {
    let lines: Vec<&str> = source
        .lines()
        .map(|line| line.trim_end_matches('\r'))
        .collect();
    let start = lines
        .iter()
        .position(|line| line.trim_start().starts_with(signature))?;
    let indent = &lines[start][..lines[start].len() - lines[start].trim_start().len()];
    let closing = format!("{indent}}}");
    let end = start + lines[start..].iter().position(|line| *line == closing)?;
    Some(lines[start..=end].to_vec())
}

/// The code lines of the body of the `src/app.rs` method whose signature line
/// starts with `signature`, up to its closing brace at the method's
/// indentation.
fn method_body(signature: &str) -> Vec<&'static str> {
    let lines = item_lines(APP_RS, signature)
        .unwrap_or_else(|| panic!("src/app.rs has no `{signature}` with a closing brace"));
    // Without the closing brace, which `code_lines` would otherwise keep.
    lines[..lines.len() - 1]
        .iter()
        .flat_map(|line| code_lines(line))
        .collect()
}

/// Every `.rs` file under `dir`, recursively, with its contents.
fn rust_sources(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut sources = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("read a src/ directory") {
            let path = entry.expect("a src/ entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let source = std::fs::read_to_string(&path).expect("read a src/ file");
                sources.push((path, source));
            }
        }
    }
    sources.sort();
    sources
}

fn runtime_sources() -> Vec<(PathBuf, String)> {
    rust_sources(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
}

/// The code lines of `source` holding `needle`, outside comments.
fn code_lines_containing<'a>(source: &'a str, needle: &str) -> Vec<&'a str> {
    code_lines(source)
        .into_iter()
        .filter(|line| line.contains(needle))
        .collect()
}

#[test]
fn main_runs_the_app_right_after_register() {
    let lines = code_lines(MAIN_RS);
    let register = lines
        .iter()
        .position(|line| *line == "let app = custom::register(app);")
        .expect("main.rs calls custom::register");
    assert_eq!(
        &lines[register + 1..],
        ["app.run()", "}"],
        "main.rs does something after custom::register that register's own run would skip"
    );
}

/// `dotenvy` is used only inside `src/lib.rs`'s `load_dotenv_filtered`, and
/// that is called only from `run_with_args`, which `register` skips.
///
/// If a regeneration changes `load_dotenv_filtered`'s signature line, update
/// the `pub fn load_dotenv_filtered(` prefix and the full definition line in
/// `allowed_calls` below to match.
#[test]
fn only_run_with_args_loads_dotenv() {
    assert!(
        !MAIN_RS.contains("dotenv"),
        "main.rs loads .env before custom::register"
    );
    let loader = item_lines(LIB_RS, "pub fn load_dotenv_filtered(")
        .expect("src/lib.rs defines load_dotenv_filtered");
    let loader = loader.join("\n");
    let run_with_args = item_lines(APP_RS, "pub fn run_with_args<I, T>")
        .expect("src/app.rs defines run_with_args")
        .join("\n");
    for (path, source) in runtime_sources() {
        let is_lib = path.ends_with("src/lib.rs");
        let is_app = path.ends_with("src/app.rs");
        let dotenvy_uses = code_lines_containing(&source, "dotenvy::");
        let allowed_uses = if is_lib {
            code_lines_containing(&loader, "dotenvy::")
        } else {
            Vec::new()
        };
        assert_eq!(
            dotenvy_uses,
            allowed_uses,
            "{} uses dotenvy outside load_dotenv_filtered",
            path.display()
        );
        let calls = code_lines_containing(&source, "load_dotenv_filtered(");
        let allowed_calls = if is_lib {
            vec!["pub fn load_dotenv_filtered(cli_name: &str) -> Vec<String> {"]
        } else if is_app {
            code_lines_containing(&run_with_args, "load_dotenv_filtered(")
        } else {
            Vec::new()
        };
        assert_eq!(
            calls,
            allowed_calls,
            "{} loads .env outside run_with_args",
            path.display()
        );
    }
    assert_eq!(
        code_lines_containing(&run_with_args, "load_dotenv_filtered(").len(),
        1,
        "run_with_args no longer loads .env, so register's copy of its setup is stale"
    );
}

#[test]
fn run_is_the_setup_register_copies_then_try_run_from() {
    assert_eq!(
        method_body("pub fn run(self)"),
        [
            "pub fn run(self) {",
            "let args: Vec<std::ffi::OsString> = std::env::args_os().collect();",
            "self.run_with_args(args)",
        ],
    );
    // `run_with_args` must be exactly `register`'s setup followed by
    // `try_run_from`'s pipeline and an exit. Compared with each other rather
    // than with literal text, so a regeneration that changes both pipelines
    // alike (a new `run_inner` argument) passes, and one that adds setup to
    // `run_with_args` alone fails. On a failure: mirror the new setup in
    // `custom::register`, then update the lines below.
    let generic = [
        "where",
        "I: IntoIterator<Item = T>,",
        "T: Into<std::ffi::OsString>,",
        "{",
    ];
    let run_with_args = method_body("pub fn run_with_args<I, T>");
    let try_run_from = method_body("pub fn try_run_from<I, T>");
    assert_eq!(
        run_with_args[0],
        "pub fn run_with_args<I, T>(mut self, args: I)"
    );
    assert_eq!(
        try_run_from[0],
        "pub fn try_run_from<I, T>(mut self, args: I) -> i32"
    );
    assert_eq!(run_with_args[1..5], generic);
    assert_eq!(try_run_from[1..5], generic);
    assert_eq!(
        run_with_args[5..9],
        [
            "crate::reset_sigpipe();",
            "let ignored_dotenv_keys = crate::load_dotenv_filtered(&self.name);",
            "crate::init_logging(&self.name);",
            "crate::warn_ignored_dotenv_keys(&ignored_dotenv_keys);",
        ],
        "run_with_args's setup changed; mirror it in custom::register"
    );
    let (result, pipeline) = try_run_from[5..]
        .split_last()
        .expect("try_run_from has a body");
    let mut expected: Vec<String> = pipeline.iter().map(|line| line.to_string()).collect();
    expected.push(format!("let exit = {result};"));
    expected.push("drop(out);".into());
    expected.push("std::process::exit(exit);".into());
    assert_eq!(
        run_with_args[9..],
        expected,
        "run_with_args no longer runs exactly try_run_from's pipeline after its setup"
    );
}

/// The quoted entries of the `&[&str]` or `[&str; N]` constant `name` in
/// `source`.
fn string_constant(source: &'static str, name: &str) -> Vec<&'static str> {
    let start = source
        .find(&format!("const {name}"))
        .unwrap_or_else(|| panic!("no constant {name}"));
    let body = &source[start..];
    let equals = body.find('=').expect("an initializer");
    let body = &body[equals + body[equals..].find('[').expect("an array")..];
    let body = &body[..body.find("];").expect("the array ends")];
    let entries: Vec<&str> = body.split('"').skip(1).step_by(2).collect();
    assert!(!entries.is_empty(), "{name} has no entries");
    entries
}

#[test]
fn dotenv_refuses_every_setting_the_runtime_refuses() {
    let allowed = string_constant(DOTENV_RS, "ALLOWED_CLI_SETTINGS");
    for suffix in string_constant(LIB_RS, "DOTENV_DENIED_SUFFIXES") {
        let setting = suffix.trim_start_matches('_');
        assert!(
            !allowed.contains(&setting),
            "the runtime refuses `<PREFIX>{suffix}` from .env and dotenv.rs allows it"
        );
    }
}

/// The upper-case setting name (`[A-Z0-9_]`, without a leading `_`) at the
/// start of `text`, or `None` when `text` does not start with one.
fn setting_name_at(text: &str) -> Option<&str> {
    let text = text.strip_prefix('_')?;
    let end = text
        .find(|c: char| !(c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'))
        .unwrap_or(text.len());
    let name = text[..end].trim_end_matches('_');
    name.starts_with(|c: char| c.is_ascii_uppercase())
        .then_some(name)
}

/// The setting names `source` builds from the binary's prefix, by the name
/// after it: `{prefix}_NAME` or `{}_NAME` in a format string, and a `"_NAME"`
/// literal passed as the suffix argument of `scoped(`. Another `"_NAME"`
/// literal, such as a variable name the source reads as it is, does not
/// count. Test modules are skipped.
fn settings_in(source: &str) -> std::collections::BTreeSet<String> {
    let mut settings = std::collections::BTreeSet::new();
    let source = source.replace("\r\n", "\n");
    let source = source
        .split("#[cfg(test)]\nmod ")
        .next()
        .unwrap_or_default();
    for line in code_lines(source) {
        for marker in ["{prefix}", "{}"] {
            for (index, _) in line.match_indices(marker) {
                if let Some(name) = setting_name_at(&line[index + marker.len()..]) {
                    settings.insert(name.to_string());
                }
            }
        }
    }
    // `scoped(` calls are read from the code lines joined, so a call that
    // rustfmt split over lines, or whose prefix argument holds parentheses,
    // still counts.
    let joined = code_lines(source).join(" ");
    for (index, _) in joined.match_indices("scoped(") {
        let Some(arguments) = balanced_arguments(&joined[index + "scoped(".len()..]) else {
            continue;
        };
        let arguments = arguments.trim().trim_end_matches(',').trim_end();
        let Some((_, suffix)) = arguments.rsplit_once(", \"") else {
            continue;
        };
        if let Some(name) = setting_name_at(suffix) {
            if suffix[1 + name.len()..] == *"\"" {
                settings.insert(name.to_string());
            }
        }
    }
    settings
}

/// The text of a call's arguments, from just after its opening parenthesis
/// up to the matching closing one, or `None` when it does not close.
fn balanced_arguments(text: &str) -> Option<&str> {
    let mut depth = 1;
    for (index, c) in text.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&text[..index]);
                }
            }
            _ => {}
        }
    }
    None
}

/// The settings the runtime builds from the binary's prefix: those
/// [`settings_in`] finds in `src/`, the runtime's `.env` deny-list, and the
/// default User-Agent suffix flag.
fn runtime_settings() -> std::collections::BTreeSet<String> {
    let mut settings = std::collections::BTreeSet::new();
    for (_, source) in runtime_sources() {
        settings.extend(settings_in(&source));
    }
    for suffix in string_constant(LIB_RS, "DOTENV_DENIED_SUFFIXES") {
        settings.insert(suffix.trim_start_matches('_').to_string());
    }
    let flag = string_constant_value(USER_AGENT_RS, "DEFAULT_SUFFIX_FLAG");
    settings.insert(flag.to_ascii_uppercase().replace('-', "_"));
    settings
}

#[test]
fn the_setting_scan_counts_only_names_joined_to_the_prefix() {
    let source = "\
let proxy = first_env([scoped(prefix, \"_PROXY\")]);\r
let ca = first_env([scoped(
    cfg.prefix(),
    \"_CA_BUNDLE\",
)]);
let log = format!(\"{prefix}_LOG\");
let output = format!(\"{}_OUTPUT\", name);
let index = std::env::var(\"_CLAP_COMPLETE_INDEX\");
let other = lookup(\"_NOT_A_SETTING\");
// let commented = format!(\"{prefix}_IN_A_COMMENT\");
#[cfg(test)]
mod tests {
    const IN_A_TEST: &str = \"{prefix}_IN_A_TEST\";
}
";
    let expected: std::collections::BTreeSet<String> = ["CA_BUNDLE", "LOG", "OUTPUT", "PROXY"]
        .into_iter()
        .map(String::from)
        .collect();
    assert_eq!(settings_in(source), expected);
}

/// The value of the `&str` constant `name` in `source`.
fn string_constant_value(source: &'static str, name: &str) -> &'static str {
    let start = source
        .find(&format!("const {name}: &str = \""))
        .unwrap_or_else(|| panic!("no &str constant {name}"));
    let value = &source[start..].split('"').nth(1).expect("a string value");
    value
}

#[test]
fn every_runtime_setting_is_classified() {
    let allowed = string_constant(DOTENV_RS, "ALLOWED_CLI_SETTINGS");
    let settings = runtime_settings();
    for expected in ["OUTPUT", "LOG", "BASE_URL", "PROXY", "USER_AGENT_SUFFIX"] {
        assert!(
            settings.contains(expected),
            "the scan of src/ no longer finds `{expected}`; it has stopped matching \
             how the runtime builds setting names: {settings:?}"
        );
    }
    let unclassified: Vec<&String> = settings
        .iter()
        .filter(|setting| {
            !allowed.contains(&setting.as_str())
                && !REFUSED_CLI_SETTINGS.contains(&setting.as_str())
        })
        .collect();
    assert!(
        unclassified.is_empty(),
        "the runtime reads HEDRA_CLI_ settings that are neither allowed from .env \
         (ALLOWED_CLI_SETTINGS, cli/hedra-cli/dotenv.rs) nor refused on purpose \
         (REFUSED_CLI_SETTINGS, here): {unclassified:?}"
    );
    for setting in &allowed {
        assert!(
            settings.contains(*setting),
            "dotenv.rs allows `HEDRA_CLI_{setting}`, which the runtime does not read"
        );
    }
}
