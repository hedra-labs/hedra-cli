//! Checks on the generated code `custom::register` depends on (ENG-12596).
//!
//! `register` runs the command itself through `CliApp::try_run_from` and
//! exits, so the runtime's own `.env` loader in `CliApp::run` never runs. That
//! holds only while the generated `main.rs` calls `register` and then `run`,
//! and while `run`'s setup is the one `register` copies. A regeneration that
//! changes either fails here, so the copy in `register` is checked again
//! before the regeneration lands. The same goes for the runtime's own `.env`
//! deny-list, which `dotenv.rs` copies.
//!
//! Hand-written and .fernignore-protected (the `tests/` entry).

const MAIN_RS: &str = include_str!("../cli/hedra-cli/main.rs");
const APP_RS: &str = include_str!("../src/app.rs");
const LIB_RS: &str = include_str!("../src/lib.rs");
const DOTENV_RS: &str = include_str!("../cli/hedra-cli/dotenv.rs");

/// `source`'s code lines, trimmed, without comments or blank lines.
fn code_lines(source: &str) -> Vec<&str> {
    source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect()
}

/// The code lines of the body of the method whose signature line starts with
/// `signature`, up to its closing brace at the method's indentation.
fn method_body(signature: &str) -> Vec<&'static str> {
    let start = APP_RS
        .find(signature)
        .unwrap_or_else(|| panic!("src/app.rs has no `{signature}`"));
    let rest = &APP_RS[start..];
    let end = rest
        .find("\n    }\n")
        .unwrap_or_else(|| panic!("`{signature}` has no closing brace"));
    code_lines(&rest[..end])
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

#[test]
fn only_run_with_args_loads_dotenv() {
    assert!(
        !MAIN_RS.contains("dotenv"),
        "main.rs loads .env before custom::register"
    );
    assert_eq!(
        APP_RS.matches("load_dotenv_filtered(").count(),
        1,
        "src/app.rs loads .env outside run_with_args"
    );
    assert!(method_body("pub fn run_with_args<I, T>")
        .iter()
        .any(|line| line.contains("load_dotenv_filtered(")));
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
    assert_eq!(
        method_body("pub fn run_with_args<I, T>"),
        [
            "pub fn run_with_args<I, T>(mut self, args: I)",
            "where",
            "I: IntoIterator<Item = T>,",
            "T: Into<std::ffi::OsString>,",
            "{",
            "crate::reset_sigpipe();",
            "let ignored_dotenv_keys = crate::load_dotenv_filtered(&self.name);",
            "crate::init_logging(&self.name);",
            "crate::warn_ignored_dotenv_keys(&ignored_dotenv_keys);",
            "let args: Vec<std::ffi::OsString> = args.into_iter().map(Into::into).collect();",
            "let prepare_error = self.prepare(&args);",
            "let rt = tokio::runtime::Runtime::new().expect(\"Failed to create tokio runtime\");",
            "let mut out = std::io::stdout().lock();",
            "let exit = rt.block_on(self.run_inner(args, prepare_error, &mut out));",
            "drop(out);",
            "std::process::exit(exit);",
        ],
    );
    assert_eq!(
        method_body("pub fn try_run_from<I, T>"),
        [
            "pub fn try_run_from<I, T>(mut self, args: I) -> i32",
            "where",
            "I: IntoIterator<Item = T>,",
            "T: Into<std::ffi::OsString>,",
            "{",
            "let args: Vec<std::ffi::OsString> = args.into_iter().map(Into::into).collect();",
            "let prepare_error = self.prepare(&args);",
            "let rt = tokio::runtime::Runtime::new().expect(\"Failed to create tokio runtime\");",
            "let mut out = std::io::stdout().lock();",
            "rt.block_on(self.run_inner(args, prepare_error, &mut out))",
        ],
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
    let ours = string_constant(DOTENV_RS, "DENIED_CLI_SUFFIXES");
    for suffix in string_constant(LIB_RS, "DOTENV_DENIED_SUFFIXES") {
        let setting = suffix.trim_start_matches('_');
        assert!(
            ours.contains(&setting),
            "the runtime refuses `<PREFIX>{suffix}` from .env and dotenv.rs does not"
        );
    }
}
