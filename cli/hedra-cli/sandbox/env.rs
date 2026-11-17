//! The workspace's environment variables (`sandbox env`), beyond what the
//! generated commands do on their own.
//!
//! `ls`, `get` and `rm` are generated from the embedded spec. `set` is
//! replaced here: the generated one takes the value as `--value`, where shell
//! history, process listings and CI logs keep it. This one reads it from piped
//! stdin, one trailing line ending dropped, or from `--from-file` byte for
//! byte, and never from an argument or a terminal.
//!
//! A Secret's value is never printed. The API never returns one, and
//! [`redact_secret_values`] drops a `value` from any Secret an answer to the
//! generated commands carries anyway. `set` prints only the metadata the API
//! answers with, its `--dry-run` shows the value as redacted, and its request
//! is not dumped by `--debug`, whose redaction does not know the `value` key.
//!
//! [`check_create_env`] checks `sandbox create`'s `env` before any request:
//! each value is a literal string, or `{"ref": "NAME"}` naming a saved
//! variable, under the same key (`MODEL_URL: {ref: MODEL_URL}`) or another one
//! (`OPENAI_API_KEY: {ref: OPENAI_KEY}`). Whether a GPU model's pools take a
//! Secret at all is the API's to say: the generated `sandbox gpus` prints its
//! `secrets_supported`, and a create refused for it names that field.

use std::io::{IsTerminal, Read};
use std::path::Path;

use clap::{Arg, ArgAction, ArgMatches, Command};
use fern_cli_sdk::error::{api_error_from_body, CliError};
use fern_cli_sdk::formatter::{OutputFormat, OutputPipeline};
use fern_cli_sdk::openapi::discovery::RestMethod;
use fern_cli_sdk::openapi::AppContext;
use fern_cli_sdk::sdk_executor::SdkRequestExecutor;
use serde_json::{json, Map, Value};

/// The generated group `set` is grafted into.
pub(crate) const GROUP: [&str; 2] = ["sandbox", "env"];

/// The collection's path under the API base.
const COLLECTION: &str = "/sandboxes/environment-variables";

/// The API's bounds on a value, in bytes of UTF-8 (diffusion's
/// `MAX_ENV_VAR_VALUE_BYTES` and `MAX_SECRET_VALUE_BYTES`).
const MAX_CONFIG_VALUE_BYTES: usize = 64 * 1024;
const MAX_SECRET_VALUE_BYTES: usize = 16 * 1024;

/// What `--dry-run` shows in place of the value.
const REDACTED: &str = "<redacted>";

const SET_EXAMPLES: &str = "\
Examples:
  # A Secret, piped: one trailing newline is dropped
  printf '%s' \"$OPENAI_API_KEY\" | hedra-cli sandbox env set --name OPENAI_KEY --type secret

  # A Config, from a file, byte for byte
  hedra-cli sandbox env set --name MODEL_URL --type config --from-file model_url.txt

  # An empty Config
  hedra-cli sandbox env set --name EXTRA_ARGS --type config --from-file /dev/null

  # A sandbox reads saved variables by reference, under the same name or another
  hedra-cli sandbox create --name trainer --image ghcr.io/acme/trainer:1 --gpu h100:1 \\
    --ttl-seconds 3600 \\
    --env '{\"LOG_LEVEL\": \"info\", \"MODEL_URL\": {\"ref\": \"MODEL_URL\"}, \"OPENAI_API_KEY\": {\"ref\": \"OPENAI_KEY\"}}'";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Config,
    Secret,
}

impl Kind {
    fn parse(raw: &str) -> Option<Self> {
        match raw {
            "config" => Some(Kind::Config),
            "secret" => Some(Kind::Secret),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Kind::Config => "config",
            Kind::Secret => "secret",
        }
    }

    fn limit(self) -> usize {
        match self {
            Kind::Config => MAX_CONFIG_VALUE_BYTES,
            Kind::Secret => MAX_SECRET_VALUE_BYTES,
        }
    }
}

/// Where the value comes from.
enum Source<'a> {
    Stdin,
    File(&'a Path),
}

/// The value to save. Not `Debug`, so no `{:?}` or panic message can print it.
struct SavedValue(String);

pub(crate) fn set_command() -> Command {
    Command::new("set")
        .about("Save a Config or Secret variable, its value piped on stdin or read from --from-file")
        .long_about(
            "Save a workspace environment variable: a Config, whose value any member can read \
             back with `sandbox env get`, or a Secret, whose value is never returned. 201 for a \
             new name, 200 when it replaces the value of a name the workspace holds as the same \
             type; a name held as the other type is refused, so delete it first.\n\n\
             The value is piped on stdin, with one trailing line ending (\\n or \\r\\n) dropped, \
             or read from --from-file byte for byte. It is never taken from an argument, where \
             shell history and process listings keep it, or typed at a terminal. An empty value \
             on stdin is refused, so a run with nothing piped cannot blank a saved Config: save \
             an empty Config with --from-file /dev/null. A Secret is 1 byte to 16 KiB of UTF-8, \
             a Config at most 64 KiB, and neither may hold a NUL.\n\n\
             A sandbox reads a variable only when its create names it in `env` as \
             {\"ref\": \"NAME\"}, when it starts: a running sandbox keeps the value it started \
             with, and a new or restarted one reads the saved value. A create that references \
             a Secret can be refused on a GPU model whose `secrets_supported` is false in \
             `hedra-cli sandbox gpus`; a Config reference works on every model.",
        )
        .after_long_help(SET_EXAMPLES)
        .arg(
            Arg::new("name")
                .long("name")
                .value_name("NAME")
                .required(true)
                .help(
                    "The variable's name, which is also its name in the sandbox: uppercase \
                     letters, digits and underscores, not starting with a digit",
                ),
        )
        .arg(
            Arg::new("type")
                .long("type")
                .value_name("TYPE")
                .required(true)
                .value_parser(["config", "secret"])
                .help("`config` for a value members can read back, `secret` for one that is never returned"),
        )
        .arg(
            Arg::new("from-file")
                .long("from-file")
                .value_name("PATH")
                .help("Read the value from this file, byte for byte, instead of stdin"),
        )
        // Declared only to refuse them without clap quoting what was typed,
        // which may be the value.
        .arg(
            Arg::new("value")
                .long("value")
                .num_args(0..=1)
                .action(ArgAction::Set)
                .hide(true),
        )
        .arg(
            Arg::new("stray")
                .num_args(1..)
                .action(ArgAction::Append)
                .hide(true),
        )
}

/// `sandbox env set`.
pub(crate) fn set(matches: &ArgMatches, ctx: &dyn std::any::Any) -> Result<(), CliError> {
    let app = ctx.downcast_ref::<AppContext>().ok_or_else(|| {
        CliError::Validation("sandbox env set: unexpected binding context type".into())
    })?;
    if matches.value_source("value").is_some() {
        return Err(CliError::Validation(
            "`sandbox env set` takes no value on the command line, where shell history and \
             process listings keep it. Pipe it on stdin or pass --from-file PATH."
                .into(),
        ));
    }
    if matches.value_source("stray").is_some() {
        return Err(CliError::Validation(
            "`sandbox env set` takes no positional arguments: name the variable with --name, and \
             pipe its value on stdin or pass --from-file PATH."
                .into(),
        ));
    }
    let name = matches
        .get_one::<String>("name")
        .expect("--name is required");
    check_name(name)?;
    let kind = Kind::parse(
        matches
            .get_one::<String>("type")
            .expect("--type is required"),
    )
    .expect("clap admits only config and secret");
    let value = match matches.get_one::<String>("from-file") {
        Some(path) => read_value(Source::File(Path::new(path)), kind, std::io::empty(), false)?,
        None => {
            let stdin = std::io::stdin();
            let terminal = stdin.is_terminal();
            read_value(Source::Stdin, kind, stdin.lock(), terminal)?
        }
    };

    let pipeline = OutputPipeline::from_matches(matches, app.http_config().name())
        .map_err(|e| CliError::Validation(e.to_string()))?;
    if flag(matches, "dry-run") {
        let preview = json!({
            "dry_run": true,
            "method": "POST",
            "url": format!("{}{COLLECTION}", app.effective_base_url().trim_end_matches('/')),
            "content_type": "application/json",
            "body": {"name": name, "type": kind.as_str(), "value": REDACTED},
        });
        return emit(&pipeline, &preview);
    }

    let body = serde_json::to_vec(&json!({"name": name, "type": kind.as_str(), "value": value.0}))
        .map_err(|e| CliError::Other(anyhow::anyhow!("could not encode the request: {e}")))?;
    let mut answer = run_async(post(app, body, flag(matches, "debug")))?;
    redact(&mut answer);
    emit(&pipeline, &answer)?;
    if pipeline.format == OutputFormat::Table && !flag(matches, "quiet") {
        eprintln!(
            "Running sandboxes keep the value they started with; a new or restarted sandbox \
             reads the saved one."
        );
    }
    Ok(())
}

/// POST the encoded body through the CLI's executor (auth, global headers,
/// retries) and read the answer.
async fn post(app: &AppContext, body: Vec<u8>, debug: bool) -> Result<Value, CliError> {
    let executor = app.build_sdk_executor();
    // `--debug` dumps the request body, and its redaction does not know the
    // `value` key, so this request is sent without the dump.
    let executor = if debug {
        eprintln!(
            "--debug: `sandbox env set` does not print its request, which carries the value."
        );
        match std::sync::Arc::try_unwrap(executor) {
            Ok(executor) => std::sync::Arc::new(executor.with_debug(false, Vec::new())),
            Err(_) => {
                return Err(CliError::Other(anyhow::anyhow!(
                    "could not build a request executor without --debug"
                )))
            }
        }
    } else {
        executor
    };
    let base = super::api::base_url(app.base_url_override(), &app.effective_base_url());
    let request = reqwest::Client::new()
        .post(format!("{base}{COLLECTION}"))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body)
        .build()
        .map_err(|e| CliError::Other(anyhow::anyhow!("could not build the request: {e}")))?;
    let response = SdkRequestExecutor::execute(&*executor, request)
        .await
        .map_err(|e| e.into_cli_error())?;
    let status = response.status();
    let bytes = response
        .bytes()
        .await
        .map_err(|e| CliError::Network(format!("could not read Hedra's answer: {e}")))?;
    // The executor hands back an error status as an answer; report it the way
    // the generated commands do. The API never quotes the value in one.
    if !status.is_success() {
        return Err(api_error_from_body(
            status.as_u16(),
            &String::from_utf8_lossy(&bytes),
        ));
    }
    serde_json::from_slice(&bytes).map_err(|e| {
        CliError::Other(anyhow::anyhow!(
            "Hedra sent an answer hedra-cli could not read: {e}"
        ))
    })
}

fn emit(pipeline: &OutputPipeline, value: &Value) -> Result<(), CliError> {
    let mut out = std::io::stdout().lock();
    pipeline
        .emit(&mut out, value, false, true)
        .map_err(|e| CliError::Other(anyhow::anyhow!("Failed to write output: {e}")))
}

fn flag(matches: &ArgMatches, id: &str) -> bool {
    matches
        .try_get_one::<bool>(id)
        .ok()
        .flatten()
        .copied()
        .unwrap_or(false)
}

fn run_async<T>(fut: impl std::future::Future<Output = T>) -> T {
    tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(fut))
}

/// The characters of every variable's name. The API also bounds the length
/// and reserves some names, and answers for those itself.
fn check_name(name: &str) -> Result<(), CliError> {
    let mut chars = name.chars();
    let first_ok = chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_');
    if first_ok && chars.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Ok(());
    }
    // Not quoted back: it may be the value, passed by mistake for the name.
    Err(CliError::Validation(
        "--name is not a variable name: letters, digits and _, not starting with a digit".into(),
    ))
}

/// Read the value from `source`: a file byte for byte, or `stdin` with one
/// trailing line ending dropped; never from a terminal. No refusal quotes the
/// value or the file's path, which may be a token passed by mistake.
fn read_value(
    source: Source<'_>,
    kind: Kind,
    stdin: impl Read,
    stdin_is_terminal: bool,
) -> Result<SavedValue, CliError> {
    let limit = kind.limit();
    // One byte past the largest input that can be accepted, a value of
    // `limit` bytes and a CRLF, so a value cut short here is still too long
    // after the trim and refused, never saved truncated.
    let cap = (limit + 3) as u64;
    let mut raw = Vec::new();
    let from_file = matches!(source, Source::File(_));
    match source {
        Source::File(path) => {
            let file = std::fs::File::open(path).map_err(|e| {
                CliError::Validation(format!("could not open the value file: {}", e.kind()))
            })?;
            file.take(cap).read_to_end(&mut raw).map_err(|e| {
                CliError::Validation(format!("could not read the value file: {}", e.kind()))
            })?;
        }
        Source::Stdin => {
            if stdin_is_terminal {
                return Err(CliError::Validation(
                    "pipe the value on stdin or pass --from-file PATH; a value typed at a terminal \
                     is not accepted"
                        .into(),
                ));
            }
            stdin.take(cap).read_to_end(&mut raw).map_err(|e| {
                CliError::Validation(format!("could not read the value on stdin: {}", e.kind()))
            })?;
            if raw.ends_with(b"\r\n") {
                raw.truncate(raw.len() - 2);
            } else if raw.ends_with(b"\n") {
                raw.truncate(raw.len() - 1);
            }
        }
    }
    if raw.len() > limit {
        return Err(CliError::Validation(format!(
            "a {} value must be at most {} bytes",
            kind.as_str(),
            limit
        )));
    }
    if raw.is_empty() {
        return Err(CliError::Validation(match (kind, from_file) {
            (Kind::Secret, _) => "a secret value must not be empty".to_string(),
            (Kind::Config, false) => "a config value on stdin must not be empty; save an empty \
                                      config with --from-file /dev/null"
                .to_string(),
            (Kind::Config, true) => return Ok(SavedValue(String::new())),
        }));
    }
    if raw.contains(&0) {
        return Err(CliError::Validation(
            "the value must not contain a NUL byte".into(),
        ));
    }
    String::from_utf8(raw)
        .map(SavedValue)
        .map_err(|_| CliError::Validation("the value must be UTF-8".into()))
}

/// Drop `value` from every Secret in an answer. The API never sends one; this
/// keeps a malformed answer from printing it. Registered for every output
/// format; `--format raw` and `--format http` print the server's bytes
/// before any hook runs.
pub(crate) async fn redact_secret_values(mut v: Value, _: Vec<String>) -> Result<Value, CliError> {
    redact(&mut v);
    Ok(v)
}

fn redact(v: &mut Value) {
    match v {
        Value::Object(fields) => {
            if fields.get("type").and_then(Value::as_str) == Some("secret") {
                fields.remove("value");
            }
            fields.values_mut().for_each(redact);
        }
        Value::Array(items) => items.iter_mut().for_each(redact),
        _ => {}
    }
}

/// Refuse a `sandbox create` whose `env` holds a value that is neither a
/// string nor `{"ref": "NAME"}`, before the request is built. The key is named;
/// the value is not quoted, since a literal may be a secret. The API checks
/// the rest: the name rules, that the variable exists, and the reference count.
pub(crate) fn check_create_env(
    method: &RestMethod,
    params: &mut Map<String, Value>,
) -> Result<(), CliError> {
    if !method.http_method.eq_ignore_ascii_case("POST")
        || method.path.trim_matches('/') != "sandboxes"
    {
        return Ok(());
    }
    let Some(Value::Object(env)) = params.get("env") else {
        return Ok(());
    };
    for (key, value) in env {
        if !is_env_value(value) {
            return Err(CliError::Validation(format!(
                "--env {key:?}: each value is a string, or {{\"ref\": \"NAME\"}} naming a saved \
                 variable (`sandbox env ls` lists them), e.g. \
                 --env '{{\"LOG_LEVEL\": \"info\", \"OPENAI_API_KEY\": {{\"ref\": \"OPENAI_KEY\"}}}}'"
            )));
        }
    }
    Ok(())
}

fn is_env_value(value: &Value) -> bool {
    match value {
        Value::String(_) => true,
        Value::Object(fields) => {
            fields.len() == 1 && fields.get("ref").is_some_and(Value::is_string)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stdin(bytes: &[u8], kind: Kind) -> Result<String, String> {
        read_value(Source::Stdin, kind, bytes, false)
            .map(|v| v.0)
            .map_err(|e| e.to_string())
    }

    fn file(bytes: &[u8], kind: Kind) -> Result<String, String> {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("value");
        std::fs::write(&path, bytes).unwrap();
        read_value(Source::File(&path), kind, std::io::empty(), false)
            .map(|v| v.0)
            .map_err(|e| e.to_string())
    }

    #[test]
    fn stdin_drops_one_trailing_line_ending() {
        assert_eq!(stdin(b"abc\n", Kind::Secret).unwrap(), "abc");
        assert_eq!(stdin(b"abc\r\n", Kind::Secret).unwrap(), "abc");
        assert_eq!(stdin(b"abc\n\n", Kind::Secret).unwrap(), "abc\n");
        assert_eq!(stdin(b"abc", Kind::Config).unwrap(), "abc");
    }

    #[test]
    fn a_file_is_read_byte_for_byte() {
        assert_eq!(file(b"abc\n", Kind::Secret).unwrap(), "abc\n");
        assert_eq!(file(b"", Kind::Config).unwrap(), "");
    }

    #[test]
    fn empty_values_are_refused_except_a_config_from_a_file() {
        assert!(stdin(b"", Kind::Config)
            .unwrap_err()
            .contains("--from-file /dev/null"));
        assert!(stdin(b"\n", Kind::Config).is_err());
        assert!(stdin(b"", Kind::Secret)
            .unwrap_err()
            .contains("must not be empty"));
        assert!(file(b"", Kind::Secret)
            .unwrap_err()
            .contains("must not be empty"));
    }

    #[test]
    fn a_terminal_is_refused() {
        let err = read_value(Source::Stdin, Kind::Secret, &b"typed"[..], true)
            .err()
            .unwrap();
        assert!(err.to_string().contains("terminal"));
    }

    #[test]
    fn values_are_bounded_by_type() {
        let secret = vec![b'a'; MAX_SECRET_VALUE_BYTES];
        assert!(stdin(&secret, Kind::Secret).is_ok());
        let mut over = secret.clone();
        over.push(b'a');
        assert!(stdin(&over, Kind::Secret)
            .unwrap_err()
            .contains("at most 16384 bytes"));
        assert!(stdin(&over, Kind::Config).is_ok());
        let config = vec![b'a'; MAX_CONFIG_VALUE_BYTES + 1];
        assert!(file(&config, Kind::Config)
            .unwrap_err()
            .contains("at most 65536 bytes"));
        // A long input is cut short past the limit, never saved truncated.
        let huge = vec![b'a'; MAX_CONFIG_VALUE_BYTES * 4];
        assert!(stdin(&huge, Kind::Config).is_err());
    }

    #[test]
    fn nul_and_non_utf8_are_refused_without_quoting_the_value() {
        let err = stdin(b"tok\0en", Kind::Secret).unwrap_err();
        assert!(err.contains("NUL") && !err.contains("tok"), "{err}");
        let err = stdin(b"tok\xffen", Kind::Secret).unwrap_err();
        assert!(err.contains("UTF-8") && !err.contains("tok"), "{err}");
    }

    #[test]
    fn a_missing_file_is_refused_without_its_path() {
        let err = read_value(
            Source::File(Path::new("/nonexistent/sk-live-123")),
            Kind::Secret,
            std::io::empty(),
            false,
        )
        .err()
        .unwrap()
        .to_string();
        assert!(!err.contains("sk-live-123"), "{err}");
    }

    #[test]
    fn names_are_checked_without_being_quoted() {
        for good in ["OPENAI_KEY", "_X", "a1", "Model_Url"] {
            assert!(check_name(good).is_ok(), "{good}");
        }
        for bad in ["", "1X", "A-B", "../keys", "sk live"] {
            let err = check_name(bad).err().unwrap().to_string();
            assert!(bad.is_empty() || !err.contains(bad), "{err}");
        }
    }

    #[test]
    fn redaction_drops_a_secret_value_anywhere_and_keeps_a_config_value() {
        let mut v = json!({
            "name": "K", "type": "secret", "value": "leak",
            "data": [{"name": "C", "type": "config", "value": "keep"},
                     {"name": "S", "type": "secret", "value": "leak"}]
        });
        redact(&mut v);
        assert!(!v.to_string().contains("leak"), "{v}");
        assert!(v.to_string().contains("keep"), "{v}");
    }

    #[test]
    fn env_values_are_literals_or_single_field_references() {
        for ok in [json!("x"), json!(""), json!({"ref": "A"})] {
            assert!(is_env_value(&ok), "{ok}");
        }
        for bad in [
            json!(5),
            json!(null),
            json!(true),
            json!(["x"]),
            json!({}),
            json!({"ref": 5}),
            json!({"ref": "A", "x": 1}),
            json!({"name": "A"}),
        ] {
            assert!(!is_env_value(&bad), "{bad}");
        }
    }
}
