#!/usr/bin/env python3
"""Tests for patch-dotenv-filter.py.

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

_SCRIPT = Path(__file__).resolve().parent / "patch-dotenv-filter.py"
_spec = importlib.util.spec_from_file_location("patch_dotenv_filter", _SCRIPT)
pd = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(pd)

# The .env filter region of src/lib.rs as the generator emits it, with the text
# around each anchor that must survive patching.
GENERATED = (
    "const DOTENV_DENIED_SUFFIXES: &[&str] = &[\n"
    '    "_BASE_URL",\n'
    + pd.SUFFIX_ANCHOR
    + "\n"
    "/// True when `key` is one a `.env` file must not be able to set.\n"
    + pd.MATCH_ANCHOR
    + "\n"
    + pd.DOC_ANCHOR
    + "pub fn load_dotenv_filtered(cli_name: &str) -> Vec<String> {\n"
    "    for entry in entries.flatten() {\n"
    + pd.LOAD_ANCHOR
    + "            ignored.push(key);\n"
    "        }\n"
    "    }\n"
    "}\n"
    "\n"
    "pub fn warn_ignored_dotenv_keys(ignored: &[String]) {\n"
    "        tracing::warn!(\n"
    + pd.WARN_ANCHOR
    + "        );\n"
    "}\n"
)

# The `.env` region of src/lib.rs exactly as the generator emits it (copied
# from an unpatched regeneration), from _REGION_START up to _REGION_END: the
# deny-lists, `dotenv_key_is_denied`, `load_dotenv_filtered` and
# `warn_ignored_dotenv_keys`.
_REGION_START = "const DOTENV_DENIED_BARE: &[&str] = &[\n"
_REGION_END = "#[cfg(test)]\nmod dotenv_filter_tests"
GENERATED_VERBATIM = (
    "const DOTENV_DENIED_BARE: &[&str] = &[\n"
    "    // Names a program the CLI executes when paging output.\n"
    '    "PAGER",\n'
    "    // Route traffic through an attacker-chosen intermediary.\n"
    '    "HTTP_PROXY",\n'
    '    "HTTPS_PROXY",\n'
    '    "ALL_PROXY",\n'
    '    "NO_PROXY",\n'
    "    // Replace the trust store, enabling transparent interception.\n"
    '    "SSL_CERT_FILE",\n'
    '    "CURL_CA_BUNDLE",\n'
    '    "REQUESTS_CA_BUNDLE",\n'
    "];\n"
    "\n"
    "/// Suffixes of `<PREFIX>_…` env vars a `.env` file may not set. Same\n"
    "/// reasoning as [`DOTENV_DENIED_BARE`], plus the two cross-host guards — a\n"
    "/// `.env` that could disable them would undo the redirect and pagination\n"
    "/// protections.\n"
    "const DOTENV_DENIED_SUFFIXES: &[&str] = &[\n"
    '    "_BASE_URL",\n'
    '    "_PAGER",\n'
    '    "_PROXY",\n'
    '    "_NO_PROXY",\n'
    '    "_INSECURE",\n'
    '    "_INSECURE_SKIP_VERIFY",\n'
    '    "_CA_BUNDLE",\n'
    '    "_EXTRA_CA_CERTS",\n'
    '    "_ALLOW_CROSS_HOST_REDIRECTS",\n'
    '    "_ALLOW_CROSS_HOST_PAGINATION",\n'
    "];\n"
    "\n"
    "/// True when `key` is one a `.env` file must not be able to set.\n"
    "pub(crate) fn dotenv_key_is_denied(key: &str, prefix: &str) -> bool {\n"
    "    if DOTENV_DENIED_BARE.contains(&key) {\n"
    "        return true;\n"
    "    }\n"
    "    key.strip_prefix(prefix)\n"
    "        .is_some_and(|rest| DOTENV_DENIED_SUFFIXES.contains(&rest))\n"
    "}\n"
    "\n"
    "/// Load `.env`, ignoring keys that control transport or execution.\n"
    "///\n"
    "/// A generated CLI is routinely run inside repositories the operator did not\n"
    "/// write, and `dotenvy::dotenv()` searches the working directory and its\n"
    "/// ancestors — so an attacker-authored `.env` would otherwise be able to point\n"
    "/// the CLI at another host, disable certificate verification, route requests\n"
    "/// through a proxy, turn off the cross-host redirect and pagination guards, or\n"
    "/// name an arbitrary program for the CLI to execute as its pager. The last is\n"
    "/// the sharpest: `<PREFIX>_PAGER` is run as a command.\n"
    "///\n"
    "/// The legitimate use of `.env` — credentials and output preferences — is\n"
    "/// unaffected; only the keys in [`DOTENV_DENIED_BARE`] and\n"
    "/// [`DOTENV_DENIED_SUFFIXES`] are dropped. Real process environment always\n"
    "/// wins, matching `dotenvy::dotenv()`'s own precedence, so anything genuinely\n"
    "/// exported by the operator's shell still applies.\n"
    "/// Returns the keys that were ignored, in file order, so the caller can report\n"
    "/// them *after* logging is initialized. Warning from inside this function would\n"
    "/// be silently dropped: it runs before `init_logging`, so no subscriber exists\n"
    "/// yet and a legitimate `<PREFIX>_BASE_URL` in `.env` would appear to be\n"
    "/// ignored for no reason.\n"
    "pub fn load_dotenv_filtered(cli_name: &str) -> Vec<String> {\n"
    "    let prefix = cli_name.to_uppercase().replace('-', \"_\");\n"
    "    let mut ignored = Vec::new();\n"
    "    let Ok(entries) = dotenvy::dotenv_iter() else {\n"
    "        // No `.env` (the common case) or it is unreadable — nothing to do.\n"
    "        return ignored;\n"
    "    };\n"
    "    for entry in entries.flatten() {\n"
    "        let (key, value) = entry;\n"
    "        if dotenv_key_is_denied(&key, &prefix) {\n"
    "            ignored.push(key);\n"
    "            continue;\n"
    "        }\n"
    "        // Real environment wins, as with `dotenvy::dotenv()`.\n"
    "        if std::env::var_os(&key).is_none() {\n"
    "            std::env::set_var(&key, &value);\n"
    "        }\n"
    "    }\n"
    "    ignored\n"
    "}\n"
    "\n"
    "/// Report the keys [`load_dotenv_filtered`] dropped. Call after\n"
    "/// `init_logging`, or the messages go nowhere.\n"
    "pub fn warn_ignored_dotenv_keys(ignored: &[String]) {\n"
    "    for key in ignored {\n"
    "        tracing::warn!(\n"
    "            key = %key,\n"
    '            "ignoring {key} from .env: it controls transport or process execution, and a \\\n'
    '             .env file is not a trusted source for it. Export it from your shell instead."\n'
    "        );\n"
    "    }\n"
    "}\n"
    "\n"
)

def _run(argv: list[str]) -> tuple[int, str]:
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = pd.main(["patch-dotenv-filter.py", *argv])
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
        for _, anchor, patched_body in pd.PATCHES:
            self.assertIn(patched_body, patched)
            self.assertNotIn(anchor, patched)
        self.assertIn('    "_BASE_URL",\n', patched)
        self.assertIn("            ignored.push(key);\n", patched)

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
        self.assertIn("5 .env filter patch(es) missing", out)
        self.assertEqual(self._read(), GENERATED)

    def test_check_passes_on_the_patched_shape(self) -> None:
        self._write(GENERATED)
        _run([self.path])
        code, _ = _run(["--check", self.path])
        self.assertEqual(code, 0)

    def test_check_names_a_half_patched_file(self) -> None:
        self._write(GENERATED.replace(pd.SUFFIX_ANCHOR, pd.SUFFIX_PATCH))
        code, out = _run(["--check", self.path])
        self.assertEqual(code, 1)
        self.assertIn("missing: dotenv_key_is_denied, load_dotenv_filtered doc, load_dotenv_filtered, "
            "warn_ignored_dotenv_keys", out)

    def test_a_drifted_anchor_refuses_and_writes_nothing(self) -> None:
        # The second anchor drifted: the first must not be applied on its own.
        drifted = GENERATED.replace("key.strip_prefix(prefix)", "key.strip_prefix(&prefix)")
        self._write(drifted)
        with self.assertRaises(SystemExit) as raised:
            _run([self.path])
        self.assertIn("dotenv_key_is_denied matches neither", str(raised.exception))
        self.assertEqual(self._read(), drifted)

    def test_a_duplicated_anchor_refuses(self) -> None:
        self._write(GENERATED + pd.LOAD_ANCHOR)
        with self.assertRaises(SystemExit) as raised:
            _run([self.path])
        self.assertIn("load_dotenv_filtered matches neither", str(raised.exception))
        self.assertIn("anchor x2", str(raised.exception))

    def test_a_duplicated_doc_anchor_refuses(self) -> None:
        self._write(GENERATED + pd.DOC_ANCHOR)
        with self.assertRaises(SystemExit) as raised:
            _run(["--check", self.path])
        self.assertIn("load_dotenv_filtered doc matches neither", str(raised.exception))
        self.assertIn("anchor x2", str(raised.exception))

    def test_a_drifted_doc_anchor_refuses_and_writes_nothing(self) -> None:
        drifted = GENERATED.replace("/// unaffected; only the keys", "/// unaffected: only the keys")
        self._write(drifted)
        with self.assertRaises(SystemExit) as raised:
            _run([self.path])
        self.assertIn("load_dotenv_filtered doc matches neither", str(raised.exception))
        self.assertEqual(self._read(), drifted)

    def test_a_duplicated_suffix_anchor_refuses(self) -> None:
        self._write(GENERATED + pd.SUFFIX_ANCHOR)
        with self.assertRaises(SystemExit) as raised:
            _run(["--check", self.path])
        self.assertIn("DOTENV_DENIED_SUFFIXES matches neither", str(raised.exception))
        self.assertIn("anchor x2", str(raised.exception))

    def test_a_drifted_suffix_anchor_refuses_and_writes_nothing(self) -> None:
        # The anchor is the list's last entry: one appended after it by the
        # generator moves the closing bracket off the anchor.
        drifted = GENERATED.replace(
            pd.SUFFIX_ANCHOR,
            pd.SUFFIX_ANCHOR.replace("];\n", '    "_NEW",\n];\n'),
        )
        self._write(drifted)
        with self.assertRaises(SystemExit) as raised:
            _run([self.path])
        self.assertIn("DOTENV_DENIED_SUFFIXES matches neither", str(raised.exception))
        self.assertEqual(self._read(), drifted)

    def test_the_committed_lib_rs_is_patched(self) -> None:
        lib = Path(__file__).resolve().parents[2] / "src" / "lib.rs"
        code, _ = _run(["--check", str(lib)])
        self.assertEqual(code, 0)

    def test_usage_without_a_path(self) -> None:
        with self.assertRaises(SystemExit):
            _run(["--check"])

    def test_patches_a_half_patched_file(self) -> None:
        # Apply mode, not just --check: the site still unpatched must be written.
        self._write(GENERATED.replace(pd.SUFFIX_ANCHOR, pd.SUFFIX_PATCH))
        code, _ = _run([self.path])
        self.assertEqual(code, 0)
        patched = self._read()
        for _, anchor, patched_body in pd.PATCHES:
            self.assertIn(patched_body, patched)
            self.assertNotIn(anchor, patched)

    def test_a_duplicated_patch_refuses(self) -> None:
        self._write(GENERATED.replace(pd.SUFFIX_ANCHOR, pd.SUFFIX_PATCH) + pd.SUFFIX_PATCH)
        with self.assertRaises(SystemExit) as raised:
            _run(["--check", self.path])
        self.assertIn("anchor x0, patch x2", str(raised.exception))

    def test_a_stray_anchor_beside_the_patch_refuses(self) -> None:
        self._write(GENERATED.replace(pd.SUFFIX_ANCHOR, pd.SUFFIX_PATCH) + pd.SUFFIX_ANCHOR)
        with self.assertRaises(SystemExit) as raised:
            _run(["--check", self.path])
        self.assertIn("anchor x1, patch x1", str(raised.exception))

    def test_the_verbatim_generated_region_patches_to_the_committed_one(self) -> None:
        # GENERATED is built from the script's own anchors, so it cannot notice
        # an anchor that drifted from the generator's text or a patch that
        # leaves stray lines. This region is copied verbatim from generator
        # output; patching it must give exactly what src/lib.rs carries.
        self._write(GENERATED_VERBATIM)
        code, _ = _run([self.path])
        self.assertEqual(code, 0)
        lib = (Path(__file__).resolve().parents[2] / "src" / "lib.rs").read_text(encoding="utf-8")
        start = lib.index(_REGION_START)
        end = lib.index(_REGION_END, start)
        self.assertEqual(self._read(), lib[start:end])


if __name__ == "__main__":
    unittest.main()
