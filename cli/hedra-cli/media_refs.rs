//! Bare-value shorthand for media-reference flags (ENG-11602).
//!
//! A media-reference field is a body property whose schema is the `MediaRef`
//! discriminated union — `oneOf [UrlRef, AssetRef]` on `source`, with branches
//! `{source: "url", url}` and `{source: "asset", asset_id}`. The runtime only
//! sees such a field as `type: object`, so the CLI demanded the whole JSON
//! object and rejected a bare id. This expands the two bare forms instead:
//!
//! | typed                           | sent                                     |
//! | ------------------------------- | ---------------------------------------- |
//! | `asset_341d…`                   | `{"source": "asset", "asset_id": "…"}`   |
//! | `https://…` / `http://…`        | `{"source": "url", "url": "…"}`          |
//! | `{"source": "asset", …}` (JSON) | unchanged                                |
//!
//! for single fields and for each element of an array of them
//! (`--input.images a --input.images b`). Anything else is rejected before the
//! request is built, with an error naming the three accepted forms.
//!
//! Fields are found from the union's *shape* in the embedded spec, never from
//! a field-name list, so a model added to the catalog later is covered with no
//! edit here. A plain `type: object` field is not a `MediaRef` and is left to
//! the runtime exactly as before. The typeless `anyOf [MediaRef,
//! array<MediaRef>]` fields (`hedra-avatar` / `hedra-character-3` `audio`)
//! are not `MediaRef` either and are deliberately not matched.
//!
//! The runtime has no request-side hook of its own, so this rides on the one
//! generic seam carried as a Fern Replay patch on `src/openapi/executor.rs`:
//! [`fern_cli_sdk::openapi::executor::set_param_transform`], which hands the
//! collected parameters to a transform before any coercion. Everything
//! Hedra-specific lives here, in a `.fernignore`d file.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use fern_cli_sdk::error::CliError;
use fern_cli_sdk::openapi::discovery::RestMethod;
use serde_json::{json, Map, Value};

/// How deep to follow nested body objects looking for media fields. The
/// submit bodies put them one level down (`input.*`); this leaves room.
const MAX_DEPTH: u8 = 8;

/// Where the media reference sits in a parameter's value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Slot {
    /// The value is one `MediaRef`.
    Single,
    /// The value is an array whose elements are each a `MediaRef`.
    Items,
}

/// Install the shorthand. Called once from `custom::register`.
pub fn install() {
    fern_cli_sdk::openapi::executor::set_param_transform(expand);
}

fn embedded_spec() -> &'static Value {
    static SPEC: OnceLock<Value> = OnceLock::new();
    SPEC.get_or_init(|| serde_json::from_str(include_str!("openapi0.json")).unwrap_or(Value::Null))
}

fn expand(method: &RestMethod, params: &mut Map<String, Value>) -> Result<(), CliError> {
    // Parsed lazily, once, and only for an operation that was handed at
    // least one parameter.
    if params.is_empty() {
        return Ok(());
    }
    let slots = media_slots(embedded_spec(), &method.http_method, &method.path);
    for (key, slot) in slots {
        let Some(value) = params.get_mut(&key) else {
            continue;
        };
        let flag = flag_name(method, &key);
        match slot {
            Slot::Single => *value = expand_one(value, &flag)?,
            Slot::Items => {
                if let Value::Array(items) = value {
                    for (i, item) in items.iter_mut().enumerate() {
                        *item = expand_one(item, &format!("{flag} (item {})", i + 1))?;
                    }
                }
            }
        }
    }
    Ok(())
}

/// Rewrite one flag value supplied for a `MediaRef`.
fn expand_one(value: &Value, flag: &str) -> Result<Value, CliError> {
    match value {
        // The JSON-object form, already decoded by the flag collector.
        Value::Object(_) => Ok(value.clone()),
        // A nullable field's `null` sentinel belongs to the runtime.
        Value::Null => Ok(Value::Null),
        Value::String(raw) => {
            let s = raw.trim();
            if s.starts_with('{') {
                // Reaches here only when the collector could not decode it
                // (or via `--params` as a string); say why, not just "string".
                return match serde_json::from_str::<Value>(s) {
                    Ok(obj @ Value::Object(_)) => Ok(obj),
                    Ok(_) => Err(rejected(flag, raw, None)),
                    Err(e) => Err(rejected(flag, raw, Some(&e.to_string()))),
                };
            }
            if let Some(id) = asset_id(s) {
                return Ok(json!({ "source": "asset", "asset_id": id }));
            }
            if is_http_url(s) {
                return Ok(json!({ "source": "url", "url": s }));
            }
            Err(rejected(flag, raw, None))
        }
        other => Err(rejected(flag, &other.to_string(), None)),
    }
}

/// `asset_<id>`: the prefix plus a non-empty, whitespace-free remainder. The
/// id itself is opaque — the server issues it and validates it.
fn asset_id(s: &str) -> Option<&str> {
    let rest = s.strip_prefix("asset_")?;
    (!rest.is_empty() && !s.contains(char::is_whitespace)).then_some(s)
}

fn is_http_url(s: &str) -> bool {
    let lower = s.get(..8).unwrap_or(s).to_ascii_lowercase();
    let rest = if lower.starts_with("https://") {
        &s[8..]
    } else if lower.starts_with("http://") {
        &s[7..]
    } else {
        return false;
    };
    !rest.is_empty() && !s.contains(char::is_whitespace)
}

fn rejected(flag: &str, got: &str, json_error: Option<&str>) -> CliError {
    let why = json_error
        .map(|e| format!(" (not valid JSON: {e})"))
        .unwrap_or_default();
    CliError::Validation(format!(
        "{flag} expects an asset id (asset_…), an http(s):// URL, or a JSON object \
         such as '{{\"source\":\"asset\",\"asset_id\":\"asset_…\"}}'; got '{got}'{why}"
    ))
}

/// The flag a wire key is registered under, for error messages. Mirrors the
/// runtime's `resolve_param_flag_name` for body params, which is not public:
/// an explicit override, else the display name, else the wire name, kebabed.
fn flag_name(method: &RestMethod, key: &str) -> String {
    let param = method.parameters.get(key);
    if let Some(flag) = param.and_then(|p| p.flag_name_override.as_deref()) {
        return format!("--{flag}");
    }
    let source = param.and_then(|p| p.display_name.as_deref()).unwrap_or(key);
    format!("--{}", source.replace('_', "-"))
}

// ---------------------------------------------------------------------------
// Detection
// ---------------------------------------------------------------------------

/// Every media field in the JSON request body of `http_method path`, keyed by
/// the dotted wire name the runtime collects it under (`input.source_image`).
fn media_slots(spec: &Value, http_method: &str, path: &str) -> BTreeMap<String, Slot> {
    let mut out = BTreeMap::new();
    let schema = &spec["paths"][path][http_method.to_ascii_lowercase()]["requestBody"]["content"]
        ["application/json"]["schema"];
    if !schema.is_null() {
        walk(spec, schema, "", 0, &mut out);
    }
    out
}

fn walk(spec: &Value, schema: &Value, prefix: &str, depth: u8, out: &mut BTreeMap<String, Slot>) {
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
        if is_media_ref(spec, prop) {
            out.insert(key, Slot::Single);
        } else if prop["type"] == "array" && is_media_ref(spec, resolve(spec, &prop["items"])) {
            out.insert(key, Slot::Items);
        } else {
            walk(spec, prop, &key, depth + 1, out);
        }
    }
}

/// The `MediaRef` shape: a `oneOf` discriminated on `source` with exactly two
/// branches, `{source: "url", url}` and `{source: "asset", asset_id}`.
fn is_media_ref(spec: &Value, schema: &Value) -> bool {
    if schema["discriminator"]["propertyName"] != "source" {
        return false;
    }
    let Some(branches) = schema["oneOf"].as_array() else {
        return false;
    };
    let mut tags: Vec<&str> = branches
        .iter()
        .filter_map(|b| branch_tag(resolve(spec, b)))
        .collect();
    tags.sort_unstable();
    branches.len() == 2 && tags == ["asset", "url"]
}

/// `"url"` / `"asset"` for a branch that pins `source` to that one value and
/// carries the payload field that goes with it.
fn branch_tag(branch: &Value) -> Option<&'static str> {
    let props = branch["properties"].as_object()?;
    let source = props.get("source")?;
    let pinned = match source["enum"].as_array() {
        Some(values) if values.len() == 1 => values[0].as_str()?,
        _ => source["const"].as_str()?,
    };
    match pinned {
        "url" if props.contains_key("url") => Some("url"),
        "asset" if props.contains_key("asset_id") => Some("asset"),
        _ => None,
    }
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

    /// Every `(method, path, key, slot)` media field in the embedded spec.
    fn all_slots() -> Vec<(String, String, String, Slot)> {
        let spec = embedded_spec();
        let mut found = Vec::new();
        for (path, ops) in spec["paths"].as_object().expect("paths") {
            for method in ops.as_object().expect("operations").keys() {
                for (key, slot) in media_slots(spec, method, path) {
                    found.push((method.clone(), path.clone(), key, slot));
                }
            }
        }
        found
    }

    #[test]
    fn finds_every_media_field_in_the_embedded_spec() {
        let found = all_slots();
        let ops: std::collections::BTreeSet<_> = found.iter().map(|f| &f.1).collect();
        // Not a pinned count: the catalog grows. The floor is spec 3.17.12's
        // 151 fields across 89 submit operations (ENG-11602).
        assert!(found.len() >= 151, "found {} media fields", found.len());
        assert!(ops.len() >= 89, "found {} operations", ops.len());
        // Every one sits under the model-specific `input` object.
        assert!(found.iter().all(|f| f.2.starts_with("input.")), "{found:?}");
    }

    #[test]
    fn detects_single_fields_and_array_items() {
        let spec = embedded_spec();
        let slots = media_slots(spec, "POST", "/models/topaz-image-upscaler");
        assert_eq!(
            slots.into_iter().collect::<Vec<_>>(),
            vec![("input.source_image".to_string(), Slot::Single)]
        );
        let slots = media_slots(spec, "post", "/models/eyeline-id-relight");
        assert_eq!(slots.get("input.images"), Some(&Slot::Items));
        assert_eq!(slots.get("input.source_video"), Some(&Slot::Single));
    }

    #[test]
    fn leaves_the_typeless_audio_union_alone() {
        // `anyOf [MediaRef, array<MediaRef>]` is out of scope (ENG-11602).
        let spec = embedded_spec();
        for path in ["/models/hedra-avatar", "/models/hedra-character-3"] {
            let slots = media_slots(spec, "POST", path);
            assert!(!slots.contains_key("input.audio"), "{path}: {slots:?}");
            assert!(
                !slots.contains_key("input.bounding_box_target"),
                "{path}: {slots:?}"
            );
        }
    }

    #[test]
    fn ignores_objects_that_are_not_media_refs() {
        let spec = json!({
            "paths": {"/x": {"post": {"requestBody": {"content": {"application/json": {
                "schema": {"type": "object", "properties": {
                    "plain": {"type": "object", "properties": {"a": {"type": "string"}}},
                    "other_union": {
                        "type": "object",
                        "discriminator": {"propertyName": "source"},
                        "oneOf": [
                            {"properties": {"source": {"enum": ["url"]}, "url": {}}},
                            {"properties": {"source": {"enum": ["file"]}, "path": {}}}
                        ]
                    },
                    "wrong_discriminator": {
                        "type": "object",
                        "discriminator": {"propertyName": "kind"},
                        "oneOf": [
                            {"properties": {"source": {"enum": ["url"]}, "url": {}}},
                            {"properties": {"source": {"enum": ["asset"]}, "asset_id": {}}}
                        ]
                    },
                    "by_ref": {"$ref": "#/components/schemas/MediaRef"}
                }}
            }}}}}},
            "components": {"schemas": {
                "MediaRef": {
                    "discriminator": {"propertyName": "source"},
                    "oneOf": [
                        {"$ref": "#/components/schemas/UrlRef"},
                        {"$ref": "#/components/schemas/AssetRef"}
                    ]
                },
                "UrlRef": {"properties": {"source": {"const": "url"}, "url": {}}},
                "AssetRef": {"properties": {"source": {"const": "asset"}, "asset_id": {}}}
            }}
        });
        let slots = media_slots(&spec, "POST", "/x");
        assert_eq!(
            slots.into_iter().collect::<Vec<_>>(),
            vec![("by_ref".to_string(), Slot::Single)]
        );
    }

    #[test]
    fn expands_the_bare_forms() {
        assert_eq!(
            expand_one(&json!("asset_341d2ba0-c784"), "--f").unwrap(),
            json!({"source": "asset", "asset_id": "asset_341d2ba0-c784"})
        );
        assert_eq!(
            expand_one(&json!("https://cdn.example/a.png"), "--f").unwrap(),
            json!({"source": "url", "url": "https://cdn.example/a.png"})
        );
        assert_eq!(
            expand_one(&json!("HTTP://cdn.example/a.png"), "--f").unwrap(),
            json!({"source": "url", "url": "HTTP://cdn.example/a.png"})
        );
        let obj = json!({"source": "url", "url": "https://x"});
        assert_eq!(expand_one(&obj, "--f").unwrap(), obj);
        assert_eq!(
            expand_one(&json!(r#"{"source":"asset","asset_id":"asset_1"}"#), "--f").unwrap(),
            json!({"source": "asset", "asset_id": "asset_1"})
        );
    }

    #[test]
    fn rejects_everything_else_naming_the_accepted_forms() {
        for bad in [
            json!("foo"),
            json!("asset_"),
            json!("https://"),
            json!("ftp://x"),
            json!("asset_a b"),
            json!("{not json"),
            json!(42),
            json!(["asset_a"]),
        ] {
            let err = expand_one(&bad, "--input.source-image")
                .unwrap_err()
                .to_string();
            assert!(
                err.starts_with("--input.source-image expects"),
                "{bad}: {err}"
            );
            for form in ["asset id (asset_…)", "http(s):// URL", "JSON object"] {
                assert!(err.contains(form), "{bad}: {err}");
            }
        }
    }
}
