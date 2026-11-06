//! `exec` on a Windows console and on a closed pipe, run as the built binary
//! against a mock v3 API and a fake broker.
//!
//! `cargo test`'s handles are pipes and every test in one process shares one
//! console, so each console test re-runs this test binary as a host process
//! (`HEDRA_TEST_CONSOLE_HOST`), which allocates a console of its own with code
//! page 437, runs hedra-cli on it, types keys into its input buffer and writes
//! the screen buffer and the console modes back for the test to check.

#![cfg(windows)]

mod sandbox_stream_support;

use std::fs;
use std::io::Read;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use hedra_cli_sandbox::console_host::{self, ConsoleHost, Key};
use sandbox_stream_support::*;
use serde_json::{json, Value};

const WAIT: Duration = Duration::from_secs(30);
const HOST: &str = "HEDRA_TEST_CONSOLE_HOST";
const DETACHED_PROCESS: u32 = 0x0000_0008;
const VK_UP: u16 = 0x26;
const ENABLE_VIRTUAL_TERMINAL_INPUT: u32 = 0x0200;

fn exec_args<'a>(extra: &[&'a str]) -> Vec<&'a str> {
    let mut args = vec!["sandbox", "exec", ID];
    args.extend_from_slice(extra);
    args
}

/// What the host process does: the command, the keys to type once the
/// command has put the console in raw mode, and whether to hold the command
/// until the test writes `done`.
fn config(cmd: &Command, keys: &[Key], hold: bool) -> Value {
    let keys: Vec<Value> = keys
        .iter()
        .map(|k| match *k {
            Key::Char(c) => json!({ "char": c.to_string() }),
            Key::Ctrl(c) => json!({ "ctrl": c.to_string() }),
            Key::Virtual(vk) => json!({ "vk": vk }),
        })
        .collect();
    json!({
        "program": cmd.get_program().to_string_lossy(),
        "args": cmd.get_args().map(|a| a.to_string_lossy()).collect::<Vec<_>>(),
        "env": cmd
            .get_envs()
            .filter_map(|(k, v)| Some((k.to_string_lossy(), v?.to_string_lossy())))
            .collect::<Vec<_>>(),
        "cwd": cmd.get_current_dir().map(|d| d.to_string_lossy().into_owned()),
        "keys": keys,
        "hold": hold,
    })
}

/// Runs as the host when this process was started as one, and says whether
/// it was.
fn host_main() -> bool {
    let Some(dir) = std::env::var_os(HOST) else {
        return false;
    };
    let dir = Path::new(&dir);
    let cfg: Value = serde_json::from_slice(&fs::read(dir.join("config.json")).unwrap()).unwrap();
    let host = ConsoleHost::allocate().unwrap();
    let before = host.modes().unwrap();

    let mut cmd = Command::new(cfg["program"].as_str().unwrap());
    cmd.env_clear();
    for a in cfg["args"].as_array().unwrap() {
        cmd.arg(a.as_str().unwrap());
    }
    for kv in cfg["env"].as_array().unwrap() {
        cmd.env(kv[0].as_str().unwrap(), kv[1].as_str().unwrap());
    }
    if let Some(cwd) = cfg["cwd"].as_str() {
        cmd.current_dir(cwd);
    }
    let mut child = host.spawn(&mut cmd).unwrap();

    let keys: Vec<Key> = cfg["keys"]
        .as_array()
        .unwrap()
        .iter()
        .map(|k| {
            if let Some(c) = k["char"].as_str() {
                Key::Char(c.chars().next().unwrap())
            } else if let Some(c) = k["ctrl"].as_str() {
                Key::Ctrl(c.chars().next().unwrap())
            } else {
                Key::Virtual(k["vk"].as_u64().unwrap() as u16)
            }
        })
        .collect();
    if !keys.is_empty() {
        // Type only once the command has put the input handle in raw mode.
        let deadline = Instant::now() + WAIT;
        while host.modes().unwrap().0 & ENABLE_VIRTUAL_TERMINAL_INPUT == 0 {
            assert!(Instant::now() < deadline, "the console never went raw");
            std::thread::sleep(Duration::from_millis(20));
        }
        host.type_keys(&keys).unwrap();
    }

    let code = if cfg["hold"].as_bool().unwrap() {
        let deadline = Instant::now() + WAIT;
        while !dir.join("done").exists() && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        let _ = child.kill();
        let _ = child.wait();
        None
    } else {
        console_host::wait(&mut child, WAIT).unwrap()
    };
    let after = host.modes().unwrap();
    let result = json!({
        "code": code,
        "screen": host.screen().unwrap(),
        "before": [before.0, before.1],
        "after": [after.0, after.1],
    });
    fs::write(dir.join("result.json"), result.to_string()).unwrap();
    true
}

/// A host process running one console test's command.
struct Hosted {
    dir: tempfile::TempDir,
    host: std::process::Child,
}

impl Hosted {
    /// Re-runs this test binary as the host for `test`.
    fn start(test: &str, cmd: &Command, keys: &[Key], hold: bool) -> Self {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("config.json"),
            config(cmd, keys, hold).to_string(),
        )
        .unwrap();
        let host = Command::new(std::env::current_exe().unwrap())
            .args([test, "--exact", "--nocapture", "--test-threads=1"])
            .env(HOST, dir.path())
            .creation_flags(DETACHED_PROCESS)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        Self { dir, host }
    }

    /// Tells a holding host to end its command.
    fn release(&self) {
        fs::write(self.dir.path().join("done"), b"").unwrap();
    }

    /// Waits for the host and returns what it wrote back.
    fn finish(self) -> Value {
        let out = wait(self.host, WAIT + WAIT);
        assert!(out.status.success(), "the host failed: {}", stderr(&out));
        serde_json::from_slice(&fs::read(self.dir.path().join("result.json")).unwrap()).unwrap()
    }
}

fn position(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

#[tokio::test]
async fn keys_typed_at_a_console_arrive_as_utf8_ctrl_z_as_0x1a_and_up_as_its_escape_sequence() {
    if host_main() {
        return;
    }
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([Step::Start, Step::Hold]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cmd = run.command(&exec_args(&["-t", "--", "cat"]));
    let hosted = Hosted::start(
        "keys_typed_at_a_console_arrive_as_utf8_ctrl_z_as_0x1a_and_up_as_its_escape_sequence",
        &cmd,
        &[Key::Char('é'), Key::Ctrl('z'), Key::Virtual(VK_UP)],
        true,
    );

    let calls = broker.wait_for(WAIT, |c| {
        c.first()
            .is_some_and(|c| position(&c.stdin, b"\x1b[A").is_some())
    });
    hosted.release();
    let _ = hosted.finish();

    let stdin = &calls[0].stdin;
    assert!(calls[0].start.as_ref().unwrap().tty);
    let e = position(stdin, "é".as_bytes()).unwrap_or_else(|| panic!("no é: {stdin:?}"));
    let z = position(stdin, b"\x1a").unwrap_or_else(|| panic!("no 0x1A: {stdin:?}"));
    let up = position(stdin, b"\x1b[A").unwrap();
    assert!(e < z && z < up, "{stdin:?}");
}

#[tokio::test]
async fn output_on_a_code_page_437_console_prints_utf8_and_replaces_invalid_bytes() {
    if host_main() {
        return;
    }
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout("héllo ✓\n".as_bytes().to_vec()),
        Step::Stderr("wörld ✓\n".as_bytes().to_vec()),
        Step::Stdout(b"bad \xff\xfe end\n".to_vec()),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cmd = run.command(&exec_args(&["-n", "--", "train"]));
    let result = Hosted::start(
        "output_on_a_code_page_437_console_prints_utf8_and_replaces_invalid_bytes",
        &cmd,
        &[],
        false,
    )
    .finish();

    assert_eq!(result["code"], json!(0), "{result}");
    let screen = result["screen"].as_str().unwrap();
    assert!(screen.contains("héllo ✓"), "{screen}");
    assert!(screen.contains("wörld ✓"), "{screen}");
    let bad = screen
        .lines()
        .find(|l| l.starts_with("bad "))
        .unwrap_or_else(|| panic!("no bad line: {screen}"));
    assert!(bad.contains('\u{FFFD}') && bad.ends_with(" end"), "{bad:?}");
}

#[tokio::test]
async fn exec_t_on_a_console_writes_both_console_modes_back_on_exit() {
    if host_main() {
        return;
    }
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::Stdout(b"done\r\n".to_vec()),
        Step::Exit(0),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    let cmd = run.command(&exec_args(&["-t", "--", "true"]));
    let result = Hosted::start(
        "exec_t_on_a_console_writes_both_console_modes_back_on_exit",
        &cmd,
        &[],
        false,
    )
    .finish();

    assert_eq!(result["code"], json!(0), "{result}");
    assert_eq!(result["after"], result["before"], "{result}");
}

#[tokio::test]
async fn exec_piped_into_a_closed_reader_exits_141_with_no_stderr_line() {
    let broker = FakeBroker::start();
    broker.push_exec(ExecScript::admit([
        Step::Start,
        Step::StdoutForever(b"y\n".to_vec(), Duration::from_millis(10)),
    ]));
    let server = running_sandbox(&broker).await;
    let run = Run::new(&server);
    // exec … | head -c1
    let mut child = run.spawn(&exec_args(&["-n", "--", "yes"]));
    let mut first = [0u8; 1];
    child
        .stdout
        .as_mut()
        .unwrap()
        .read_exact(&mut first)
        .unwrap();
    drop(child.stdout.take());
    let out = wait(child, WAIT);
    assert_eq!(out.status.code(), Some(141), "{}", stderr(&out));
    assert!(stderr(&out).is_empty(), "{}", stderr(&out));
    broker.wait_for(WAIT, |c| c[0].client_gone);
}
