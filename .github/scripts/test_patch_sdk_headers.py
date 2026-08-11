#!/usr/bin/env python3
"""Tests for patch-sdk-headers.py.

    python3 -m unittest discover -s .github/scripts

Stdlib only, and no network or cargo: the script is pure text manipulation, and
these run anywhere python3 does. The script runs on every fern-bot PR, so the
cases that matter most are the ones where it must NOT write — an already-patched
executor, a config already at the right version — and the ones where it must
refuse loudly rather than guess: a regenerated shape the anchors no longer
match. The Rust the script injects is compile-checked by `cargo test --locked`
(tests/wire_contract.rs), not here.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import os
import tempfile
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).resolve().parent / "patch-sdk-headers.py"
_spec = importlib.util.spec_from_file_location("patch_sdk_headers", _SCRIPT)
ph = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(ph)


# Trimmed from hedra-sdk/src/config.rs as the generator emits it: the tuple
# lines carry the exact spacing and trailing comma the anchors must match.
CONFIG_RS = '''\
use crate::Environment;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ClientConfig {
    pub base_url: String,
    pub custom_headers: HashMap<String, String>,
    pub user_agent: String,
}
impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: Environment::default().url().to_string(),
            timeout: Duration::from_secs(60),
            max_retries: 3,
            custom_headers: HashMap::from([
                ("X-Fern-Language".to_string(), "Rust".to_string()),
                ("X-Fern-SDK-Name".to_string(), "hedra_sdk".to_string()),
                ("X-Fern-SDK-Version".to_string(), "0.1.0".to_string()),
            ]),
            user_agent: "Api Rust SDK".to_string(),
        }
    }
}
'''

# Trimmed from src/sdk_executor.rs as the generator emits it: `new()` with the
# `build_client` expect immediately followed by the struct literal.
EXECUTOR_RS = '''\
pub struct CliExecutor {
    client: Client,
    auth_provider: DynAuthProvider,
    global_headers: Vec<(String, String)>,
    base_url_override: Option<String>,
    retries: RetriesConfig,
}

impl CliExecutor {
    /// Create a new executor wired to the CLI's runtime context.
    pub fn new(
        http_config: HttpConfig,
        auth_provider: DynAuthProvider,
        global_headers: Vec<(String, String)>,
        base_url_override: Option<String>,
    ) -> Self {
        let client = http_config
            .build_client()
            .expect("HttpConfig::build_client failed");
        Self {
            client,
            auth_provider,
            global_headers,
            base_url_override,
            retries: RetriesConfig::default(),
        }
    }
}
'''


def run(*argv):
    """Invoke main() with argv, capturing (exit_code, stdout)."""
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = ph.main(["patch-sdk-headers.py", *argv])
    return code, out.getvalue()


class ReadSdkVersion(unittest.TestCase):
    def test_reads_the_generated_literal(self):
        self.assertEqual(ph.read_sdk_version(CONFIG_RS, "config.rs"), "0.1.0")

    def test_reads_a_prerelease_version(self):
        source = CONFIG_RS.replace('"0.1.0"', '"1.0.0-dev"')
        self.assertEqual(ph.read_sdk_version(source, "config.rs"), "1.0.0-dev")

    def test_no_literal(self):
        source = CONFIG_RS.replace("X-Fern-SDK-Version", "X-Fern-SDK-Vers")
        with self.assertRaises(SystemExit) as cm:
            ph.read_sdk_version(source, "config.rs")
        self.assertIn("no X-Fern-SDK-Version literal found", str(cm.exception))

    def test_two_literals(self):
        line = '                ("X-Fern-SDK-Version".to_string(), "0.1.0".to_string()),\n'
        source = CONFIG_RS.replace(line, line + line)
        with self.assertRaises(SystemExit) as cm:
            ph.read_sdk_version(source, "config.rs")
        self.assertIn("refusing to guess", str(cm.exception))


class SetSdkVersion(unittest.TestCase):
    def test_replaces_the_version(self):
        out = ph.set_sdk_version(CONFIG_RS, "1.0.0-dev", "config.rs")
        self.assertIn('("X-Fern-SDK-Version".to_string(), "1.0.0-dev".to_string()),', out)
        self.assertNotIn('"0.1.0"', out)

    def test_leaves_language_and_name_alone(self):
        out = ph.set_sdk_version(CONFIG_RS, "1.0.0-dev", "config.rs")
        self.assertIn('("X-Fern-Language".to_string(), "Rust".to_string()),', out)
        self.assertIn('("X-Fern-SDK-Name".to_string(), "hedra_sdk".to_string()),', out)

    def test_leaves_every_other_byte_alone(self):
        out = ph.set_sdk_version(CONFIG_RS, "1.0.0-dev", "config.rs")
        self.assertEqual(out.replace('"1.0.0-dev".to_string()', '"0.1.0".to_string()'), CONFIG_RS)

    def test_preserves_indentation(self):
        out = ph.set_sdk_version(CONFIG_RS, "2.0.0", "config.rs")
        self.assertIn('                ("X-Fern-SDK-Version".to_string(), "2.0.0".to_string()),', out)

    def test_is_idempotent(self):
        once = ph.set_sdk_version(CONFIG_RS, "1.0.0-dev", "config.rs")
        twice = ph.set_sdk_version(once, "1.0.0-dev", "config.rs")
        self.assertEqual(once, twice)

    def test_round_trips_with_read(self):
        out = ph.set_sdk_version(CONFIG_RS, "4.5.6-rc.1", "config.rs")
        self.assertEqual(ph.read_sdk_version(out, "config.rs"), "4.5.6-rc.1")


class ExecutorState(unittest.TestCase):
    def test_generated_shape_is_unpatched(self):
        self.assertEqual(ph.executor_state(EXECUTOR_RS, "sdk_executor.rs"), "unpatched")

    def test_patched_shape_is_patched(self):
        patched = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs")
        self.assertEqual(ph.executor_state(patched, "sdk_executor.rs"), "patched")

    def test_unrecognized_shape_refuses(self):
        source = EXECUTOR_RS.replace("HttpConfig::build_client failed", "build failed")
        with self.assertRaises(SystemExit) as cm:
            ph.executor_state(source, "sdk_executor.rs")
        self.assertIn("refusing to guess", str(cm.exception))

    def test_two_anchors_refuse(self):
        source = EXECUTOR_RS + EXECUTOR_RS
        with self.assertRaises(SystemExit) as cm:
            ph.executor_state(source, "sdk_executor.rs")
        self.assertIn("refusing to guess", str(cm.exception))

    def test_patched_plus_stray_anchor_refuses(self):
        source = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs") + EXECUTOR_RS
        with self.assertRaises(SystemExit) as cm:
            ph.executor_state(source, "sdk_executor.rs")
        self.assertIn("refusing to guess", str(cm.exception))


class PatchExecutor(unittest.TestCase):
    def test_inserts_the_merge(self):
        out = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs")
        self.assertIn("hedra_sdk::ClientConfig::default()", out)
        self.assertIn("global_headers.extend(identity);", out)

    def test_keeps_the_surrounding_constructor(self):
        out = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs")
        self.assertIn('.expect("HttpConfig::build_client failed");', out)
        self.assertIn("retries: RetriesConfig::default(),", out)

    def test_touches_nothing_outside_the_anchor(self):
        out = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs")
        self.assertEqual(out.replace(ph.EXECUTOR_PATCH, ph.EXECUTOR_ANCHOR), EXECUTOR_RS)

    def test_is_idempotent(self):
        once = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs")
        twice = ph.patch_executor(once, "sdk_executor.rs")
        self.assertEqual(once, twice)

    def test_names_the_removal_ticket(self):
        # The merge is a local fork of upstream behaviour; the marker is what a
        # future reader greps for when ENG-10234 lands and the fork comes out.
        self.assertIn("ENG-10234", ph.EXECUTOR_PATCH)


class Cli(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)
        self.config = self.write("config.rs", CONFIG_RS)
        self.executor = self.write("sdk_executor.rs", EXECUTOR_RS)

    def write(self, name, content):
        path = os.path.join(self.dir.name, name)
        Path(path).write_text(content, encoding="utf-8")
        return path

    def read(self, path):
        return Path(path).read_text(encoding="utf-8")

    # apply mode

    def test_apply_patches_both_files(self):
        code, out = run("1.0.0-dev", self.config, self.executor)
        self.assertEqual(code, 0)
        self.assertIn('"0.1.0" -> "1.0.0-dev"', out)
        self.assertIn("merged the identity headers", out)
        self.assertEqual(ph.read_sdk_version(self.read(self.config), self.config), "1.0.0-dev")
        self.assertEqual(ph.executor_state(self.read(self.executor), self.executor), "patched")

    def test_apply_is_a_no_op_on_the_second_run(self):
        run("1.0.0-dev", self.config, self.executor)
        snapshot = {p: self.read(p) for p in (self.config, self.executor)}
        code, out = run("1.0.0-dev", self.config, self.executor)
        self.assertEqual(code, 0)
        self.assertIn("(unchanged)", out)
        for path, content in snapshot.items():
            self.assertEqual(self.read(path), content)

    def test_apply_rejects_a_bad_version_before_touching_anything(self):
        before = self.read(self.config)
        with self.assertRaises(SystemExit):
            run("not-a-version", self.config, self.executor)
        self.assertEqual(self.read(self.config), before)

    def test_apply_reports_a_missing_file(self):
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0-dev", os.path.join(self.dir.name, "nope.rs"), self.executor)
        self.assertIn("nope.rs", str(cm.exception))

    def test_swapped_paths_die_loudly(self):
        # The executor file has no X-Fern-SDK-Version literal, so handing the
        # files over in the wrong order must refuse before writing anything.
        before = self.read(self.executor)
        with self.assertRaises(SystemExit):
            run("1.0.0-dev", self.executor, self.config)
        self.assertEqual(self.read(self.executor), before)

    # sequencing: Patch 1 (version) must never trail Patch 2 (executor merge)

    def test_version_is_stamped_even_when_the_executor_anchor_is_gone(self):
        # The safe failure direction: a stamped version with no merge is inert;
        # a merge with a stale version puts 0.1.0 on the wire.
        broken_source = EXECUTOR_RS.replace(
            '.expect("HttpConfig::build_client failed");', ".unwrap();"
        )
        broken = self.write("broken.rs", broken_source)
        with self.assertRaises(SystemExit):
            run("1.0.0-dev", self.config, broken)
        self.assertEqual(ph.read_sdk_version(self.read(self.config), self.config), "1.0.0-dev")
        self.assertEqual(self.read(broken), broken_source)

    # --check

    def test_check_fails_on_the_generated_tree(self):
        code, out = run("--check", "1.0.0-dev", self.config, self.executor)
        self.assertEqual(code, 1)
        self.assertIn('expected "1.0.0-dev"', out)
        self.assertIn("identity-header merge missing", out)
        self.assertIn("2 file(s) missing the ENG-10226 header patches", out)

    def test_check_passes_after_apply(self):
        run("1.0.0-dev", self.config, self.executor)
        code, out = run("--check", "1.0.0-dev", self.config, self.executor)
        self.assertEqual(code, 0)
        self.assertIn('X-Fern-SDK-Version = "1.0.0-dev"', out)
        self.assertIn("identity-header merge present", out)

    def test_check_never_writes(self):
        before = {p: self.read(p) for p in (self.config, self.executor)}
        run("--check", "1.0.0-dev", self.config, self.executor)
        for path, content in before.items():
            self.assertEqual(self.read(path), content)

    def test_check_catches_a_stale_version_behind_a_patched_executor(self):
        # The state Patch-2-without-Patch-1 would produce: merge present, wrong
        # literal. One drifted file, exit 1.
        run("1.0.0-dev", self.config, self.executor)
        self.write("config.rs", CONFIG_RS)
        code, out = run("--check", "1.0.0-dev", self.config, self.executor)
        self.assertEqual(code, 1)
        self.assertIn("1 file(s) missing the ENG-10226 header patches", out)

    def test_check_validates_the_version_argument(self):
        with self.assertRaises(SystemExit) as cm:
            run("--check", "1.0", self.config, self.executor)
        self.assertIn("not a Cargo-style SemVer version", str(cm.exception))

    # usage

    def test_usage_with_no_arguments(self):
        with self.assertRaises(SystemExit) as cm:
            run()
        self.assertIn("usage:", str(cm.exception))

    def test_usage_with_too_few_arguments(self):
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0-dev", self.config)
        self.assertIn("usage:", str(cm.exception))

    def test_usage_with_too_many_arguments(self):
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0-dev", self.config, self.executor, self.executor)
        self.assertIn("usage:", str(cm.exception))


class RegenerationScenario(unittest.TestCase):
    """The sequence refresh-lockfile.yml runs after every regeneration."""

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)
        self.config = os.path.join(self.dir.name, "config.rs")
        self.executor = os.path.join(self.dir.name, "sdk_executor.rs")

    def regenerate(self):
        Path(self.config).write_text(CONFIG_RS, encoding="utf-8")
        Path(self.executor).write_text(EXECUTOR_RS, encoding="utf-8")

    def test_regenerate_apply_check_reapply(self):
        for cycle in range(2):
            with self.subTest(cycle=cycle):
                # Regeneration resets both files to the generated shape.
                self.regenerate()
                code, _ = run("--check", "1.0.0-dev", self.config, self.executor)
                self.assertEqual(code, 1)

                # The workflow re-applies both patches in one step.
                code, _ = run("1.0.0-dev", self.config, self.executor)
                self.assertEqual(code, 0)
                self.assertEqual(run("--check", "1.0.0-dev", self.config, self.executor)[0], 0)

                # A re-run leaves the tree byte-identical, so the worktree
                # stays clean and refresh-lockfile.yml's push loop terminates.
                snapshot = {p: Path(p).read_text() for p in (self.config, self.executor)}
                run("1.0.0-dev", self.config, self.executor)
                for path, content in snapshot.items():
                    self.assertEqual(Path(path).read_text(), content)


if __name__ == "__main__":
    unittest.main()
