#!/usr/bin/env python3
"""Re-apply the .env allow-list patches to freshly generated runtime code.

The generator emits `load_dotenv_filtered` and its deny-list in src/lib.rs, and
regeneration resets that file, so an ordinary in-repo fix is reverted by the
next fern-bot PR. src/lib.rs cannot be frozen in .fernignore — it is runtime
the generator still evolves — so, like patch-sdk-headers.py, the fix is
re-applied after every regeneration.

The rule the patches implement: a `.env` may set only HEDRA_ keys and
NO_COLOR, minus the transport, TLS, pager and log-file settings the deny-list
names, compared in any case. The generated deny-list alone cannot be made
complete, because libraries the CLI links read environment variables of
their own, so the patched loader admits a short allow-list and refuses
everything else.

  Patch 1  DOTENV_DENIED_SUFFIXES — add `_LOG_FILE`. `<PREFIX>_LOG_FILE`
           names the directory the CLI writes its logs to.

  Patch 2  dotenv_key_is_denied — compare the key and the prefix in any case,
           since Windows resolves a variable whatever case its name is set in;
           and add `dotenv_key_is_allowed`, the allow-list. The vendor prefix
           is the first `_`-separated segment of the CLI prefix, so the
           function stays generic. It is a separate function, gated at the call
           site, because the generated unit tests assert that
           `dotenv_key_is_denied` passes keys outside the CLI's prefix.

  Patch 3  load_dotenv_filtered's doc comment — state that rule in place of
           the generated "only the deny-listed keys are dropped".

  Patch 4  load_dotenv_filtered — skip a key unless it is allowed and not
           denied.

  Patch 5  warn_ignored_dotenv_keys — say why a key was ignored in terms of
           the allow-list; the generated text names only transport and
           process execution.

The oracle for the injected Rust is tests/dotenv_filter.rs: it fails on the
generated tree and passes on the patched one.

.fern/replay.yml excludes src/lib.rs, as it does src/http.rs and
src/sdk_executor.rs: this script owns the file's one deviation from generated
output, so a frozen Replay copy of the same fix could only conflict with it.

Anchors are exact text with an exactly-one-match-or-SystemExit discipline,
never a silent no-op: when regeneration changes the file's shape, the next
fern-bot PR fails this script loudly and someone updates the anchors with the
defect status in view, rather than silently shipping the unpatched loader.

This is a deliberate local fork of upstream generator behaviour. When ENG-12663
lands in fern-api/fern and the generator pin bumps, delete this script, its
tests, both workflow call sites, the `check-dotenv` and `patch-dotenv`
recipes in the justfile, `src/lib.rs` in refresh-lockfile.yml's commit
allowlist and message, that file's "--- .env deny-list patches ---" note, and
the `src/lib.rs` entry in .fern/replay.yml, so the fix is not applied twice.

Usage:
  patch-dotenv-denylist.py <lib.rs>
      apply every patch in place
  patch-dotenv-denylist.py --check <lib.rs>
      report drift, write nothing
"""

from __future__ import annotations

import sys

USAGE = """usage:
  {prog} <lib.rs>
      apply every patch in place
  {prog} --check <lib.rs>
      report drift, write nothing"""

# DOTENV_DENIED_SUFFIXES's last entry and closing bracket as the generator
# emits them.
SUFFIX_ANCHOR = (
    '    "_ALLOW_CROSS_HOST_PAGINATION",\n'
    "];\n"
)

SUFFIX_PATCH = (
    '    "_ALLOW_CROSS_HOST_PAGINATION",\n'
    "    // ENG-12596 (patched post-generation; delete when ENG-12663 lands\n"
    "    // upstream): names the directory the CLI writes its logs to.\n"
    '    "_LOG_FILE",\n'
    "];\n"
)

# dotenv_key_is_denied as the generator emits it.
MATCH_ANCHOR = (
    "pub(crate) fn dotenv_key_is_denied(key: &str, prefix: &str) -> bool {\n"
    "    if DOTENV_DENIED_BARE.contains(&key) {\n"
    "        return true;\n"
    "    }\n"
    "    key.strip_prefix(prefix)\n"
    "        .is_some_and(|rest| DOTENV_DENIED_SUFFIXES.contains(&rest))\n"
    "}\n"
)

# The same function compared in any case, followed by the allow-list.
MATCH_PATCH = (
    "pub(crate) fn dotenv_key_is_denied(key: &str, prefix: &str) -> bool {\n"
    "    // ENG-12596 (patched post-generation; delete when ENG-12663 lands\n"
    "    // upstream): compare in any case. Windows resolves a variable whatever\n"
    "    // case its name is set in, so `Hedra_Cli_Base_Url` must be denied like\n"
    "    // `HEDRA_CLI_BASE_URL`.\n"
    "    let key = key.to_ascii_uppercase();\n"
    "    if DOTENV_DENIED_BARE.contains(&key.as_str()) {\n"
    "        return true;\n"
    "    }\n"
    "    key.strip_prefix(&prefix.to_ascii_uppercase())\n"
    "        .is_some_and(|rest| DOTENV_DENIED_SUFFIXES.contains(&rest))\n"
    "}\n"
    "\n"
    "/// True when a `.env` file may set `key` at all, before\n"
    "/// [`dotenv_key_is_denied`] applies: the CLI vendor's own variables, which\n"
    "/// start with the first `_`-separated segment of `prefix` (`HEDRA_` for\n"
    "/// `HEDRA_CLI`), and `NO_COLOR`, compared in any case.\n"
    "///\n"
    "/// ENG-12596 (patched post-generation; delete when ENG-12663 lands\n"
    "/// upstream). Libraries the CLI links read variables of their own, so a\n"
    "/// list of keys to refuse cannot be complete. Every other key has to come\n"
    "/// from the real environment.\n"
    "pub(crate) fn dotenv_key_is_allowed(key: &str, prefix: &str) -> bool {\n"
    '    const ALSO_ALLOWED: &[&str] = &["NO_COLOR"];\n'
    "    let key = key.to_ascii_uppercase();\n"
    "    let prefix = prefix.to_ascii_uppercase();\n"
    "    let vendor = match prefix.split('_').next() {\n"
    '        Some(segment) if !segment.is_empty() => format!("{segment}_"),\n'
    '        _ => format!("{prefix}_"),\n'
    "    };\n"
    "    key.starts_with(&vendor) || ALSO_ALLOWED.contains(&key.as_str())\n"
    "}\n"
)

# load_dotenv_filtered's doc comment, up to its note on the return value, as
# the generator emits it.
DOC_ANCHOR = (
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
)

DOC_PATCH = (
    "/// Load `.env`, keeping only the keys a `.env` file may set.\n"
    "///\n"
    "/// A generated CLI is routinely run inside repositories the operator did not\n"
    "/// write, and `dotenvy::dotenv()` searches the working directory and its\n"
    "/// ancestors — so an attacker-authored `.env` would otherwise be able to point\n"
    "/// the CLI at another host, disable certificate verification, route requests\n"
    "/// through a proxy, turn off the cross-host redirect and pagination guards, or\n"
    "/// name an arbitrary program for the CLI to execute as its pager. The last is\n"
    "/// the sharpest: `<PREFIX>_PAGER` is run as a command.\n"
    "///\n"
    "/// ENG-12596 (patched post-generation; delete when ENG-12663 lands\n"
    "/// upstream): a key loads only when [`dotenv_key_is_allowed`] admits it, as\n"
    "/// one of the CLI vendor's own variables or `NO_COLOR`, and\n"
    "/// [`dotenv_key_is_denied`] does not refuse it as one of the transport, TLS,\n"
    "/// pager and log-file settings in [`DOTENV_DENIED_BARE`] and\n"
    "/// [`DOTENV_DENIED_SUFFIXES`]. Credentials and output preferences still\n"
    "/// load. Real process environment always wins, matching\n"
    "/// `dotenvy::dotenv()`'s own precedence, so anything genuinely exported by\n"
    "/// the operator's shell still applies.\n"
)

# load_dotenv_filtered's filter call as the generator emits it.
LOAD_ANCHOR = "        if dotenv_key_is_denied(&key, &prefix) {\n"

LOAD_PATCH = (
    "        // ENG-12596 (patched post-generation; delete when ENG-12663 lands\n"
    "        // upstream): a key loads only when `dotenv_key_is_allowed` admits it\n"
    "        // and the deny-lists named above do not refuse it.\n"
    "        if !dotenv_key_is_allowed(&key, &prefix) || dotenv_key_is_denied(&key, &prefix) {\n"
)

# warn_ignored_dotenv_keys's message as the generator emits it.
WARN_ANCHOR = (
    '            "ignoring {key} from .env: it controls transport or process execution, and a \\\n'
    '             .env file is not a trusted source for it. Export it from your shell instead."\n'
)

WARN_PATCH = (
    '            "ignoring {key} from .env: a .env file may set only this CLI\'s own \\\n'
    "             variables and NO_COLOR, other than its transport, TLS, pager and \\\n"
    "             log-file settings. \\\n"
    '             Export it from your shell instead."\n'
)

# (site, anchor, patched body), in application order.
PATCHES = (
    ("DOTENV_DENIED_SUFFIXES", SUFFIX_ANCHOR, SUFFIX_PATCH),
    ("dotenv_key_is_denied", MATCH_ANCHOR, MATCH_PATCH),
    ("load_dotenv_filtered doc", DOC_ANCHOR, DOC_PATCH),
    ("load_dotenv_filtered", LOAD_ANCHOR, LOAD_PATCH),
    ("warn_ignored_dotenv_keys", WARN_ANCHOR, WARN_PATCH),
)


def state(source: str, path: str, site: str, anchor: str, patched_body: str) -> str:
    """"patched" or "unpatched"; SystemExit on any shape the anchors miss.

    The two shapes are mutually exclusive — each patch breaks up its anchor's
    contiguous text — so anything but exactly one of them means regeneration
    moved `site` and the anchors need updating by hand.
    """
    patched = source.count(patched_body)
    anchored = source.count(anchor)
    if patched == 1 and anchored == 0:
        return "patched"
    if patched == 0 and anchored == 1:
        return "unpatched"
    raise SystemExit(
        f"{path}: {site} matches neither the generated nor the patched "
        f"shape (anchor x{anchored}, patch x{patched}); refusing to guess"
    )


def apply(source: str, path: str, site: str, anchor: str, patched_body: str) -> str:
    """`source` with `patched_body` in place of `anchor`; idempotent."""
    if state(source, path, site, anchor, patched_body) == "patched":
        return source
    return source.replace(anchor, patched_body, 1)


def _read(path: str) -> str:
    try:
        with open(path, encoding="utf-8") as fh:
            return fh.read()
    except OSError as exc:
        raise SystemExit(f"{path}: {exc.strerror or exc}")


def _write(path: str, source: str) -> None:
    with open(path, "w", encoding="utf-8") as fh:
        fh.write(source)


def main(argv: list[str]) -> int:
    prog = argv[0] if argv else "patch-dotenv-denylist.py"
    args = argv[1:]

    check = bool(args) and args[0] == "--check"
    if check:
        args = args[1:]

    if len(args) != 1:
        raise SystemExit(USAGE.format(prog=prog))

    (lib_path,) = args
    source = _read(lib_path)

    # Classify every site before writing anything, so a drifted second anchor
    # fails the run without leaving the first patch half-applied on disk.
    states = [(site, state(source, lib_path, site, a, p)) for site, a, p in PATCHES]

    if check:
        drift = [site for site, s in states if s != "patched"]
        for site, s in states:
            print(f"{lib_path}: {site} {'patched' if s == 'patched' else 'NOT patched'}")
        if drift:
            print(f"{len(drift)} .env deny-list patch(es) missing: {', '.join(drift)}")
            return 1
        return 0

    for site, anchor, patched_body in PATCHES:
        source = apply(source, lib_path, site, anchor, patched_body)
    for site, s in states:
        print(f"{lib_path}: {site} {'already patched (unchanged)' if s == 'patched' else 'patched'}")
    if any(s != "patched" for _, s in states):
        _write(lib_path, source)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
