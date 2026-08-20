// SPDX-License-Identifier: Apache-2.0

//! Hand-owned `--format table` views for responses the generic renderer
//! mangles.
//!
//! This file is yours to edit — it is listed in `.fernignore`, which is
//! load-bearing rather than decorative: the generator never emits this
//! path, so the entry is the only thing stopping regeneration from
//! *deleting* the file.
//!
//! ## The problem
//!
//! `formatter::format_table_page` hands every object to `extract_items`, a
//! heuristic written for list envelopes (`{"data": [...], "next_cursor":
//! ...}`). It returns the first non-empty array it finds and the renderer
//! then prints *only* that array, discarding every sibling field. Because
//! `serde_json` is compiled without `preserve_order`, its `Map` is a
//! `BTreeMap`, so "first" means "alphabetically first".
//!
//! Seven response schemas are single resources that happen to carry an
//! array, and every one of them loses data:
//!
//! | operation                     | renders only   | drops                |
//! |-------------------------------|----------------|----------------------|
//! | `jobs get`                    | `logs`         | 10 fields            |
//! | `jobs get-status`             | `logs`         | 5 fields             |
//! | `keys create` / `keys rotate` | `scopes`       | 7 fields, incl. the  |
//! |                               |                | `credential` secret  |
//! | `billing get-usage`           | `data`         | 6 fields             |
//! | `webhooks redeliver`          | `redeliveries` | 13 fields            |
//! | `log-drains create-log-drain` | `header_names` | 15 fields            |
//!
//! ## The fix
//!
//! Two layers, both applied as `transform_response` hooks registered from
//! `custom.rs` and — like `reshape_model_list_table` — only when the
//! resolved output format is `Table`. JSON/YAML/CSV/JSONL output is
//! therefore byte-identical to before, so scripts are unaffected.
//!
//! 1. [`dehijack`] rewrites arrays so `extract_items` cannot grab them,
//!    which restores every dropped field. It deliberately leaves genuine
//!    list envelopes alone so `keys list` and friends keep their table.
//! 2. [`jobs`] additionally reshapes the job envelopes for human reading:
//!    humanised cost and duration, the error envelope surfaced, and
//!    follow-up commands after a submit.
//!
//! ## Known limitation
//!
//! Field *order* is alphabetical and cannot be changed from here — the
//! renderer walks a `BTreeMap`. Semantic ordering would require
//! hand-owning `src/formatter.rs`, which stays generator-owned by choice.

use fern_cli_sdk::error::CliError;
use serde_json::{Map, Value};

// ── Hook entry points ───────────────────────────────────────────────

/// `transform_response` hook for `["jobs", "*"]`.
///
/// One registration covers `get`, `get-status`, `submit` and all ~80
/// generated `submit-<model>` leaves; the operation is selected from the
/// path rather than by registering a pattern per command. Operations the
/// generic renderer already handles correctly (`list`, `list-job-logs`)
/// pass through untouched.
pub(crate) async fn jobs(value: Value, path: Vec<String>) -> Result<Value, CliError> {
    let op = path.get(1).map(String::as_str).unwrap_or_default();
    Ok(match op {
        "get" => job_result(&value),
        "get-status" => job_status(&value),
        // `submit` plus every `submit-<model>` variant.
        _ if op.starts_with("submit") => job_submit(&value),
        _ => value,
    })
}

/// `transform_response` hook for the non-job resources whose single-object
/// responses are hijacked by an array field.
pub(crate) async fn defuse_arrays(value: Value, _: Vec<String>) -> Result<Value, CliError> {
    Ok(dehijack(value))
}

// ── Layer 1: generic de-hijacking ───────────────────────────────────

/// Rewrite `value` so `extract_items` cannot mistake it for a list.
///
/// Genuine list envelopes are returned unchanged — their array table is
/// the correct rendering and the whole point of the heuristic.
pub(crate) fn dehijack(value: Value) -> Value {
    match value {
        Value::Object(ref obj) if is_list_envelope(obj) => value,
        other => defuse(other),
    }
}

/// True when `obj` looks like a paginated list envelope: exactly one array
/// field, and every other field is pagination metadata.
///
/// Deliberately strict. Misjudging a list as a single resource only costs
/// prettiness (rows render as indexed keys); misjudging a single resource
/// as a list costs data, which is the bug being fixed.
fn is_list_envelope(obj: &Map<String, Value>) -> bool {
    let mut arrays = obj.iter().filter(|(_, v)| v.is_array());
    let Some((array_key, _)) = arrays.next() else {
        return false;
    };
    if arrays.next().is_some() {
        return false;
    }
    obj.keys().all(|key| {
        key == array_key
            || matches!(
                key.as_str(),
                "next_cursor" | "nextPageToken" | "cursor" | "has_more" | "kind"
            )
    })
}

/// Recursively replace arrays with values `extract_items` ignores.
///
/// Arrays of scalars collapse to a joined string, which is what
/// `value_to_cell` would have rendered anyway. Arrays of objects become
/// objects keyed by a 1-based index, so the renderer's own object
/// flattening turns them into `field.1.subfield` rows instead of a single
/// unreadable JSON blob.
fn defuse(value: Value) -> Value {
    match value {
        Value::Array(items) => {
            if items
                .iter()
                .all(|item| !item.is_object() && !item.is_array())
            {
                Value::String(items.iter().map(scalar_cell).collect::<Vec<_>>().join(", "))
            } else {
                let width = index_width(items.len());
                Value::Object(
                    items
                        .into_iter()
                        .enumerate()
                        .map(|(i, item)| (format!("{:0width$}", i + 1), defuse(item)))
                        .collect(),
                )
            }
        }
        Value::Object(obj) => Value::Object(
            obj.into_iter()
                .map(|(key, val)| (key, defuse(val)))
                .collect(),
        ),
        other => other,
    }
}

// ── Layer 2: job views ──────────────────────────────────────────────

/// `jobs get` — the result envelope.
fn job_result(value: &Value) -> Value {
    let Some(job) = value.as_object() else {
        return value.clone();
    };
    let mut out = Map::new();
    put(&mut out, "job", job.get("job_id"));
    put(&mut out, "status", job.get("status"));
    out.insert("model".into(), Value::String(model_label(job)));

    if let Some(cost) = job.get("cost").and_then(Value::as_f64) {
        let currency = job.get("currency").and_then(Value::as_str).unwrap_or("");
        out.insert(
            "cost".into(),
            Value::String(format!("{cost:.2} {currency}").trim_end().to_string()),
        );
    }
    if let Some(ms) = job
        .get("metrics")
        .and_then(Value::as_object)
        .and_then(|m| m.get("processing_time_ms"))
        .and_then(Value::as_i64)
    {
        out.insert("elapsed".into(), Value::String(duration(ms)));
    }
    if let Some(prompt) = job.get("prompt").and_then(Value::as_str) {
        out.insert("prompt".into(), Value::String(one_line(prompt, 96)));
    }

    if let Some(err) = job.get("error").and_then(Value::as_object) {
        let code = err.get("code").and_then(Value::as_str).unwrap_or("ERROR");
        let message = err.get("message").and_then(Value::as_str).unwrap_or("");
        out.insert("error".into(), Value::String(format!("{code}: {message}")));
        if err.get("retryable").and_then(Value::as_bool) == Some(true) {
            let after = err
                .get("retry_after")
                .and_then(Value::as_i64)
                .map(|s| format!(" (after {s}s)"))
                .unwrap_or_default();
            out.insert("error.retry".into(), Value::String(format!("yes{after}")));
        }
        if let Some(param) = err.get("param").and_then(Value::as_str) {
            out.insert("error.param".into(), Value::String(param.to_string()));
        }
        let details = err
            .get("details")
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let width = index_width(details.len());
        for (i, detail) in details.iter().enumerate() {
            out.insert(
                format!("error.detail.{:0width$}", i + 1),
                Value::String(field_error(detail)),
            );
        }
    }

    insert_outputs(&mut out, job.get("outputs").and_then(Value::as_array));
    insert_logs(&mut out, job.get("logs").and_then(Value::as_array));
    Value::Object(out)
}

/// `jobs get-status` — the lightweight poll envelope.
fn job_status(value: &Value) -> Value {
    let Some(job) = value.as_object() else {
        return value.clone();
    };
    let mut out = Map::new();
    put(&mut out, "job", job.get("job_id"));
    put(&mut out, "status", job.get("status"));
    if let Some(fraction) = job.get("progress").and_then(Value::as_f64) {
        out.insert(
            "progress".into(),
            Value::String(format!("{:.0}%", fraction * 100.0)),
        );
    }
    if let Some(eta) = job.get("estimated_completion_at").and_then(Value::as_str) {
        out.insert("eta".into(), Value::String(instant(eta)));
    }
    insert_logs(&mut out, job.get("logs").and_then(Value::as_array));
    Value::Object(out)
}

/// `jobs submit` and every `jobs submit-<model>` — the 202 acknowledgement.
///
/// The two URL fields are server paths (`/v3/jobs/{id}/status`), which are
/// not actionable from a shell. They are replaced by the CLI commands that
/// do the same thing.
fn job_submit(value: &Value) -> Value {
    let Some(job) = value.as_object() else {
        return value.clone();
    };
    let mut out = Map::new();
    put(&mut out, "job", job.get("job_id"));
    put(&mut out, "status", job.get("status"));
    put(&mut out, "model", job.get("model"));
    if let Some(eta) = job.get("estimated_completion_at").and_then(Value::as_str) {
        out.insert("eta".into(), Value::String(instant(eta)));
    }
    if let Some(id) = job.get("job_id").and_then(Value::as_str) {
        out.insert(
            "next".into(),
            Value::String(format!("hedra-cli jobs get {id}")),
        );
        out.insert(
            "watch".into(),
            Value::String(format!("hedra-cli jobs stream {id}")),
        );
    }
    Value::Object(out)
}

// ── Job field helpers ───────────────────────────────────────────────

/// `model` plus the quality level, when the model offers one.
fn model_label(job: &Map<String, Value>) -> String {
    let model = job.get("model").and_then(Value::as_str).unwrap_or_default();
    match job.get("quality").and_then(Value::as_str) {
        Some(quality) => format!("{model}  ({quality})"),
        None => model.to_string(),
    }
}

/// Emit one `outputs.N` row per output, with the asset id and the full
/// download URL on their own rows.
///
/// The URL is deliberately **not** truncated: a presigned URL that cannot
/// be copied is worse than one that wraps.
fn insert_outputs(out: &mut Map<String, Value>, outputs: Option<&Vec<Value>>) {
    let Some(outputs) = outputs.filter(|o| !o.is_empty()) else {
        return;
    };
    let width = index_width(outputs.len());
    let rows: Vec<[String; 3]> = outputs
        .iter()
        .map(|item| {
            let o = item.as_object();
            let get = |k: &str| o.and_then(|o| o.get(k));
            let dimensions = match (
                get("width").and_then(Value::as_i64),
                get("height").and_then(Value::as_i64),
            ) {
                (Some(w), Some(h)) => format!("{w}x{h}"),
                _ => String::new(),
            };
            let length = get("duration_ms")
                .and_then(Value::as_i64)
                .map(duration)
                .unwrap_or_default();
            [
                get("content_type")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                dimensions,
                length,
            ]
        })
        .collect();
    let widths = column_widths(&rows);

    for (i, item) in outputs.iter().enumerate() {
        let key = format!("outputs.{:0width$}", i + 1);
        // Nothing follows this row, so the padding is trimmed away.
        out.insert(
            key.clone(),
            Value::String(pad(&rows[i], &widths).trim_end().to_string()),
        );
        let o = item.as_object();
        if let Some(asset) = o.and_then(|o| o.get("asset_id")).and_then(Value::as_str) {
            out.insert(format!("{key}.asset"), Value::String(asset.to_string()));
        }
        let url = o
            .and_then(|o| o.get("url"))
            .and_then(Value::as_str)
            .unwrap_or("(expired)");
        out.insert(format!("{key}.url"), Value::String(url.to_string()));
    }
}

/// Emit one `logs.N` row per lifecycle event, pre-aligned into columns.
fn insert_logs(out: &mut Map<String, Value>, logs: Option<&Vec<Value>>) {
    let Some(logs) = logs.filter(|l| !l.is_empty()) else {
        return;
    };
    let width = index_width(logs.len());
    let rows: Vec<[String; 3]> = logs
        .iter()
        .map(|item| {
            let o = item.as_object();
            let get = |k: &str| o.and_then(|o| o.get(k)).and_then(Value::as_str);
            [
                get("timestamp").map(clock).unwrap_or_default(),
                get("level").unwrap_or_default().to_string(),
                get("event").unwrap_or_default().to_string(),
            ]
        })
        .collect();
    let widths = column_widths(&rows);

    for (i, item) in logs.iter().enumerate() {
        let message = item
            .as_object()
            .and_then(|o| o.get("message"))
            .and_then(Value::as_str)
            .unwrap_or_default();
        out.insert(
            format!("logs.{:0width$}", i + 1),
            Value::String(format!(
                "{}  {}",
                pad(&rows[i], &widths),
                one_line(message, 72)
            )),
        );
    }
}

// ── Formatting primitives ───────────────────────────────────────────

/// Copy `source` into `out` under `key` when present.
fn put(out: &mut Map<String, Value>, key: &str, source: Option<&Value>) {
    if let Some(value) = source.filter(|v| !v.is_null()) {
        out.insert(key.to_string(), value.clone());
    }
}

/// Digits needed to index `len` items, so `logs.02` sorts before `logs.10`.
///
/// The renderer sorts keys as strings, so indices must be zero-padded to a
/// fixed width or the tenth row lands between the first and the second.
fn index_width(len: usize) -> usize {
    len.to_string().len()
}

/// Widest value in each column.
fn column_widths<const N: usize>(rows: &[[String; N]]) -> [usize; N] {
    let mut widths = [0usize; N];
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            widths[i] = widths[i].max(cell.chars().count());
        }
    }
    widths
}

/// Pad every cell to its column width and join with two spaces.
///
/// The last column is padded too, because callers append a further column
/// to the result (a log's message). Callers for whom the padded row *is*
/// the final value trim it themselves — trimming here would silently
/// unalign whatever gets appended.
fn pad<const N: usize>(row: &[String; N], widths: &[usize; N]) -> String {
    row.iter()
        .enumerate()
        .map(|(i, cell)| {
            let fill = widths[i].saturating_sub(cell.chars().count());
            format!("{cell}{}", " ".repeat(fill))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

/// Humanise a millisecond duration.
fn duration(ms: i64) -> String {
    if ms < 0 {
        return String::new();
    }
    let seconds = ms as f64 / 1000.0;
    if seconds < 60.0 {
        return format!("{seconds:.1}s");
    }
    let total = seconds.round() as i64;
    let (minutes, seconds) = (total / 60, total % 60);
    if minutes < 60 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{}h {}m", minutes / 60, minutes % 60)
    }
}

/// `2026-08-20T18:42:11.482913Z` → `2026-08-20 18:42:11 UTC`.
///
/// The API serves UTC instants. There is no timezone library in the build
/// graph (`chrono` is a lockfile-only dependency), so the instant is shown
/// as served and explicitly labelled rather than left silently ambiguous.
fn instant(timestamp: &str) -> String {
    let trimmed: String = timestamp.chars().take(19).collect();
    if trimmed.chars().count() < 19 {
        return timestamp.to_string();
    }
    let zulu = timestamp.ends_with('Z') || timestamp.contains("+00:00");
    format!(
        "{}{}",
        trimmed.replace('T', " "),
        if zulu { " UTC" } else { "" }
    )
}

/// `2026-08-20T18:40:02.113Z` → `18:40:02Z`, the time of day alone.
fn clock(timestamp: &str) -> String {
    let time: String = timestamp.chars().skip(11).take(8).collect();
    if time.chars().count() < 8 {
        return timestamp.to_string();
    }
    let suffix = if timestamp.ends_with('Z') || timestamp.contains("+00:00") {
        "Z"
    } else {
        ""
    };
    format!("{time}{suffix}")
}

/// Collapse whitespace onto one line and clip to `limit` characters.
///
/// The key/value renderer writes one line per field, so an embedded
/// newline would break the column alignment of every following row.
fn one_line(text: &str, limit: usize) -> String {
    let squeezed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if squeezed.chars().count() <= limit {
        return squeezed;
    }
    let clipped: String = squeezed.chars().take(limit.saturating_sub(1)).collect();
    format!("{clipped}…")
}

/// Render one `FieldError` as `param: message`.
fn field_error(detail: &Value) -> String {
    let o = detail.as_object();
    let get = |k: &str| o.and_then(|o| o.get(k)).and_then(Value::as_str);
    match (get("param"), get("message")) {
        (Some(param), Some(message)) => format!("{param}: {message}"),
        (None, Some(message)) => message.to_string(),
        _ => scalar_cell(detail),
    }
}

/// Render a scalar the way the generic renderer's `value_to_cell` would.
fn scalar_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fern_cli_sdk::formatter::{format_value, OutputFormat};
    use serde_json::json;

    /// Render through the real table formatter, which is what the user
    /// actually sees. Asserting on the reshaped `Value` alone would not
    /// catch the `BTreeMap` key ordering that motivated the zero-padded
    /// indices.
    fn render(value: &Value) -> String {
        format_value(value, &OutputFormat::Table)
    }

    fn field(rendered: &str, key: &str) -> Option<String> {
        rendered.lines().find_map(|line| {
            let (found, rest) = line.split_once("  ")?;
            (found.trim_end() == key).then(|| rest.trim().to_string())
        })
    }

    fn completed_job() -> Value {
        json!({
            "job_id": "job_01JQ8ZK",
            "model": "kling-26-pro",
            "quality": "pro",
            "status": "COMPLETED",
            "prompt": "A slow dolly-in on an astronaut cat",
            "outputs": [{
                "status": "COMPLETED",
                "asset_id": "asset_01JQ8ZM",
                "url": "https://storage.hedra.com/out.mp4?sig=abc",
                "content_type": "video/mp4",
                "width": 1920, "height": 1080, "duration_ms": 5000, "fps": 24
            }],
            "metrics": { "processing_time_ms": 84213 },
            "error": null,
            "logs": [
                {"id": 1, "timestamp": "2026-08-20T18:40:02.113Z", "level": "INFO",
                 "event": "queued", "message": "Job accepted and queued",
                 "source": "hedra", "data": {}},
                {"id": 2, "timestamp": "2026-08-20T18:41:46.327Z", "level": "INFO",
                 "event": "completed", "message": "Generation completed",
                 "source": "hedra", "data": {"outputs": 1}}
            ],
            "cost": 0.42,
            "currency": "USD"
        })
    }

    // ── The bug being fixed ─────────────────────────────────────────

    #[test]
    fn untransformed_job_result_renders_only_its_logs() {
        // Guards the premise: without the hook the renderer picks `logs`
        // (alphabetically first non-empty array) and drops everything
        // else. If this ever fails, the generic renderer was fixed
        // upstream and this module can be reconsidered.
        let rendered = render(&completed_job());
        assert!(rendered.contains("Job accepted and queued"));
        assert!(!rendered.contains("job_01JQ8ZK"));
        assert!(!rendered.contains("COMPLETED"));
        assert!(!rendered.contains("storage.hedra.com"));
    }

    #[test]
    fn job_result_keeps_every_field_the_renderer_dropped() {
        let rendered = render(&job_result(&completed_job()));
        assert_eq!(field(&rendered, "job").as_deref(), Some("job_01JQ8ZK"));
        assert_eq!(field(&rendered, "status").as_deref(), Some("COMPLETED"));
        assert_eq!(
            field(&rendered, "model").as_deref(),
            Some("kling-26-pro  (pro)")
        );
        assert_eq!(field(&rendered, "cost").as_deref(), Some("0.42 USD"));
        assert_eq!(field(&rendered, "elapsed").as_deref(), Some("1m 24s"));
        assert_eq!(
            field(&rendered, "prompt").as_deref(),
            Some("A slow dolly-in on an astronaut cat")
        );
        // Logs survive too — the fix adds fields, it does not trade one
        // set of dropped data for another.
        assert!(rendered.contains("Job accepted and queued"));
        assert!(rendered.contains("Generation completed"));
    }

    #[test]
    fn output_url_is_never_truncated() {
        // A presigned URL that cannot be copied is worse than one that
        // wraps, so the URL gets its own untruncated row.
        let rendered = render(&job_result(&completed_job()));
        assert_eq!(
            field(&rendered, "outputs.1.url").as_deref(),
            Some("https://storage.hedra.com/out.mp4?sig=abc")
        );
        assert_eq!(
            field(&rendered, "outputs.1.asset").as_deref(),
            Some("asset_01JQ8ZM")
        );
        assert_eq!(
            field(&rendered, "outputs.1").as_deref(),
            Some("video/mp4  1920x1080  5.0s")
        );
    }

    #[test]
    fn expired_output_says_so_rather_than_vanishing() {
        let mut job = completed_job();
        job["outputs"][0]["url"] = Value::Null;
        let rendered = render(&job_result(&job));
        assert_eq!(
            field(&rendered, "outputs.1.url").as_deref(),
            Some("(expired)")
        );
    }

    // ── Failure is the case that mattered most ──────────────────────

    #[test]
    fn failed_job_surfaces_the_error_envelope() {
        let mut job = completed_job();
        job["status"] = json!("FAILED");
        job["outputs"] = json!([]);
        job["metrics"] = Value::Null;
        job["error"] = json!({
            "code": "PROVIDER_ERROR",
            "message": "Upstream model returned 503 after 3 attempts",
            "retryable": true,
            "retry_after": 30,
            "param": "input.duration",
            "details": [
                {"param": "input.duration", "message": "must be <= 10"},
                {"param": "input.fps", "message": "unsupported value"}
            ]
        });
        let rendered = render(&job_result(&job));
        assert_eq!(field(&rendered, "status").as_deref(), Some("FAILED"));
        assert_eq!(
            field(&rendered, "error").as_deref(),
            Some("PROVIDER_ERROR: Upstream model returned 503 after 3 attempts")
        );
        assert_eq!(
            field(&rendered, "error.retry").as_deref(),
            Some("yes (after 30s)")
        );
        assert_eq!(
            field(&rendered, "error.param").as_deref(),
            Some("input.duration")
        );
        assert_eq!(
            field(&rendered, "error.detail.1").as_deref(),
            Some("input.duration: must be <= 10")
        );
        assert_eq!(
            field(&rendered, "error.detail.2").as_deref(),
            Some("input.fps: unsupported value")
        );
    }

    #[test]
    fn queued_job_omits_fields_it_has_no_value_for() {
        let mut job = completed_job();
        job["status"] = json!("IN_QUEUE");
        job["outputs"] = json!([]);
        job["metrics"] = Value::Null;
        job["cost"] = Value::Null;
        job["currency"] = Value::Null;
        let rendered = render(&job_result(&job));
        assert_eq!(field(&rendered, "status").as_deref(), Some("IN_QUEUE"));
        // Absent rather than blank: an empty `cost` row reads as "free".
        assert!(field(&rendered, "cost").is_none());
        assert!(field(&rendered, "elapsed").is_none());
        assert!(field(&rendered, "outputs.1").is_none());
    }

    #[test]
    fn job_without_logs_still_renders_the_summary() {
        let mut job = completed_job();
        job.as_object_mut().unwrap().remove("logs");
        let rendered = render(&job_result(&job));
        assert_eq!(field(&rendered, "job").as_deref(), Some("job_01JQ8ZK"));
        assert_eq!(
            field(&rendered, "outputs.1.asset").as_deref(),
            Some("asset_01JQ8ZM")
        );
    }

    #[test]
    fn log_messages_line_up_in_a_column() {
        // Regression: `pad` used to trim its own trailing whitespace,
        // which stripped the padding off the last column it produced —
        // so the appended message started at a different offset on every
        // row, in exactly the rows whose event names differed in length.
        let mut job = completed_job();
        job["logs"] = json!([
            {"id": 1, "timestamp": "2026-08-20T18:40:02.113Z", "level": "INFO",
             "event": "queued", "message": "first", "source": "hedra", "data": {}},
            {"id": 2, "timestamp": "2026-08-20T18:41:46.327Z", "level": "ERROR",
             "event": "completed", "message": "second", "source": "hedra", "data": {}}
        ]);
        let rendered = render(&job_result(&job));
        let offsets: Vec<usize> = ["first", "second"]
            .iter()
            .map(|needle| {
                let line = rendered
                    .lines()
                    .find(|l| l.contains(needle))
                    .expect("message row present");
                line.find(needle).unwrap()
            })
            .collect();
        assert_eq!(
            offsets[0], offsets[1],
            "messages start at different columns:\n{rendered}"
        );
    }

    // ── Key ordering ────────────────────────────────────────────────

    #[test]
    fn tenth_log_sorts_after_the_second() {
        // The renderer sorts keys as strings, so unpadded indices would
        // order 1, 10, 11, 2, ... — the events would read out of sequence.
        let logs: Vec<Value> = (1..=12)
            .map(|i| {
                json!({
                    "id": i,
                    "timestamp": format!("2026-08-20T18:{:02}:00.000Z", 40 + i),
                    "level": "INFO", "event": "progress",
                    "message": format!("step {i}"), "source": "hedra", "data": {}
                })
            })
            .collect();
        let mut job = completed_job();
        job["logs"] = Value::Array(logs);

        let rendered = render(&job_result(&job));
        let order: Vec<usize> = rendered
            .lines()
            .filter_map(|l| l.split_once("  "))
            .filter_map(|(k, _)| k.trim_end().strip_prefix("logs.")?.parse().ok())
            .collect();
        assert_eq!(order, (1..=12).collect::<Vec<usize>>());
    }

    // ── Submit ──────────────────────────────────────────────────────

    #[test]
    fn submit_replaces_server_paths_with_runnable_commands() {
        let ack = json!({
            "job_id": "job_01JQ8ZK",
            "model": "kling-26-pro",
            "status": "IN_QUEUE",
            "status_url": "/v3/jobs/job_01JQ8ZK/status",
            "result_url": "/v3/jobs/job_01JQ8ZK",
            "estimated_completion_at": "2026-08-20T18:42:11.482913Z"
        });
        let rendered = render(&job_submit(&ack));
        assert_eq!(
            field(&rendered, "next").as_deref(),
            Some("hedra-cli jobs get job_01JQ8ZK")
        );
        assert_eq!(
            field(&rendered, "watch").as_deref(),
            Some("hedra-cli jobs stream job_01JQ8ZK")
        );
        assert_eq!(
            field(&rendered, "eta").as_deref(),
            Some("2026-08-20 18:42:11 UTC")
        );
        // The bare server paths are gone — they were not runnable.
        assert!(!rendered.contains("/v3/jobs"));
    }

    #[test]
    fn submit_without_an_estimate_omits_the_eta() {
        let ack = json!({
            "job_id": "job_01JQ8ZK", "model": "flux-dev", "status": "IN_QUEUE",
            "status_url": "/v3/jobs/job_01JQ8ZK/status",
            "result_url": "/v3/jobs/job_01JQ8ZK",
            "estimated_completion_at": Value::Null
        });
        let rendered = render(&job_submit(&ack));
        assert!(field(&rendered, "eta").is_none());
        assert_eq!(field(&rendered, "model").as_deref(), Some("flux-dev"));
    }

    // ── Status ──────────────────────────────────────────────────────

    #[test]
    fn status_renders_progress_as_a_percentage() {
        // The API sends a 0..1 fraction, which reads as a rounding error
        // when printed raw.
        let status = json!({
            "job_id": "job_01JQ8ZK",
            "status": "IN_PROGRESS",
            "progress": 0.5,
            "estimated_completion_at": "2026-08-20T18:42:11Z",
            "logs": [{"id": 1, "timestamp": "2026-08-20T18:40:02.113Z",
                      "level": "INFO", "event": "started",
                      "message": "Generation started", "source": "hedra", "data": {}}]
        });
        let rendered = render(&job_status(&status));
        assert_eq!(field(&rendered, "progress").as_deref(), Some("50%"));
        assert_eq!(field(&rendered, "status").as_deref(), Some("IN_PROGRESS"));
        assert!(rendered.contains("Generation started"));
    }

    // ── Pass-through ────────────────────────────────────────────────

    #[tokio::test]
    async fn jobs_list_is_left_to_the_generic_renderer() {
        let list = json!({
            "data": [{"job_id": "job_a", "model": "flux-dev", "status": "COMPLETED",
                      "created_at": "2026-08-20T18:40:02.113Z"}],
            "next_cursor": Value::Null
        });
        let path = vec!["jobs".to_string(), "list".to_string()];
        assert_eq!(jobs(list.clone(), path).await.unwrap(), list);
    }

    #[tokio::test]
    async fn every_submit_model_variant_takes_the_submit_view() {
        let ack = json!({
            "job_id": "job_01JQ8ZK", "model": "gpt-image-2", "status": "IN_QUEUE",
            "status_url": "/v3/jobs/job_01JQ8ZK/status",
            "result_url": "/v3/jobs/job_01JQ8ZK"
        });
        for op in ["submit", "submit-gpt-image-2", "submit-kling-26-pro"] {
            let path = vec!["jobs".to_string(), op.to_string()];
            let out = jobs(ack.clone(), path).await.unwrap();
            assert_eq!(
                out.get("next").and_then(Value::as_str),
                Some("hedra-cli jobs get job_01JQ8ZK"),
                "{op} did not take the submit view"
            );
        }
    }

    // ── Layer 1: de-hijacking ───────────────────────────────────────

    #[test]
    fn keys_create_stops_dropping_the_credential() {
        // The worst instance of the bug: `scopes` sorts first, so the one
        // field that can never be retrieved again was the one discarded.
        let created = json!({
            "key_id": "key_01JQ8ZK",
            "credential": "key_01JQ8ZK:sk-live-9f2c",
            "kind": "secret",
            "name": "ci",
            "scopes": ["jobs:write", "jobs:read"],
            "workspace_id": "ws_01JQ8ZK",
            "expires_at": Value::Null,
            "created_at": "2026-08-20T18:40:02.113Z"
        });
        assert!(!render(&created).contains("sk-live-9f2c"));

        let rendered = render(&dehijack(created));
        assert_eq!(
            field(&rendered, "credential").as_deref(),
            Some("key_01JQ8ZK:sk-live-9f2c")
        );
        // Scalar arrays read better joined than as indexed rows.
        assert_eq!(
            field(&rendered, "scopes").as_deref(),
            Some("jobs:write, jobs:read")
        );
    }

    #[test]
    fn arrays_of_objects_become_indexed_rows() {
        let redelivery = json!({
            "job_id": "job_a",
            "status": "delivered",
            "attempts": 3,
            "webhook_url": "https://example.test/hook",
            "redeliveries": [
                {"attempt": 1, "response_status": 500},
                {"attempt": 2, "response_status": 200}
            ]
        });
        let rendered = render(&dehijack(redelivery));
        assert_eq!(
            field(&rendered, "webhook_url").as_deref(),
            Some("https://example.test/hook")
        );
        assert_eq!(
            field(&rendered, "redeliveries.1.attempt").as_deref(),
            Some("1")
        );
        assert_eq!(
            field(&rendered, "redeliveries.2.response_status").as_deref(),
            Some("200")
        );
    }

    #[test]
    fn genuine_list_envelopes_are_left_alone() {
        // `keys list` is the shape the heuristic was written for; its
        // array table is correct and must survive.
        let list = json!({
            "data": [{"key_id": "key_a", "name": "ci"}],
            "next_cursor": Value::Null
        });
        assert_eq!(dehijack(list.clone()), list);
    }

    #[test]
    fn a_single_resource_with_a_data_array_is_not_a_list() {
        // `billing get-usage` carries a `data` array *and* six summary
        // fields, so it is not a list envelope however much it looks like
        // one at a glance.
        let usage = json!({
            "start": "2026-08-01", "end": "2026-08-20", "group_by": "model",
            "total_jobs": 412, "total_spent": 93.4, "currency": "USD",
            "data": [{"model": "flux-dev", "jobs": 12, "spent": 1.2}]
        });
        assert!(!render(&usage).contains("93.4"));

        let rendered = render(&dehijack(usage));
        assert_eq!(field(&rendered, "total_spent").as_deref(), Some("93.4"));
        assert_eq!(field(&rendered, "total_jobs").as_deref(), Some("412"));
        assert_eq!(
            field(&rendered, "data.1.model").as_deref(),
            Some("flux-dev")
        );
    }

    #[test]
    fn empty_arrays_do_not_produce_phantom_rows() {
        let drain = json!({
            "id": "ld_01", "name": "siem", "enabled": true,
            "header_names": [], "batch_size": 100
        });
        let rendered = render(&dehijack(drain));
        assert_eq!(field(&rendered, "name").as_deref(), Some("siem"));
        assert_eq!(field(&rendered, "batch_size").as_deref(), Some("100"));
    }

    // ── Primitives ──────────────────────────────────────────────────

    #[test]
    fn durations_scale_with_magnitude() {
        assert_eq!(duration(0), "0.0s");
        assert_eq!(duration(5000), "5.0s");
        assert_eq!(duration(59_900), "59.9s");
        assert_eq!(duration(84_213), "1m 24s");
        assert_eq!(duration(3_600_000), "1h 0m");
        assert_eq!(duration(7_265_000), "2h 1m");
    }

    #[test]
    fn timestamps_are_labelled_not_silently_ambiguous() {
        assert_eq!(
            instant("2026-08-20T18:42:11.482913Z"),
            "2026-08-20 18:42:11 UTC"
        );
        assert_eq!(
            instant("2026-08-20T18:42:11+00:00"),
            "2026-08-20 18:42:11 UTC"
        );
        // An offset we cannot convert is passed through rather than
        // mislabelled as UTC.
        assert_eq!(instant("2026-08-20T18:42:11+02:00"), "2026-08-20 18:42:11");
        assert_eq!(instant("garbage"), "garbage");
        assert_eq!(clock("2026-08-20T18:40:02.113Z"), "18:40:02Z");
        assert_eq!(clock("nope"), "nope");
    }

    #[test]
    fn multiline_text_is_flattened_so_alignment_survives() {
        // A prompt with a newline would otherwise start a row the
        // renderer never padded, shifting every column after it.
        let flattened = one_line("first line\nsecond\tline", 96);
        assert_eq!(flattened, "first line second line");
        assert!(!flattened.contains('\n'));
        assert_eq!(one_line("abcdef", 4), "abc…");
    }

    #[test]
    fn multibyte_text_clips_on_char_boundaries() {
        // Byte-slicing "日本語テキスト" here would panic.
        assert_eq!(one_line("日本語テキスト", 4), "日本語…");
    }
}
