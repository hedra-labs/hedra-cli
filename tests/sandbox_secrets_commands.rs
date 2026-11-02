//! Exercise the preview scaffold through the shipped binary, without secrets.
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};

use tempfile::TempDir;
use wiremock::MockServer;

fn run(args: &[&str], preview: Option<&str>, base_url: Option<&str>) -> Output {
    let directory = TempDir::new().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_hedra-cli"));
    command
        .env_clear()
        .current_dir(directory.path())
        .env("NO_COLOR", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(preview) = preview {
        command.env("HEDRA_CLI_SANDBOX_PREVIEW", preview);
    }
    if let Some(url) = base_url {
        command.args(["--base-url", url]);
    }
    let mut child = command.args(args).spawn().unwrap();
    // Hold the pipe open without supplying any bytes: a stub that reads stdin
    // would hang. No real credentials, HOME, or .env are available to this child.
    let _stdin = child.stdin.take().unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            let _ = child.wait();
            panic!("command blocked with stdin open: {args:?}");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

fn combined(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[tokio::test]
async fn stubs_fail_without_reading_stdin_files_or_contacting_the_api() {
    let server = MockServer::start().await;
    for args in [
        vec!["set", "TOKEN"],
        vec!["set", "TOKEN", "--from-file", "/does-not-exist/token"],
        vec!["set", "TOKEN", "--debug"],
        vec!["set", "TOKEN", "--dry-run"],
        vec!["ls"],
        vec!["rm", "TOKEN"],
        vec!["rm", "TOKEN", "--yes", "--quiet"],
    ] {
        let uri = server.uri();
        let output = tokio::task::spawn_blocking(move || {
            let mut command = vec!["sandbox", "secrets"];
            command.extend(args);
            run(&command, Some("1"), Some(&uri))
        })
        .await
        .unwrap();
        assert_eq!(output.status.code(), Some(3), "{}", combined(&output));
        let body: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(body["error"]["message"], "Secrets are not available yet.");
        assert!(output.stderr.is_empty(), "{}", combined(&output));
    }
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[test]
fn preview_is_hidden_and_refuses_execution_unless_explicitly_enabled() {
    for preview in [None, Some("0"), Some("true")] {
        let help = run(&["--help"], preview, None);
        assert!(help.status.success());
        assert!(!combined(&help).contains("sandbox"));
        let output = run(&["sandbox", "secrets", "ls"], preview, None);
        assert_eq!(output.status.code(), Some(3));
        assert!(combined(&output).contains("Set HEDRA_CLI_SANDBOX_PREVIEW=1"));
    }
    let help = run(&["--help"], Some("1"), None);
    assert!(help.status.success());
    assert!(combined(&help).contains("sandbox"));
}

#[test]
fn help_reserves_the_interface_and_parser_rejects_a_missing_name() {
    let help = run(&["sandbox", "secrets", "--help"], Some("1"), None);
    assert!(help.status.success());
    let text = combined(&help);
    for expected in ["set", "ls", "rm", "not available yet"] {
        assert!(text.contains(expected), "{text}");
    }
    for verb in ["set", "rm"] {
        let output = run(&["sandbox", "secrets", verb], Some("1"), None);
        assert_eq!(output.status.code(), Some(3));
        assert!(combined(&output).contains("NAME"));
    }
}
