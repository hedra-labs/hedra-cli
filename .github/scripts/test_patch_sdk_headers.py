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


# The SDK crate's directory and the Rust identifier `sdk_crate()` derives from
# it. Real values, not placeholders: the patch bodies name the crate, and the
# ENG-10291 rename (hedra-sdk -> hedra-cli-sdk) is the exact drift that made a
# hardcoded name a defect.
SDK_DIR = "hedra-cli-sdk"
SDK_CRATE = "hedra_cli_sdk"

# `sdk_crate()` verifies the directory-derived name against the manifest rather
# than trusting it, so the fixture tree needs a real [package] table.
SDK_CARGO_TOML = f'''\
[package]
name = "{SDK_CRATE}"
version = "0.1.0"
edition = "2021"

[dependencies]
reqwest = "0.12"
'''

# Trimmed from hedra-cli-sdk/src/config.rs as the generator emits it: the tuple
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
                ("X-Fern-SDK-Name".to_string(), "hedra_cli_sdk".to_string()),
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


# Trimmed from src/http.rs as the generator emits it: the HeaderMap built and
# installed inside the User-Agent branch, so it exists only when the UA parses.
HTTP_RS = '''\
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

impl HttpConfig {
    pub fn build_client(&self) -> Result<reqwest::Client, CliError> {
        let prefix = &self.prefix;

        let mut builder = reqwest::Client::builder();
        let user_agent = self.user_agent();
        if let Ok(header_value) = HeaderValue::from_str(&user_agent) {
            let mut headers = HeaderMap::new();
            headers.insert(USER_AGENT, header_value);
            builder = builder.default_headers(headers);
        }

        for cert in &self.extra_root_certs {
            builder = builder.add_root_certificate(cert.clone());
        }
        builder.build().map_err(CliError::from)
    }
}
'''


def run(*argv):
    """Invoke main() with argv, capturing (exit_code, stdout)."""
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = ph.main(["patch-sdk-headers.py", *argv])
    return code, out.getvalue()


class Tree(unittest.TestCase):
    """A temp checkout laid out the way the script is actually invoked.

    `sdk_crate()` reads the crate name off `<crate-dir>/src/config.rs` and
    verifies it against `<crate-dir>/Cargo.toml`, both resolved relative to the
    process's cwd — exactly how both workflow call sites pass them. So the
    fixtures are a real directory tree entered with chdir, not flat temp files:
    an absolute path fails `parts[1] != "src"` before any patch is attempted,
    which is what left this suite 18-errors red and unnoticed (no workflow ran
    it) while the script it guards was being hand-repaired three times in a day.
    """

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)
        root = Path(self.dir.name)
        (root / SDK_DIR / "src").mkdir(parents=True)
        (root / "src").mkdir()
        (root / SDK_DIR / "Cargo.toml").write_text(SDK_CARGO_TOML, encoding="utf-8")

        # Relative, because that is what the script is given in CI.
        self.config = f"{SDK_DIR}/src/config.rs"
        self.http = "src/http.rs"
        self.executor = "src/sdk_executor.rs"

        cwd = os.getcwd()
        self.addCleanup(os.chdir, cwd)
        os.chdir(root)
        self.regenerate()

    def regenerate(self):
        """Reset every generated file to the shape fern emits."""
        self.write(self.config, CONFIG_RS)
        self.write(self.http, HTTP_RS)
        self.write(self.executor, EXECUTOR_RS)

    def write(self, name, content):
        Path(name).parent.mkdir(parents=True, exist_ok=True)
        Path(name).write_text(content, encoding="utf-8")
        return name

    def read(self, path):
        return Path(path).read_text(encoding="utf-8")

    @property
    def generated(self):
        """The three paths, in the order the script takes them."""
        return (self.config, self.http, self.executor)


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
        self.assertIn('("X-Fern-SDK-Name".to_string(), "hedra_cli_sdk".to_string()),', out)

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
        self.assertEqual(
            ph.executor_state(EXECUTOR_RS, "sdk_executor.rs", SDK_CRATE), "unpatched"
        )

    def test_patched_shape_is_patched(self):
        patched = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs", SDK_CRATE)
        self.assertEqual(
            ph.executor_state(patched, "sdk_executor.rs", SDK_CRATE), "patched"
        )

    def test_unrecognized_shape_refuses(self):
        source = EXECUTOR_RS.replace("HttpConfig::build_client failed", "build failed")
        with self.assertRaises(SystemExit) as cm:
            ph.executor_state(source, "sdk_executor.rs", SDK_CRATE)
        self.assertIn("refusing to guess", str(cm.exception))

    def test_two_anchors_refuse(self):
        source = EXECUTOR_RS + EXECUTOR_RS
        with self.assertRaises(SystemExit) as cm:
            ph.executor_state(source, "sdk_executor.rs", SDK_CRATE)
        self.assertIn("refusing to guess", str(cm.exception))

    def test_patched_plus_stray_anchor_refuses(self):
        source = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs", SDK_CRATE) + EXECUTOR_RS
        with self.assertRaises(SystemExit) as cm:
            ph.executor_state(source, "sdk_executor.rs", SDK_CRATE)
        self.assertIn("refusing to guess", str(cm.exception))

    def test_a_body_patched_against_another_crate_refuses(self):
        # The ENG-10291 state: the merge is present but names the pre-rename
        # crate, so it no longer compiles. Reads as neither shape by design.
        stale = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs", "hedra_sdk")
        with self.assertRaises(SystemExit) as cm:
            ph.executor_state(stale, "sdk_executor.rs", SDK_CRATE)
        self.assertIn("refusing to guess", str(cm.exception))


class PatchExecutor(unittest.TestCase):
    def test_inserts_the_merge(self):
        out = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs", SDK_CRATE)
        self.assertIn(f"{SDK_CRATE}::ClientConfig::default()", out)
        self.assertIn("global_headers.extend(identity);", out)

    def test_keeps_the_surrounding_constructor(self):
        out = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs", SDK_CRATE)
        self.assertIn('.expect("HttpConfig::build_client failed");', out)
        self.assertIn("retries: RetriesConfig::default(),", out)

    def test_touches_nothing_outside_the_anchor(self):
        out = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs", SDK_CRATE)
        self.assertEqual(
            out.replace(ph.executor_patch(SDK_CRATE), ph.EXECUTOR_ANCHOR), EXECUTOR_RS
        )

    def test_is_idempotent(self):
        once = ph.patch_executor(EXECUTOR_RS, "sdk_executor.rs", SDK_CRATE)
        twice = ph.patch_executor(once, "sdk_executor.rs", SDK_CRATE)
        self.assertEqual(once, twice)

    def test_names_the_removal_ticket(self):
        # The merge is a local fork of upstream behaviour; the marker is what a
        # future reader greps for when ENG-10234 lands and the fork comes out.
        self.assertIn("ENG-10234", ph.executor_patch(SDK_CRATE))


class HttpState(unittest.TestCase):
    def test_generated_shape_is_unpatched(self):
        self.assertEqual(ph.http_state(HTTP_RS, "http.rs", SDK_CRATE), "unpatched")

    def test_patched_shape_is_patched(self):
        patched = ph.patch_http(HTTP_RS, "http.rs", SDK_CRATE)
        self.assertEqual(ph.http_state(patched, "http.rs", SDK_CRATE), "patched")

    def test_unrecognized_shape_refuses(self):
        # The generator moving the UA seeding is the drift this must catch.
        source = HTTP_RS.replace("let mut headers = HeaderMap::new();", "let mut headers = HeaderMap::with_capacity(4);")
        with self.assertRaises(SystemExit) as cm:
            ph.http_state(source, "http.rs", SDK_CRATE)
        self.assertIn("refusing to guess", str(cm.exception))

    def test_two_anchors_refuse(self):
        with self.assertRaises(SystemExit) as cm:
            ph.http_state(HTTP_RS + HTTP_RS, "http.rs", SDK_CRATE)
        self.assertIn("refusing to guess", str(cm.exception))

    def test_a_body_patched_against_another_crate_refuses(self):
        stale = ph.patch_http(HTTP_RS, "http.rs", "hedra_sdk")
        with self.assertRaises(SystemExit) as cm:
            ph.http_state(stale, "http.rs", SDK_CRATE)
        self.assertIn("refusing to guess", str(cm.exception))


class PatchHttp(unittest.TestCase):
    def test_seeds_the_identity_headers(self):
        out = ph.patch_http(HTTP_RS, "http.rs", SDK_CRATE)
        self.assertIn(f"{SDK_CRATE}::ClientConfig::default().custom_headers", out)
        self.assertIn("builder = builder.default_headers(headers);", out)

    def test_keeps_seeding_the_user_agent(self):
        # The trio must be added to the UA, not instead of it: dropping the UA
        # would break the four openapi_path_*_user_agent locks next door.
        out = ph.patch_http(HTTP_RS, "http.rs", SDK_CRATE)
        self.assertIn("headers.insert(USER_AGENT, header_value);", out)

    def test_installs_the_map_outside_the_user_agent_branch(self):
        # The generated shape only calls default_headers when the UA parses.
        # The identity trio must not inherit that condition.
        out = ph.patch_http(HTTP_RS, "http.rs", SDK_CRATE)
        install = out.index("builder = builder.default_headers(headers);")
        branch_end = out.index("        }\n", out.index("if let Ok(header_value)"))
        self.assertGreater(install, branch_end)

    def test_touches_nothing_outside_the_anchor(self):
        out = ph.patch_http(HTTP_RS, "http.rs", SDK_CRATE)
        self.assertEqual(out.replace(ph.http_patch(SDK_CRATE), ph.HTTP_ANCHOR), HTTP_RS)

    def test_is_idempotent(self):
        once = ph.patch_http(HTTP_RS, "http.rs", SDK_CRATE)
        twice = ph.patch_http(once, "http.rs", SDK_CRATE)
        self.assertEqual(once, twice)

    def test_names_the_removal_ticket(self):
        self.assertIn("ENG-10234", ph.http_patch(SDK_CRATE))


class Cli(Tree):
    # apply mode

    def test_apply_patches_every_file(self):
        code, out = run("1.0.0-dev", *self.generated)
        self.assertEqual(code, 0)
        self.assertIn('"0.1.0" -> "1.0.0-dev"', out)
        self.assertIn("seeded the identity headers", out)
        self.assertIn("merged the identity headers", out)
        self.assertEqual(
            ph.read_sdk_version(self.read(self.config), self.config), "1.0.0-dev"
        )
        self.assertEqual(
            ph.http_state(self.read(self.http), self.http, SDK_CRATE), "patched"
        )
        self.assertEqual(
            ph.executor_state(self.read(self.executor), self.executor, SDK_CRATE),
            "patched",
        )

    def test_apply_is_a_no_op_on_the_second_run(self):
        run("1.0.0-dev", *self.generated)
        snapshot = {p: self.read(p) for p in self.generated}
        code, out = run("1.0.0-dev", *self.generated)
        self.assertEqual(code, 0)
        self.assertIn("(unchanged)", out)
        for path, content in snapshot.items():
            self.assertEqual(self.read(path), content)

    def test_apply_rejects_a_bad_version_before_touching_anything(self):
        before = self.read(self.config)
        with self.assertRaises(SystemExit):
            run("not-a-version", *self.generated)
        self.assertEqual(self.read(self.config), before)

    def test_apply_reports_a_missing_file(self):
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0-dev", f"{SDK_DIR}/src/nope.rs", self.http, self.executor)
        self.assertIn("nope.rs", str(cm.exception))

    def test_swapped_paths_die_loudly(self):
        # The executor file has no X-Fern-SDK-Version literal, so handing the
        # files over in the wrong order must refuse before writing anything.
        before = self.read(self.executor)
        with self.assertRaises(SystemExit):
            run("1.0.0-dev", self.executor, self.http, self.config)
        self.assertEqual(self.read(self.executor), before)

    def test_a_config_outside_a_crate_directory_refuses(self):
        # sdk_crate() derives the crate from <crate-dir>/src/config.rs. A flat
        # path has no crate to name, and guessing would emit a patch calling a
        # crate that does not exist.
        self.write("config.rs", CONFIG_RS)
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0-dev", "config.rs", self.http, self.executor)
        self.assertIn("cannot infer the SDK crate name", str(cm.exception))

    def test_a_manifest_disagreeing_with_the_directory_refuses(self):
        self.write(f"{SDK_DIR}/Cargo.toml", SDK_CARGO_TOML.replace(SDK_CRATE, "other"))
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0-dev", *self.generated)
        self.assertIn("does not match the directory-derived", str(cm.exception))

    # sequencing: Patch 1 (version) leads; Patch 2 (http) precedes Patch 3

    def test_version_is_stamped_even_when_a_later_anchor_is_gone(self):
        # The safe failure direction: a stamped version with no delivery is
        # inert; a delivery with a stale version puts 0.1.0 on the wire.
        broken = EXECUTOR_RS.replace(
            '.expect("HttpConfig::build_client failed");', ".unwrap();"
        )
        self.write(self.executor, broken)
        with self.assertRaises(SystemExit):
            run("1.0.0-dev", *self.generated)
        self.assertEqual(
            ph.read_sdk_version(self.read(self.config), self.config), "1.0.0-dev"
        )
        self.assertEqual(self.read(self.executor), broken)

    def test_a_drifted_executor_still_leaves_production_traffic_identified(self):
        # ENG-10310's ordering rule. src/http.rs is the path every built-in
        # command takes; src/sdk_executor.rs only serves custom commands. If the
        # executor anchor drifts, the run must already have repaired http.rs —
        # the reverse order fails with only the low-traffic path fixed, which is
        # the state the ticket was filed about.
        self.write(
            self.executor,
            EXECUTOR_RS.replace(
                '.expect("HttpConfig::build_client failed");', ".unwrap();"
            ),
        )
        with self.assertRaises(SystemExit):
            run("1.0.0-dev", *self.generated)
        self.assertEqual(
            ph.http_state(self.read(self.http), self.http, SDK_CRATE), "patched"
        )

    # --check

    def test_check_fails_on_the_generated_tree(self):
        code, out = run("--check", "1.0.0-dev", *self.generated)
        self.assertEqual(code, 1)
        self.assertIn('expected "1.0.0-dev"', out)
        self.assertIn("identity headers missing", out)
        self.assertIn("identity-header merge missing", out)
        self.assertIn("3 file(s) missing the SDK header patches", out)

    def test_check_passes_after_apply(self):
        run("1.0.0-dev", *self.generated)
        code, out = run("--check", "1.0.0-dev", *self.generated)
        self.assertEqual(code, 0)
        self.assertIn('X-Fern-SDK-Version = "1.0.0-dev"', out)
        self.assertIn("identity headers seeded into build_client", out)
        self.assertIn("identity-header merge present", out)

    def test_check_never_writes(self):
        before = {p: self.read(p) for p in self.generated}
        run("--check", "1.0.0-dev", *self.generated)
        for path, content in before.items():
            self.assertEqual(self.read(path), content)

    def test_check_catches_a_stale_version_behind_patched_delivery(self):
        # The state a delivery-without-Patch-1 would produce: headers live,
        # wrong literal. One drifted file, exit 1.
        run("1.0.0-dev", *self.generated)
        self.write(self.config, CONFIG_RS)
        code, out = run("--check", "1.0.0-dev", *self.generated)
        self.assertEqual(code, 1)
        self.assertIn("1 file(s) missing the SDK header patches", out)

    def test_check_catches_an_unpatched_http_alone(self):
        # The ENG-10310 tree exactly: executor merged, config stamped, and the
        # path all released traffic uses still sending no X-Fern-*.
        run("1.0.0-dev", *self.generated)
        self.write(self.http, HTTP_RS)
        code, out = run("--check", "1.0.0-dev", *self.generated)
        self.assertEqual(code, 1)
        self.assertIn("identity headers missing from HttpConfig::build_client", out)
        self.assertIn("1 file(s) missing the SDK header patches", out)

    def test_check_validates_the_version_argument(self):
        with self.assertRaises(SystemExit) as cm:
            run("--check", "1.0", *self.generated)
        self.assertIn("not a Cargo-style SemVer version", str(cm.exception))

    # usage

    def test_usage_with_no_arguments(self):
        with self.assertRaises(SystemExit) as cm:
            run()
        self.assertIn("usage:", str(cm.exception))

    def test_usage_with_too_few_arguments(self):
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0-dev", self.config, self.http)
        self.assertIn("usage:", str(cm.exception))

    def test_usage_with_too_many_arguments(self):
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0-dev", *self.generated, self.executor)
        self.assertIn("usage:", str(cm.exception))


class RegenerationScenario(Tree):
    """The sequence refresh-lockfile.yml runs after every regeneration."""

    def test_regenerate_apply_check_reapply(self):
        for cycle in range(2):
            with self.subTest(cycle=cycle):
                # Regeneration resets every file to the generated shape.
                self.regenerate()
                code, _ = run("--check", "1.0.0-dev", *self.generated)
                self.assertEqual(code, 1)

                # The workflow re-applies all three patches in one step.
                code, _ = run("1.0.0-dev", *self.generated)
                self.assertEqual(code, 0)
                self.assertEqual(run("--check", "1.0.0-dev", *self.generated)[0], 0)

                # A re-run leaves the tree byte-identical, so the worktree
                # stays clean and refresh-lockfile.yml's push loop terminates.
                snapshot = {p: self.read(p) for p in self.generated}
                run("1.0.0-dev", *self.generated)
                for path, content in snapshot.items():
                    self.assertEqual(self.read(path), content)


if __name__ == "__main__":
    unittest.main()
