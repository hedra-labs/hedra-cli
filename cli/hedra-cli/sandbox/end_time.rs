//! The duration and `none` forms of `sandbox update --ttl-ends-at`. The
//! generated flag sends any value other than `null` as a string, so this
//! module turns a duration into the time that long from now, and `none` into
//! `null`, before the runtime builds the request.

use fern_cli_sdk::error::CliError;
use fern_cli_sdk::openapi::discovery::RestMethod;
use hedra_cli_sdk::api::prelude::{DateTime, Utc};
use serde_json::{Map, Value};

use super::render;

const UPDATE_OPERATION: &str = "sandboxes_update";

/// Rewrites `ttl_ends_at` in `sandbox update`'s parameters: leaves an RFC
/// 3339 time as given, replaces a duration such as `10h` with the time that
/// long from now and `none` with `null`, and returns `CliError::Validation`
/// for any other string.
pub(crate) fn rewrite_ttl_ends_at(
    method: &RestMethod,
    params: &mut Map<String, Value>,
) -> Result<(), CliError> {
    if method.id.as_deref() != Some(UPDATE_OPERATION) {
        return Ok(());
    }
    let Some(Value::String(raw)) = params.get("ttl_ends_at") else {
        return Ok(());
    };
    let value = end_time(raw, Utc::now()).ok_or_else(|| {
        CliError::Validation(format!(
            "--ttl-ends-at takes an RFC 3339 time with an offset such as \
             2026-11-20T18:00:00Z, a duration from now such as 10h, 90m or 2d, \
             or none; got `{raw}`."
        ))
    })?;
    params.insert("ttl_ends_at".to_string(), value);
    Ok(())
}

fn end_time(raw: &str, now: DateTime<Utc>) -> Option<Value> {
    if raw == "none" {
        return Some(Value::Null);
    }
    if DateTime::parse_from_rfc3339(raw).is_ok() {
        return Some(Value::String(raw.to_string()));
    }
    let seconds = render::parse_duration(raw).ok()?;
    let at = DateTime::<Utc>::from_timestamp(now.timestamp().checked_add(seconds)?, 0)?;
    Some(Value::String(at.format("%Y-%m-%dT%H:%M:%SZ").to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-11-14T08:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn a_duration_is_counted_from_now() {
        assert_eq!(end_time("10h", now()), Some(json!("2026-11-14T18:00:00Z")));
        assert_eq!(end_time("90m", now()), Some(json!("2026-11-14T09:30:00Z")));
        assert_eq!(end_time("2d", now()), Some(json!("2026-11-16T08:00:00Z")));
    }

    #[test]
    fn a_time_needs_an_offset_and_other_values_are_refused() {
        for raw in [
            "",
            "tomorrow",
            "None",
            "2026-11-20",
            "2026-11-20T18:00:00",
            "1.5h",
            "-1h",
            "99999999999999d",
        ] {
            assert_eq!(end_time(raw, now()), None, "{raw}");
        }
        let offset = "2026-11-20T10:00:00.5-08:00";
        assert_eq!(end_time(offset, now()), Some(json!(offset)));
    }
}
