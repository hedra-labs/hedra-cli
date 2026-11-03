#!/usr/bin/env python3
"""Re-apply the two .env deny-list patches to freshly generated runtime code.

The generator emits `load_dotenv_filtered` and its deny-list in src/lib.rs, and
regeneration resets that file, so an ordinary in-repo fix is reverted by the
next fern-bot PR. src/lib.rs cannot be frozen in .fernignore — it is runtime
the generator still evolves — so, like patch-sdk-headers.py, the fix is
re-applied after every regeneration:

  Patch 1  DOTENV_DENIED_BARE — add `SSL_CERT_DIR`. On Linux, OpenSSL (glibc
           builds) loads it alongside the system roots and
           `rustls-native-certs` (musl builds) loads it in place of them, so a
           `.env` that sets it changes the roots the CLI trusts, as
           `SSL_CERT_FILE`, already listed, does.

  Patch 2  dotenv_key_is_denied — compare the key and the prefix in any case.
           reqwest reads `http_proxy`, `https_proxy`, `all_proxy` and
           `no_proxy`, and Windows resolves a variable whatever case its name
           is set in, so the generated case-sensitive match lets a `.env` set
           a proxy as `https_proxy` or `Https_Proxy`.

The oracle for the injected Rust is tests/dotenv_filter.rs: it fails on the
generated tree and passes on the patched one.

.fern/replay.yml excludes src/lib.rs, as it does src/http.rs and
src/sdk_executor.rs: this script owns the file's one deviation from generated
output, so a frozen Replay copy of the same fix could only conflict with it.

Anchors are exact text with an exactly-one-match-or-SystemExit discipline,
never a silent no-op: when regeneration changes the file's shape, the next
fern-bot PR fails this script loudly and someone updates the anchors with the
defect status in view, rather than silently shipping the unpatched deny-list.

This is a deliberate local fork of upstream generator behaviour. When ENG-12663
lands in fern-api/fern and the generator pin bumps, delete this script, its
tests, and both workflow call sites so the fix is not applied twice.

Usage:
  patch-dotenv-denylist.py <lib.rs>
      apply both patches in place
  patch-dotenv-denylist.py --check <lib.rs>
      report drift, write nothing
"""

from __future__ import annotations

import sys

USAGE = """usage:
  {prog} <lib.rs>
      apply both patches in place
  {prog} --check <lib.rs>
      report drift, write nothing"""

# DOTENV_DENIED_BARE's trust-store entries as the generator emits them.
BARE_ANCHOR = (
    '    "SSL_CERT_FILE",\n'
    '    "CURL_CA_BUNDLE",\n'
)

BARE_PATCH = (
    '    "SSL_CERT_FILE",\n'
    "    // ENG-12596 (patched post-generation; delete when ENG-12663 lands\n"
    "    // upstream): on Linux, OpenSSL (glibc builds) loads `SSL_CERT_DIR`\n"
    "    // alongside the system roots and `rustls-native-certs` (musl builds) in\n"
    "    // place of them.\n"
    '    "SSL_CERT_DIR",\n'
    '    "CURL_CA_BUNDLE",\n'
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

MATCH_PATCH = (
    "pub(crate) fn dotenv_key_is_denied(key: &str, prefix: &str) -> bool {\n"
    "    // ENG-12596 (patched post-generation; delete when ENG-12663 lands\n"
    "    // upstream): compare in any case. reqwest reads the lowercase proxy\n"
    "    // variables, and Windows resolves a variable whatever case its name is\n"
    "    // set in, so `https_proxy` and `Https_Proxy` must be denied like\n"
    "    // `HTTPS_PROXY`.\n"
    "    let key = key.to_ascii_uppercase();\n"
    "    if DOTENV_DENIED_BARE.contains(&key.as_str()) {\n"
    "        return true;\n"
    "    }\n"
    "    key.strip_prefix(&prefix.to_ascii_uppercase())\n"
    "        .is_some_and(|rest| DOTENV_DENIED_SUFFIXES.contains(&rest))\n"
    "}\n"
)

# (site, anchor, patched body), in application order.
PATCHES = (
    ("DOTENV_DENIED_BARE", BARE_ANCHOR, BARE_PATCH),
    ("dotenv_key_is_denied", MATCH_ANCHOR, MATCH_PATCH),
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
