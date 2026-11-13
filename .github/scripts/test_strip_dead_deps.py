#!/usr/bin/env python3
"""Tests for strip-dead-deps.py.

    python3 -m unittest discover -s .github/scripts

Stdlib only, and no network or cargo. The script runs on every fern-bot branch,
so the cases that matter most are the ones where it must NOT write: a crate
with a build script, a dependency some Rust source still uses, and a manifest
that is already clean. The positive case is the generator's real output, which
must come out byte-identical to the hand-fixed manifest #186 put on main.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import os
import tempfile
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).resolve().parent / "strip-dead-deps.py"
_spec = importlib.util.spec_from_file_location("strip_dead_deps", _SCRIPT)
sd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(sd)


# The shape of the generator's root manifest: [dependencies] with hmac among
# the others, then the template's [build-dependencies] between [profile.*] and
# [dev-dependencies], and a file that ends in a blank line.
GENERATED = '''\
[package]
name = "hedra-cli"
version = "8.0.1"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
hmac = "0.12"
dotenvy = "0.15"
serde_yaml = "0.9.34"

[profile.dist]
inherits = "release"
lto = "thin"

[build-dependencies]
serde = "1"
serde_yaml = "0.9.34"

[dev-dependencies]
tempfile = "3"

'''

# GENERATED after #186's hand fix: the same file minus the three lines and the
# table header, every separator and the trailing blank line intact.
STRIPPED = '''\
[package]
name = "hedra-cli"
version = "8.0.1"
edition = "2021"

[dependencies]
clap = { version = "4", features = ["derive"] }
dotenvy = "0.15"
serde_yaml = "0.9.34"

[profile.dist]
inherits = "release"
lto = "thin"

[dev-dependencies]
tempfile = "3"

'''


class StripTest(unittest.TestCase):
    def setUp(self):
        self._dir = tempfile.TemporaryDirectory()
        self.root = self._dir.name
        self.manifest = os.path.join(self.root, "Cargo.toml")
        os.makedirs(os.path.join(self.root, "src"))
        self.write("src/main.rs", "fn main() {}\n")

    def tearDown(self):
        self._dir.cleanup()

    def write(self, rel, text):
        path = os.path.join(self.root, rel)
        os.makedirs(os.path.dirname(path), exist_ok=True)
        with open(path, "w", encoding="utf-8", newline="") as fh:
            fh.write(text)

    def read(self):
        with open(self.manifest, encoding="utf-8", newline="") as fh:
            return fh.read()

    def strip(self, text):
        self.write("Cargo.toml", text)
        with contextlib.redirect_stdout(io.StringIO()):
            return sd.strip(self.manifest)

    def test_generated_manifest_matches_the_hand_fix(self):
        self.assertEqual(self.strip(GENERATED), ["[build-dependencies]", "hmac"])
        self.assertEqual(self.read(), STRIPPED)

    def test_second_run_is_a_no_op(self):
        self.strip(GENERATED)
        self.assertEqual(self.strip(self.read()), [])
        self.assertEqual(self.read(), STRIPPED)

    def test_clean_manifest_is_not_rewritten(self):
        self.write("Cargo.toml", STRIPPED)
        before = os.stat(self.manifest).st_mtime_ns
        os.utime(self.manifest, ns=(before - 10**9, before - 10**9))
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(sd.strip(self.manifest), [])
        self.assertEqual(os.stat(self.manifest).st_mtime_ns, before - 10**9)

    def test_build_rs_keeps_build_dependencies(self):
        self.write("build.rs", "fn main() {}\n")
        self.assertEqual(self.strip(GENERATED), ["hmac"])
        self.assertIn("[build-dependencies]\nserde = \"1\"\n", self.read())

    def test_build_key_keeps_build_dependencies(self):
        text = GENERATED.replace('edition = "2021"\n', 'edition = "2021"\nbuild = "gen/build.rs"\n')
        self.assertEqual(self.strip(text), ["hmac"])
        self.assertIn("[build-dependencies]", self.read())

    def test_build_key_outside_package_does_not_count(self):
        # A `build` key in some other table is not a build script.
        text = GENERATED.replace('lto = "thin"\n', 'lto = "thin"\nbuild = "x"\n')
        self.assertIn("[build-dependencies]", self.strip(text))

    def test_used_hmac_is_kept(self):
        self.write("src/auth/sign.rs", "use hmac::{Hmac, Mac};\n")
        self.assertEqual(self.strip(GENERATED), ["[build-dependencies]"])
        self.assertIn('hmac = "0.12"\n', self.read())

    def test_hmac_as_part_of_another_word_does_not_count(self):
        self.write("src/lib.rs", "// hmacsha256 and sha_hmac are not the crate\n")
        self.assertIn("hmac", self.strip(GENERATED))

    def test_dot_directories_and_target_are_not_scanned(self):
        # .agents/worktrees holds other checkouts; target/ holds build output.
        self.write(".agents/worktrees/x/src/lib.rs", "use hmac::Mac;\n")
        self.write("target/debug/build/out.rs", "use hmac::Mac;\n")
        self.assertIn("hmac", self.strip(GENERATED))

    def test_hmac_elsewhere_than_dependencies_is_untouched(self):
        text = STRIPPED.replace('tempfile = "3"\n', 'tempfile = "3"\nhmac = "0.12"\n')
        self.assertEqual(self.strip(text), [])
        self.assertIn('hmac = "0.12"\n', self.read())

    def test_inline_table_spec_is_removed_whole(self):
        text = GENERATED.replace('hmac = "0.12"', 'hmac = { version = "0.12", default-features = false }')
        self.strip(text)
        self.assertEqual(self.read(), STRIPPED)

    def test_build_dependencies_at_eof(self):
        text = STRIPPED + "[build-dependencies]\nserde = \"1\"\n"
        self.assertEqual(self.strip(text), ["[build-dependencies]"])
        self.assertEqual(self.read(), STRIPPED.rstrip("\n") + "\n")

    def test_crlf_manifest_is_left_untouched(self):
        # The generator writes LF, and like set-version.py the table patterns
        # key on `\n`. A CRLF manifest is never half-edited: it stays
        # byte-identical, and cargo's own lint still flags it in CI.
        text = GENERATED.replace("\n", "\r\n")
        self.assertEqual(self.strip(text), [])
        self.assertEqual(self.read(), text)

    def test_missing_manifest_exits_with_the_path(self):
        with self.assertRaises(SystemExit) as caught:
            sd.strip(os.path.join(self.root, "nope", "Cargo.toml"))
        self.assertIn("nope", str(caught.exception))


class MainTest(unittest.TestCase):
    def test_usage(self):
        with contextlib.redirect_stderr(io.StringIO()) as err:
            self.assertEqual(sd.main(["strip-dead-deps.py"]), 2)
        self.assertIn("usage:", err.getvalue())

    def test_flags_are_rejected(self):
        with contextlib.redirect_stderr(io.StringIO()):
            self.assertEqual(sd.main(["strip-dead-deps.py", "--check", "Cargo.toml"]), 2)


if __name__ == "__main__":
    unittest.main()
