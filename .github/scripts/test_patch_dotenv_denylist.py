#!/usr/bin/env python3
"""Tests for patch-dotenv-denylist.py.

    python3 -m unittest discover -s .github/scripts

Stdlib only, and no network or cargo: the script is pure text manipulation.
The script runs on every fern-bot PR, so the cases that matter most are the
ones where it must NOT write — an already-patched file — and the ones where it
must refuse loudly rather than guess: a regenerated shape the anchors no longer
match. The Rust the script injects is checked by `cargo test --locked`
(tests/dotenv_filter.rs), not here.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import os
import tempfile
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).resolve().parent / "patch-dotenv-denylist.py"
_spec = importlib.util.spec_from_file_location("patch_dotenv_denylist", _SCRIPT)
pd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pd)

# The deny-list region of src/lib.rs as the generator emits it, with the text
# around each anchor that must survive patching.
GENERATED = (
    "const DOTENV_DENIED_BARE: &[&str] = &[\n"
    '    "PAGER",\n'
    + pd.BARE_ANCHOR
    + '    "REQUESTS_CA_BUNDLE",\n'
    "];\n"
    "\n"
    "/// True when `key` is one a `.env` file must not be able to set.\n"
    + pd.MATCH_ANCHOR
    + "\n"
    "pub fn load_dotenv_filtered(cli_name: &str) -> Vec<String> {}\n"
)


def _run(argv: list[str]) -> tuple[int, str]:
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = pd.main(["patch-dotenv-denylist.py", *argv])
    return code, out.getvalue()


class PatchTests(unittest.TestCase):
    def setUp(self) -> None:
        self._dir = tempfile.TemporaryDirectory()
        self.path = os.path.join(self._dir.name, "lib.rs")

    def tearDown(self) -> None:
        self._dir.cleanup()

    def _write(self, text: str) -> None:
        Path(self.path).write_text(text, encoding="utf-8")

    def _read(self) -> str:
        return Path(self.path).read_text(encoding="utf-8")

    def test_patches_the_generated_shape(self) -> None:
        self._write(GENERATED)
        code, _ = _run([self.path])
        self.assertEqual(code, 0)
        patched = self._read()
        self.assertIn(pd.BARE_PATCH, patched)
        self.assertIn(pd.MATCH_PATCH, patched)
        self.assertNotIn(pd.MATCH_ANCHOR, patched)
        self.assertIn('    "REQUESTS_CA_BUNDLE",\n', patched)
        self.assertIn("pub fn load_dotenv_filtered", patched)

    def test_a_second_run_writes_nothing(self) -> None:
        self._write(GENERATED)
        _run([self.path])
        first = self._read()
        mtime = os.stat(self.path).st_mtime_ns
        code, out = _run([self.path])
        self.assertEqual(code, 0)
        self.assertEqual(self._read(), first)
        self.assertEqual(os.stat(self.path).st_mtime_ns, mtime)
        self.assertIn("already patched", out)

    def test_check_fails_on_the_generated_shape_and_writes_nothing(self) -> None:
        self._write(GENERATED)
        code, out = _run(["--check", self.path])
        self.assertEqual(code, 1)
        self.assertIn("2 .env deny-list patch(es) missing", out)
        self.assertEqual(self._read(), GENERATED)

    def test_check_passes_on_the_patched_shape(self) -> None:
        self._write(GENERATED)
        _run([self.path])
        code, _ = _run(["--check", self.path])
        self.assertEqual(code, 0)

    def test_check_names_a_half_patched_file(self) -> None:
        self._write(GENERATED.replace(pd.BARE_ANCHOR, pd.BARE_PATCH))
        code, out = _run(["--check", self.path])
        self.assertEqual(code, 1)
        self.assertIn("missing: dotenv_key_is_denied", out)

    def test_a_drifted_anchor_refuses_and_writes_nothing(self) -> None:
        # The second anchor drifted: the first must not be applied on its own.
        drifted = GENERATED.replace("key.strip_prefix(prefix)", "key.strip_prefix(&prefix)")
        self._write(drifted)
        with self.assertRaises(SystemExit) as raised:
            _run([self.path])
        self.assertIn("dotenv_key_is_denied matches neither", str(raised.exception))
        self.assertEqual(self._read(), drifted)

    def test_a_duplicated_anchor_refuses(self) -> None:
        self._write(GENERATED + pd.BARE_ANCHOR)
        with self.assertRaises(SystemExit) as raised:
            _run([self.path])
        self.assertIn("anchor x2", str(raised.exception))

    def test_the_committed_lib_rs_is_patched(self) -> None:
        lib = Path(__file__).resolve().parents[2] / "src" / "lib.rs"
        code, _ = _run(["--check", str(lib)])
        self.assertEqual(code, 0)

    def test_usage_without_a_path(self) -> None:
        with self.assertRaises(SystemExit):
            _run(["--check"])


if __name__ == "__main__":
    unittest.main()
