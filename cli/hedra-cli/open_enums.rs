//! Suggested values for open-enum flags (ENG-11633).
//!
//! An open enum is a string property that lists values without restricting to
//! them: `anyOf: [{type: string, enum: [...]}, {type: string}]`. The v3 TTS
//! `language` input is one — 41 ISO codes are listed, and `Korean` or `pt-BR`
//! are accepted too. The runtime has no notion of this shape, so the codes
//! never reached `--help` or shell completion. This puts them there without
//! restricting the flag:
//!
//! | surface          | effect                                                        |
//! | ---------------- | ------------------------------------------------------------- |
//! | parsing          | any string, unchanged (`Korean`, `pt-BR`, `123` → `"123"`)     |
//! | shell completion | the codes, via clap's `possible_values()`                      |
//! | `--help`         | `[suggested values: …; any other string is also accepted]`     |
//! | `-h`             | unchanged — it doubles as the completion description           |
//!
//! Both spellings are matched. The *typed* one, `{type: string, anyOf: [...]}`,
//! is what fern-config's `type_shared_unions` writes today, and the runtime
//! already type-checks it as a string in `--json` bodies. The *raw* one (no
//! top-level `type`) gets a string flag here, but the runtime's body validator
//! treats it as a multi-branch union and leaves `--json` values unchecked; the
//! API still rejects a non-string.
//!
//! Fields are found from the schema's shape in the embedded spec, never from a
//! field-name list, so a model added later is covered with no edit here.
//!
//! The runtime has no arg-level hook of its own, so this uses the one generic
//! seam carried as a Fern Replay patch on `src/openapi/commands.rs`:
//! [`fern_cli_sdk::openapi::commands::set_arg_transform`], which hands every
//! parameter's clap `Arg` to a transform before it is registered. Everything
//! Hedra-specific lives here, in a `.fernignore`d file.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use clap::builder::{PossibleValue, StringValueParser, TypedValueParser};
use clap::{Arg, Command};
use fern_cli_sdk::openapi::discovery::RestMethod;
use serde_json::Value;

/// How deep to follow nested body objects. The submit bodies put fields one
/// level down (`input.*`); this leaves room.
const MAX_DEPTH: u8 = 8;

/// Open-enum fields per operation: `(lowercase http method, path)` → dotted
/// wire name → suggested values.
type Table = BTreeMap<(String, String), BTreeMap<String, Vec<String>>>;

static TABLE: OnceLock<Table> = OnceLock::new();

/// Install the suggestions for the embedded spec. Called once from
/// `custom::register`.
pub fn install() {
    install_for(|| serde_json::from_str(include_str!("openapi0.json")).unwrap_or(Value::Null));
}

/// Install the suggestions for the spec `load` returns. The spec is read
/// lazily, on the first flag the command builder hands over.
fn install_for(load: impl FnOnce() -> Value + Send + Sync + 'static) {
    let load = std::sync::Mutex::new(Some(load));
    fern_cli_sdk::openapi::commands::set_arg_transform(move |method, wire_name, arg| {
        let table = TABLE.get_or_init(|| {
            let load = load.lock().ok().and_then(|mut l| l.take());
            load.map(|l| open_enum_table(&l())).unwrap_or_default()
        });
        match lookup(table, method, wire_name) {
            Some(values) => suggest(arg, values),
            None => arg,
        }
    });
}

fn lookup<'a>(table: &'a Table, method: &RestMethod, wire_name: &str) -> Option<&'a [String]> {
    let key = (method.http_method.to_ascii_lowercase(), method.path.clone());
    table.get(&key)?.get(wire_name).map(Vec::as_slice)
}

// ---------------------------------------------------------------------------
// The flag
// ---------------------------------------------------------------------------

/// Rewrite an open enum's flag: any string parses, clap is told the values
/// (shell completion reads them from the parser), and the long help lists
/// them in words that do not claim the list is exhaustive.
fn suggest(arg: Arg, values: &[String]) -> Arg {
    let suffix = format!(
        " [suggested values: {}; any other string is also accepted]",
        values.join(", ")
    );
    let long_help = arg
        .get_long_help()
        .or(arg.get_help())
        .map(|h| h.to_string())
        .unwrap_or_default();
    // The raw spelling reaches the builder typeless, as `<VALUE>`; the typed
    // one is already `<STRING>`. Keep any `|null` suffix the builder added.
    let value_names: Vec<String> = arg
        .get_value_names()
        .unwrap_or_default()
        .iter()
        .map(|n| match n.as_str().strip_prefix("VALUE") {
            Some(rest) => format!("STRING{rest}"),
            None => n.to_string(),
        })
        .collect();
    let mut arg = arg
        .value_parser(SuggestedValuesParser::new(values))
        // clap would render `[possible values: …]`, which says the list is
        // complete. Completion scripts ignore this setting.
        .hide_possible_values(true)
        .long_help(format!("{}{suffix}", long_help.trim_end()));
    if !value_names.is_empty() {
        arg = arg.value_names(value_names);
    }
    arg
}

/// Accepts any string, as a plain string flag does, and reports the
/// suggested values to clap. clap enforces `possible_values()` only inside
/// `PossibleValuesParser`, so reporting them here restricts nothing.
#[derive(Clone, Debug)]
struct SuggestedValuesParser(Vec<PossibleValue>);

impl SuggestedValuesParser {
    fn new(values: &[String]) -> Self {
        Self(values.iter().map(|v| PossibleValue::new(v.clone())).collect())
    }
}

impl TypedValueParser for SuggestedValuesParser {
    type Value = String;

    fn parse_ref(
        &self,
        cmd: &Command,
        arg: Option<&Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<String, clap::Error> {
        StringValueParser::new().parse_ref(cmd, arg, value)
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(self.0.iter().cloned()))
    }
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// Every open-enum field in every JSON request body of `spec`.
fn open_enum_table(spec: &Value) -> Table {
    let mut table = Table::new();
    let Some(paths) = spec["paths"].as_object() else {
        return table;
    };
    for (path, ops) in paths {
        let Some(ops) = ops.as_object() else {
            continue;
        };
        for (http_method, op) in ops {
            let schema = &op["requestBody"]["content"]["application/json"]["schema"];
            if schema.is_null() {
                continue;
            }
            let mut fields = BTreeMap::new();
            walk(spec, schema, "", 0, &mut fields);
            if !fields.is_empty() {
                table.insert((http_method.to_ascii_lowercase(), path.clone()), fields);
            }
        }
    }
    table
}

fn walk(
    spec: &Value,
    schema: &Value,
    prefix: &str,
    depth: u8,
    out: &mut BTreeMap<String, Vec<String>>,
) {
    if depth >= MAX_DEPTH {
        return;
    }
    let schema = resolve(spec, schema);
    // `allOf` branches contribute properties at the same level, as the
    // runtime's own body flattening treats them.
    if let Some(branches) = schema["allOf"].as_array() {
        for branch in branches {
            walk(spec, branch, prefix, depth + 1, out);
        }
    }
    let Some(props) = schema["properties"].as_object() else {
        return;
    };
    for (name, prop) in props {
        let key = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}.{name}")
        };
        let prop = resolve(spec, prop);
        match open_enum_values(prop) {
            Some(values) => {
                out.insert(key, values);
            }
            None => walk(spec, prop, &key, depth + 1, out),
        }
    }
}

/// The listed values of an open enum, or `None` for any other schema.
///
/// Every non-null `anyOf` branch is an inline `type: string`, exactly one
/// carries a non-empty `enum`, at least one carries none, and at most one is
/// `{type: null}`. The composition may carry `type: string` itself (the typed
/// spelling) or no `type` (the raw one). A top-level `enum` or `const` makes
/// the schema closed; `oneOf`, `allOf`, `$ref` branches and non-string
/// branches are real unions (media refs, `anyOf [point, list]`) and never
/// match.
fn open_enum_values(schema: &Value) -> Option<Vec<String>> {
    let branches = schema["anyOf"].as_array()?;
    if schema.get("type").is_some_and(|t| t != "string")
        || ["oneOf", "allOf", "enum", "const"].iter().any(|k| schema.get(k).is_some())
    {
        return None;
    }

    let mut suggested: Option<Vec<String>> = None;
    let (mut plain, mut nulls) = (0usize, 0usize);
    for branch in branches {
        if branch["type"] == "null" {
            nulls += 1;
            continue;
        }
        let bare_string = branch["type"] == "string"
            && ["$ref", "nullable", "const", "anyOf", "oneOf", "allOf"]
                .iter()
                .all(|k| branch.get(k).is_none());
        if !bare_string {
            return None;
        }
        match branch.get("enum") {
            None => plain += 1,
            Some(values) => {
                let values: Vec<String> = values
                    .as_array()?
                    .iter()
                    .map(|v| v.as_str().map(str::to_string))
                    .collect::<Option<_>>()?;
                if values.is_empty() || suggested.is_some() {
                    return None;
                }
                suggested = Some(values);
            }
        }
    }
    if plain == 0 || nulls > 1 {
        return None;
    }
    suggested
}

/// Follow a local `$ref` chain to the schema it names.
fn resolve<'a>(spec: &'a Value, mut schema: &'a Value) -> &'a Value {
    for _ in 0..16 {
        let Some(target) = schema["$ref"].as_str() else {
            return schema;
        };
        let Some(pointer) = target.strip_prefix('#') else {
            return schema;
        };
        match spec.pointer(pointer) {
            Some(next) => schema = next,
            None => return schema,
        }
    }
    schema
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn raw() -> Value {
        json!({
            "anyOf": [{"type": "string", "enum": ["auto", "en", "ko"]}, {"type": "string"}],
            "description": "Language of the text."
        })
    }

    fn typed() -> Value {
        let mut v = raw();
        v["type"] = json!("string");
        v
    }

    fn codes() -> Vec<String> {
        vec!["auto".into(), "en".into(), "ko".into()]
    }

    #[test]
    fn recognizes_the_raw_and_typed_spellings() {
        assert_eq!(open_enum_values(&raw()), Some(codes()));
        assert_eq!(open_enum_values(&typed()), Some(codes()));
        let mut nullable = raw();
        nullable["anyOf"].as_array_mut().unwrap().push(json!({"type": "null"}));
        assert_eq!(open_enum_values(&nullable), Some(codes()));
    }

    #[test]
    fn rejects_every_other_shape() {
        let cases = [
            // Closed: the enum is on the property itself.
            json!({"type": "string", "enum": ["a"], "anyOf": [{"type": "string", "enum": ["a"]}, {"type": "string"}]}),
            // No plain-string branch.
            json!({"anyOf": [{"type": "string", "enum": ["a"]}, {"type": "null"}]}),
            // Two enum branches.
            json!({"anyOf": [{"type": "string", "enum": ["a"]}, {"type": "string", "enum": ["b"]}, {"type": "string"}]}),
            // No enum branch.
            json!({"anyOf": [{"type": "string"}, {"type": "string"}]}),
            // An empty enum.
            json!({"anyOf": [{"type": "string", "enum": []}, {"type": "string"}]}),
            // String or object: the media-ref family.
            json!({"anyOf": [{"type": "string", "enum": ["a"]}, {"type": "string"}, {"type": "object"}]}),
            // `anyOf [point, list]`.
            json!({"anyOf": [{"type": "array", "items": {"type": "number"}}, {"type": "array"}]}),
            // A `$ref` branch.
            json!({"anyOf": [{"$ref": "#/components/schemas/Lang"}, {"type": "string"}]}),
            // `oneOf` is not the published spelling.
            json!({"oneOf": [{"type": "string", "enum": ["a"]}, {"type": "string"}]}),
            // A non-string top-level type.
            json!({"type": "integer", "anyOf": [{"type": "string", "enum": ["a"]}, {"type": "string"}]}),
            // Two null branches.
            json!({"anyOf": [{"type": "string", "enum": ["a"]}, {"type": "string"}, {"type": "null"}, {"type": "null"}]}),
        ];
        for case in cases {
            assert_eq!(open_enum_values(&case), None, "must not match: {case}");
        }
    }

    /// Two submit operations, one per spelling, with the field one level down
    /// behind a `$ref` exactly as the v3 spec lays out `input`.
    fn spec() -> Value {
        let op = |input: &str| {
            json!({"post": {
                "operationId": format!("submit_{input}"),
                "tags": ["jobs"],
                "requestBody": {"required": true, "content": {"application/json": {"schema": {
                    "type": "object",
                    "required": ["input"],
                    "properties": {"input": {"$ref": format!("#/components/schemas/{input}")}}
                }}}},
                "responses": {"202": {"description": "ok"}}
            }})
        };
        let input = |language: Value| {
            json!({"type": "object", "required": ["text"], "properties": {
                "text": {"type": "string"},
                "language": language
            }})
        };
        json!({
            "openapi": "3.1.0",
            "info": {"title": "t", "version": "1"},
            "paths": {"/models/raw": op("InputRaw"), "/models/typed": op("InputTyped")},
            "components": {"schemas": {"InputRaw": input(raw()), "InputTyped": input(typed())}}
        })
    }

    #[test]
    fn finds_fields_behind_refs_keyed_by_wire_name() {
        let table = open_enum_table(&spec());
        for path in ["/models/raw", "/models/typed"] {
            let fields = &table[&("post".to_string(), path.to_string())];
            assert_eq!(fields.keys().collect::<Vec<_>>(), ["input.language"], "{path}");
            assert_eq!(fields["input.language"], codes(), "{path}");
        }
    }

    #[test]
    fn the_embedded_spec_loads() {
        // No open enum ships at spec 3.20.0; this guards the walk against a
        // spec it cannot read, and every hit must sit under `input`.
        let spec: Value = serde_json::from_str(include_str!("openapi0.json")).unwrap();
        let table = open_enum_table(&spec);
        assert!(
            table.values().flat_map(|f| f.keys()).all(|k| k.starts_with("input.")),
            "{table:?}"
        );
    }

    /// The real command tree, built by the runtime from `spec()` with the
    /// transform installed through the Replay-patched seam: one submit
    /// command per spelling. This is the only test in the binary that
    /// installs the transform (the seam is first-wins).
    fn submit_commands() -> Vec<Command> {
        install_for(spec);
        let doc = fern_cli_sdk::openapi::load_openapi_spec(&spec().to_string(), "t").unwrap();
        let root = fern_cli_sdk::openapi::commands::build_cli(&doc);
        let jobs = root.find_subcommand("jobs").expect("jobs group");
        jobs.get_subcommands().cloned().collect()
    }

    #[test]
    fn the_seam_rewrites_both_spellings_end_to_end() {
        let commands = submit_commands();
        assert_eq!(commands.len(), 2, "one command per spelling");

        for mut cmd in commands {
            let name = cmd.get_name().to_string();

            for input in ["ko", "Korean", "KO", "pt-BR", "123"] {
                let m = cmd
                    .clone()
                    .try_get_matches_from(["x", "--input.text", "hi", "--input.language", input])
                    .unwrap_or_else(|e| panic!("{name}: `{input}` must parse: {e}"));
                assert_eq!(
                    m.get_one::<String>("input.language").map(String::as_str),
                    Some(input),
                    "{name}"
                );
            }

            let arg = cmd
                .get_arguments()
                .find(|a| a.get_id() == "input.language")
                .expect("flag");
            let values: Vec<String> = arg
                .get_possible_values()
                .iter()
                .map(|v| v.get_name().to_string())
                .collect();
            assert_eq!(values, codes(), "{name}");

            let long = cmd.render_long_help().to_string();
            let short = cmd.render_help().to_string();
            assert!(
                long.contains("[suggested values: auto, en, ko; any other string is also accepted]"),
                "{name} --help:\n{long}"
            );
            assert!(!short.contains("suggested values"), "{name} -h:\n{short}");
            for help in [&long, &short] {
                assert!(!help.contains("possible values"), "{name}:\n{help}");
                assert!(help.contains("--input.language <STRING>"), "{name}:\n{help}");
            }

            let mut script = Vec::new();
            clap_complete::generate(clap_complete::Shell::Zsh, &mut cmd, "x", &mut script);
            let script = String::from_utf8(script).unwrap();
            let line = script
                .lines()
                .find(|l| l.contains("--input.language"))
                .unwrap_or_else(|| panic!("{name}: no completion line:\n{script}"));
            assert!(line.contains(":STRING:(auto en ko)"), "{name}: {line}");
        }
    }
}
