#!/usr/bin/env python3
"""Tests for set-version.py.

    python3 -m unittest discover -s .github/scripts

Stdlib only, and no network or cargo: the script is pure text manipulation, and
these run anywhere python3 does. The script went from once-per-release to
once-per-regeneration in ENG-10219, so its blast radius is every fern-bot PR —
the cases that matter most here are the ones where it must NOT write: an inline
dependency `version`, a `version` in a non-[package] table, and a manifest that
is already at the target version.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import os
import tempfile
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).resolve().parent / "set-version.py"
_spec = importlib.util.spec_from_file_location("set_version", _SCRIPT)
sv = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(sv)


ROOT_MANIFEST = '''\
[package]
name = "hedra-cli"
version = "1.0.0-dev"
edition = "2021"
description = "Command-line interface for the Hedra Web API"
license = "Apache-2.0"

[lib]
name = "fern_cli_sdk"
path = "src/lib.rs"

[features]
# TLS backend selection. This comment carries the cargo-dist rationale and
# must survive a version stamp.
default = ["native-tls"]

[dependencies]
reqwest = { version = "0.12", features = ["json"], default-features = false }
serde = "1"

[dependencies.hedra_sdk]
path = "hedra-sdk"
'''

SDK_MANIFEST = '''\
[package]
name = "hedra_sdk"
version = "0.1.0"
edition = "2021"

[dependencies.hedra_types]
path = "../hedra-types"
'''


def run(*argv):
    """Invoke main() with argv, capturing (exit_code, stdout)."""
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = sv.main(["set-version.py", *argv])
    return code, out.getvalue()


class ReadVersion(unittest.TestCase):
    def test_reads_root_package_version(self):
        self.assertEqual(sv.read_version(ROOT_MANIFEST, "Cargo.toml"), "1.0.0-dev")

    def test_reads_plain_release_version(self):
        self.assertEqual(sv.read_version(SDK_MANIFEST, "hedra-sdk/Cargo.toml"), "0.1.0")

    def test_reads_zero_version(self):
        source = '[package]\nname = "hedra_types"\nversion = "0.0.0"\n'
        self.assertEqual(sv.read_version(source, "hedra-types/Cargo.toml"), "0.0.0")

    def test_ignores_inline_dependency_version(self):
        source = '[package]\nname = "x"\nversion = "9.9.9"\n\n[dependencies]\nreqwest = { version = "0.12" }\n'
        self.assertEqual(sv.read_version(source, "Cargo.toml"), "9.9.9")

    def test_tolerates_tabs_around_equals(self):
        source = '[package]\nname = "x"\nversion\t=\t"2.3.4"\n'
        self.assertEqual(sv.read_version(source, "Cargo.toml"), "2.3.4")

    def test_tolerates_trailing_whitespace(self):
        source = '[package]\nname = "x"\nversion = "2.3.4"   \n'
        self.assertEqual(sv.read_version(source, "Cargo.toml"), "2.3.4")

    def test_package_table_at_eof_without_trailing_newline(self):
        self.assertEqual(sv.read_version('[package]\nversion = "1.2.3"', "Cargo.toml"), "1.2.3")


class SetVersion(unittest.TestCase):
    def test_replaces_the_package_version(self):
        out = sv.set_version(SDK_MANIFEST, "1.0.0-dev", "hedra-sdk/Cargo.toml")
        self.assertIn('version = "1.0.0-dev"', out)
        self.assertNotIn('version = "0.1.0"', out)

    def test_leaves_every_other_byte_alone(self):
        out = sv.set_version(ROOT_MANIFEST, "2.0.0", "Cargo.toml")
        self.assertEqual(
            out.replace('version = "2.0.0"', 'version = "1.0.0-dev"'), ROOT_MANIFEST
        )

    def test_preserves_comments(self):
        out = sv.set_version(ROOT_MANIFEST, "2.0.0", "Cargo.toml")
        self.assertIn("# TLS backend selection.", out)
        self.assertIn("must survive a version stamp.", out)

    def test_does_not_touch_inline_dependency_version(self):
        out = sv.set_version(ROOT_MANIFEST, "2.0.0", "Cargo.toml")
        self.assertIn('reqwest = { version = "0.12", features = ["json"]', out)

    def test_does_not_touch_version_in_another_table(self):
        source = '[package]\nversion = "1.0.0"\n\n[dependencies.serde]\nversion = "1.0.200"\n'
        out = sv.set_version(source, "3.0.0", "Cargo.toml")
        self.assertIn('[package]\nversion = "3.0.0"', out)
        self.assertIn('[dependencies.serde]\nversion = "1.0.200"', out)

    def test_normalises_spacing_of_the_key_it_writes(self):
        out = sv.set_version('[package]\nversion\t=\t"1.0.0"\n', "2.0.0", "Cargo.toml")
        self.assertIn('version = "2.0.0"', out)

    def test_is_idempotent(self):
        once = sv.set_version(ROOT_MANIFEST, "2.0.0", "Cargo.toml")
        twice = sv.set_version(once, "2.0.0", "Cargo.toml")
        self.assertEqual(once, twice)

    def test_round_trips_with_read_version(self):
        out = sv.set_version(ROOT_MANIFEST, "4.5.6-rc.1", "Cargo.toml")
        self.assertEqual(sv.read_version(out, "Cargo.toml"), "4.5.6-rc.1")

    def test_package_table_at_eof(self):
        out = sv.set_version('[package]\nversion = "1.2.3"', "1.2.4", "Cargo.toml")
        self.assertEqual(out, '[package]\nversion = "1.2.4"')


class MalformedManifests(unittest.TestCase):
    def test_no_package_table(self):
        with self.assertRaises(SystemExit) as cm:
            sv.read_version('[dependencies]\nserde = "1"\n', "Cargo.toml")
        self.assertIn("no [package] table found", str(cm.exception))

    def test_two_package_tables(self):
        source = '[package]\nversion = "1.0.0"\n\n[package]\nversion = "2.0.0"\n'
        with self.assertRaises(SystemExit) as cm:
            sv.read_version(source, "Cargo.toml")
        self.assertIn("refusing to guess", str(cm.exception))

    def test_no_version_key(self):
        with self.assertRaises(SystemExit) as cm:
            sv.read_version('[package]\nname = "x"\n', "Cargo.toml")
        self.assertIn("found 0", str(cm.exception))

    def test_two_version_keys(self):
        with self.assertRaises(SystemExit) as cm:
            sv.read_version('[package]\nversion = "1.0.0"\nversion = "2.0.0"\n', "Cargo.toml")
        self.assertIn("found 2", str(cm.exception))

    def test_set_version_rejects_a_manifest_without_a_version_key(self):
        with self.assertRaises(SystemExit):
            sv.set_version('[package]\nname = "x"\n', "1.0.0", "Cargo.toml")


class SemverGrammar(unittest.TestCase):
    def test_accepts(self):
        for version in ["0.0.0", "1.0.0", "1.0.0-dev", "1.2.3-alpha.1", "1.0.0+build.5", "10.20.30"]:
            with self.subTest(version=version):
                self.assertTrue(sv.SEMVER.match(version))

    def test_rejects(self):
        for version in ["", "1.0", "1", "v1.0.0", "01.0.0", "1.0.0.0", "1.0.0-", "dev", "1.0.0 "]:
            with self.subTest(version=version):
                self.assertFalse(sv.SEMVER.match(version))


class Cli(unittest.TestCase):
    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)
        self.root = self.write("Cargo.toml", ROOT_MANIFEST)
        self.sdk = self.write("sdk.toml", SDK_MANIFEST)

    def write(self, name, content):
        path = os.path.join(self.dir.name, name)
        Path(path).write_text(content, encoding="utf-8")
        return path

    def read(self, path):
        return Path(path).read_text(encoding="utf-8")

    # --read

    def test_read_prints_the_bare_version(self):
        code, out = run("--read", self.root)
        self.assertEqual(code, 0)
        self.assertEqual(out, "1.0.0-dev\n")

    def test_read_does_not_write(self):
        before = self.read(self.root)
        run("--read", self.root)
        self.assertEqual(self.read(self.root), before)

    def test_read_requires_exactly_one_manifest(self):
        with self.assertRaises(SystemExit) as cm:
            run("--read", self.root, self.sdk)
        self.assertIn("--read <manifest>", str(cm.exception))

    def test_read_reports_a_missing_file(self):
        with self.assertRaises(SystemExit) as cm:
            run("--read", os.path.join(self.dir.name, "nope.toml"))
        self.assertIn("nope.toml", str(cm.exception))

    # --check

    def test_check_passes_when_aligned(self):
        code, out = run("--check", "1.0.0-dev", self.root)
        self.assertEqual(code, 0)
        self.assertIn('version = "1.0.0-dev"', out)

    def test_check_fails_when_drifted(self):
        code, out = run("--check", "1.0.0-dev", self.sdk)
        self.assertEqual(code, 1)
        self.assertIn('expected "1.0.0-dev"', out)
        self.assertIn("1 manifest(s) not at version", out)

    def test_check_never_writes(self):
        before = self.read(self.sdk)
        run("--check", "1.0.0-dev", self.sdk)
        self.assertEqual(self.read(self.sdk), before)

    def test_check_reports_every_drifted_manifest(self):
        code, out = run("--check", "2.0.0", self.root, self.sdk)
        self.assertEqual(code, 1)
        self.assertIn("2 manifest(s) not at version", out)

    def test_check_validates_the_version_argument(self):
        with self.assertRaises(SystemExit) as cm:
            run("--check", "1.0", self.root)
        self.assertIn("not a Cargo-style SemVer version", str(cm.exception))

    # write mode

    def test_write_updates_the_file(self):
        code, out = run("1.0.0-dev", self.sdk)
        self.assertEqual(code, 0)
        self.assertIn('"0.1.0" -> "1.0.0-dev"', out)
        self.assertEqual(sv.read_version(self.read(self.sdk), self.sdk), "1.0.0-dev")

    def test_write_accepts_several_manifests(self):
        code, _ = run("7.7.7", self.root, self.sdk)
        self.assertEqual(code, 0)
        self.assertEqual(sv.read_version(self.read(self.root), self.root), "7.7.7")
        self.assertEqual(sv.read_version(self.read(self.sdk), self.sdk), "7.7.7")

    def test_write_leaves_an_already_correct_manifest_byte_identical(self):
        before = self.read(self.root)
        code, out = run("1.0.0-dev", self.root)
        self.assertEqual(code, 0)
        self.assertIn("(unchanged)", out)
        self.assertEqual(self.read(self.root), before)

    def test_write_is_a_no_op_on_the_second_run(self):
        run("1.0.0-dev", self.sdk)
        after_first = self.read(self.sdk)
        run("1.0.0-dev", self.sdk)
        self.assertEqual(self.read(self.sdk), after_first)

    def test_write_rejects_a_bad_version_before_touching_anything(self):
        before = self.read(self.sdk)
        with self.assertRaises(SystemExit):
            run("not-a-version", self.sdk)
        self.assertEqual(self.read(self.sdk), before)

    def test_usage_with_no_arguments(self):
        with self.assertRaises(SystemExit) as cm:
            run()
        self.assertIn("usage:", str(cm.exception))

    def test_usage_with_a_version_but_no_manifest(self):
        with self.assertRaises(SystemExit) as cm:
            run("1.0.0")
        self.assertIn("usage:", str(cm.exception))


class AlignmentScenario(unittest.TestCase):
    """The exact sequence refresh-lockfile.yml runs, end to end."""

    def setUp(self):
        self.dir = tempfile.TemporaryDirectory()
        self.addCleanup(self.dir.cleanup)
        self.root = os.path.join(self.dir.name, "Cargo.toml")
        self.sdk = os.path.join(self.dir.name, "sdk.toml")
        self.types = os.path.join(self.dir.name, "types.toml")
        Path(self.root).write_text(ROOT_MANIFEST, encoding="utf-8")
        Path(self.sdk).write_text(SDK_MANIFEST, encoding="utf-8")
        Path(self.types).write_text(
            '[package]\nname = "hedra_types"\nversion = "0.0.0"\n', encoding="utf-8"
        )

    def align(self):
        code, out = run("--read", self.root)
        self.assertEqual(code, 0)
        version = out.strip()
        return version, run(version, self.sdk, self.types)

    def test_post_regeneration_alignment_then_no_op(self):
        # First pass: both members are reset by regeneration and get aligned.
        version, (code, _) = self.align()
        self.assertEqual(version, "1.0.0-dev")
        self.assertEqual(code, 0)
        self.assertEqual(sv.read_version(Path(self.sdk).read_text(), self.sdk), "1.0.0-dev")
        self.assertEqual(sv.read_version(Path(self.types).read_text(), self.types), "1.0.0-dev")

        # --check now agrees, which is the ci.yml gate.
        self.assertEqual(run("--check", version, self.root, self.sdk, self.types)[0], 0)

        # Second pass leaves the tree byte-identical, so the worktree stays
        # clean and refresh-lockfile.yml's push loop terminates.
        snapshot = {p: Path(p).read_text() for p in (self.root, self.sdk, self.types)}
        self.align()
        for path, content in snapshot.items():
            self.assertEqual(Path(path).read_text(), content)

    def test_root_version_is_never_rewritten_by_the_aligner(self):
        before = Path(self.root).read_text()
        self.align()
        self.assertEqual(Path(self.root).read_text(), before)


if __name__ == "__main__":
    unittest.main()
