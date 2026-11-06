//! Text the stream verbs print about a sandbox: why it ended, how long a
//! wait took, and the generated commands that inspect or stop it.

use hedra_cli_sdk::api::Sandbox;

/// "2h 5m", "40s", "3d 4h": the two largest units.
pub(crate) fn human_duration(seconds: i64) -> String {
    let s = seconds.max(0);
    let (d, h, m, sec) = (s / 86_400, s % 86_400 / 3_600, s % 3_600 / 60, s % 60);
    match (d, h, m) {
        (0, 0, 0) => format!("{sec}s"),
        (0, 0, _) => format!("{m}m {sec}s"),
        (0, _, _) => format!("{h}h {m}m"),
        _ => format!("{d}d {h}h"),
    }
}

/// Why a sandbox ended, with the failed instance's reason for
/// `instance_failed`.
pub(crate) fn stop_reason(s: &Sandbox) -> Option<String> {
    let reason = s.end_reason.as_ref()?.to_string();
    let failure = s
        .instances
        .iter()
        .flatten()
        .find_map(|i| i.failure_reason.as_ref())
        .map(|f| f.to_string());
    Some(match (reason.as_str(), failure) {
        ("instance_failed", Some(f)) => format!("instance_failed ({f})"),
        _ => reason,
    })
}

/// The generated command that stops `sandbox`, a name or an id.
pub(crate) fn stop_command(sandbox: &str) -> String {
    format!("hedra-cli sandbox stop --sandbox {sandbox}")
}

/// The generated command that reads `sandbox`, a name or an id.
/// The generated command that lists the GPU models and what is free.
pub(crate) const GPUS_COMMAND: &str = "hedra-cli sandbox gpus";
/// The generated command that lists the workspace's sandboxes.
pub(crate) const LS_COMMAND: &str = "hedra-cli sandbox ls";

pub(crate) fn get_command(sandbox: &str) -> String {
    format!("hedra-cli sandbox get --sandbox {sandbox}")
}

/// The generated commands a recovery hint names, for the test that runs each
/// through the generated command tree.
#[cfg(test)]
pub(crate) fn hinted_commands() -> Vec<String> {
    vec![
        stop_command("train"),
        get_command("train"),
        GPUS_COMMAND.to_string(),
        LS_COMMAND.to_string(),
    ]
}

/// Parses `--max-duration`: whole seconds, or a number with `s`, `m`, `h`
/// or `d`.
pub(crate) fn parse_duration(raw: &str) -> Result<i64, String> {
    let raw = raw.trim();
    let (digits, unit) = match raw.char_indices().find(|(_, c)| !c.is_ascii_digit()) {
        Some((i, _)) => raw.split_at(i),
        None => (raw, "s"),
    };
    let n: i64 = digits
        .parse()
        .map_err(|_| format!("`{raw}` is not a duration such as 90m, 8h or 2d"))?;
    let scale = match unit {
        "s" => 1,
        "m" => 60,
        "h" => 3_600,
        "d" => 86_400,
        _ => return Err(format!("`{raw}` is not a duration such as 90m, 8h or 2d")),
    };
    n.checked_mul(scale)
        .ok_or_else(|| format!("`{raw}` is too long"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn durations_parse_with_and_without_units() {
        assert_eq!(parse_duration("90"), Ok(90));
        assert_eq!(parse_duration("90m"), Ok(5_400));
        assert_eq!(parse_duration("8h"), Ok(28_800));
        assert_eq!(parse_duration("2d"), Ok(172_800));
        assert!(parse_duration("2w").is_err());
        assert!(parse_duration("h").is_err());
        assert!(parse_duration("1.5h").is_err());
    }

    #[test]
    fn human_durations_show_two_units() {
        assert_eq!(human_duration(40), "40s");
        assert_eq!(human_duration(125), "2m 5s");
        assert_eq!(human_duration(7_500), "2h 5m");
        assert_eq!(human_duration(273_600), "3d 4h");
        assert_eq!(human_duration(-5), "0s");
    }

    #[test]
    fn instance_failed_names_the_instances_reason() {
        let s: Sandbox = serde_json::from_value(serde_json::json!({
            "id": "sandbox_1", "name": "train", "gpu": "h100:1", "gpu_model": "h100",
            "replicas": 1, "status": "ended", "end_reason": "instance_failed",
            "instances": [{"replica": 0, "status": "failed", "failure_reason": "image_pull_failed"}],
            "created_at": "2026-11-03T12:00:00Z", "env_keys": [],
            "billing": {"usd_per_hour": 1.0, "paid_usd": 0.0, "settled": false},
        }))
        .unwrap();
        assert_eq!(
            stop_reason(&s).as_deref(),
            Some("instance_failed (image_pull_failed)")
        );
    }

    #[test]
    fn the_printed_stop_command_is_the_generated_one() {
        assert_eq!(
            stop_command("train"),
            "hedra-cli sandbox stop --sandbox train"
        );
    }
}
