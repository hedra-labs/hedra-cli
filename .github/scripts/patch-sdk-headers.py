#!/usr/bin/env python3
"""Re-apply the two ENG-10226 header patches to freshly generated SDK code.

Two defects share one cause: the generator emits them and regeneration resets
them, so an ordinary in-repo fix is reverted by the next fern-bot PR. Neither
file can be frozen in .fernignore — `hedra-sdk/src/config.rs` gains real fields
(two OAuth fields in #48) and `src/sdk_executor.rs` is runtime the generator
still evolves — so, like set-version.py (ENG-10219), the fix is re-applied
after every regeneration instead:

  Patch 1  hedra-sdk/src/config.rs — the `X-Fern-SDK-Version` literal is a
           hardcoded "0.1.0"; `fern generate --version` stamps the root
           manifest and has no channel to this constant. Rewrite it to the
           root crate's [package].version (set-version.py --read).

  Patch 2  src/sdk_executor.rs — `HttpClient::with_executor` documents that
           custom headers are NOT applied on the executor path; the executor's
           transport stack is responsible for them. The CLI's executor never
           picked up that responsibility, so the X-Fern-* identity trio in
           `ClientConfig::custom_headers` is constructed and discarded. Insert
           a merge of those headers into `CliExecutor::new`'s
           `global_headers` — the channel `build_request` already stamps onto
           every request (it is how X-Hedra-Spec-Version reaches the wire).

Ordering inside one run is load-bearing: Patch 1 is applied first, so if
Patch 2's anchor has drifted the failure leaves the safe half in place. The
reverse order could put `X-Fern-SDK-Version: 0.1.0` on the wire — the merge
live, the literal stale.

Two callers, both in .github/workflows/ (same design as set-version.py):

  refresh-lockfile.yml  apply both patches right after the version alignment,
                        so patches, alignment, and the lockfile refresh land
                        in one commit on the fern-bot branch.
  ci.yml                --check as a step of the required rust job, which
                        catches a hand-made regeneration PR that never touched
                        a fern-bot/** branch before `cargo test --locked`
                        fails on the same tree with a less legible message.

The oracle for the injected Rust is tests/wire_contract.rs (hedra-cli#53):
`executor_path_preserves_config_identity_headers` and
`sdk_direct_reports_the_real_crate_version` fail on the generated tree and
pass on the patched one, and `executor_path_preserves_spec_version_header`
proves the merge does not clobber the caller's own global headers.

Anchors are exact text with an exactly-one-match-or-SystemExit discipline,
never a silent no-op: when regeneration changes either file's shape, the next
fern-bot PR fails this script loudly and someone updates the anchors with the
defect status in view, rather than silently shipping unpatched headers.

This is a deliberate local fork of upstream generator behaviour. When
ENG-10234 lands in fern-api/fern and the generator pin bumps, delete this
script, its tests, and both workflow call sites so the fix is not applied
twice.

Usage:
  patch-sdk-headers.py <version> <config.rs> <sdk_executor.rs>          apply both patches in place
  patch-sdk-headers.py --check <version> <config.rs> <sdk_executor.rs>  report drift, write nothing
"""

from __future__ import annotations

import re
import sys
from pathlib import PurePosixPath

USAGE = """usage:
  {prog} <version> <config.rs> <sdk_executor.rs>           apply both patches in place
  {prog} --check <version> <config.rs> <sdk_executor.rs>   report drift, write nothing"""

# Same grammar as set-version.py: Cargo-style SemVer, pre-release and build
# suffixes allowed. The root crate's `1.0.0-dev` is a pre-release under it.
SEMVER = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)"
    r"(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?"
    r"(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$"
)

# The [package] table's body, which runs until the next table header at column
# 0 or EOF. Scoped rather than searched file-wide so a `[lib]` table's own
# `name` key — the root manifest has one — cannot be read as the crate name.
PACKAGE_TABLE = re.compile(r"(?ms)^\[package\][ \t]*\n(.*?)(?=^\[|\Z)")

# A [package] table's `name` key, used to confirm the SDK crate's identifier
# really is its directory name with dashes swapped for underscores.
PACKAGE_NAME = re.compile(r'(?m)^name[ \t]*=[ \t]*"([^"]*)"[ \t]*$')

# The X-Fern-SDK-Version tuple in ClientConfig::default()'s custom_headers.
# Anchored to the full line so the Language/Name tuples above it can't match.
VERSION_TUPLE = re.compile(
    r'(?m)^([ \t]*)\("X-Fern-SDK-Version"\.to_string\(\), "([^"]*)"\.to_string\(\)\),[ \t]*$'
)

# CliExecutor::new as the generator emits it: the client build flowing straight
# into the struct literal. The patch inserts the merge between the two.
EXECUTOR_ANCHOR = (
    '            .expect("HttpConfig::build_client failed");\n'
    "        Self {"
)

def sdk_crate(config_path: str) -> str:
    """The SDK's Rust crate identifier, from the directory holding its config.rs.

    The patch body names the SDK crate, so it cannot be a fixed string: the
    ENG-10291 rename moved hedra-sdk/ -> hedra-cli-sdk/, and with it the crate
    from `hedra_sdk` to `hedra_cli_sdk`. Taking the name from the config.rs path
    the caller already passes keeps the inserted code and the crate it calls
    into in lockstep by construction — one argument, not two that can disagree.

    Cargo's own directory-to-crate convention (dashes become underscores) is
    what makes this exact rather than a guess, and it is checked below against
    the manifest.
    """
    parts = PurePosixPath(config_path).parts
    if len(parts) < 2 or parts[1] != "src":
        raise SystemExit(
            f"{config_path}: expected <crate-dir>/src/config.rs; cannot infer the SDK crate name"
        )
    crate = parts[0].replace("-", "_")

    # Verify rather than assume: a crate whose [package] name diverges from its
    # directory would otherwise yield a patch that references a crate that does
    # not exist, and the failure would surface as a Rust compile error far from
    # here.
    manifest = PurePosixPath(parts[0]) / "Cargo.toml"
    try:
        with open(manifest, encoding="utf-8") as fh:
            table = PACKAGE_TABLE.search(fh.read())
    except OSError as exc:
        raise SystemExit(f"{manifest}: {exc.strerror or exc}")
    if not table:
        raise SystemExit(f"{manifest}: no [package] table found")
    declared = PACKAGE_NAME.search(table.group(1))
    if not declared:
        raise SystemExit(f"{manifest}: no [package] name key found")
    if declared.group(1) != crate:
        raise SystemExit(
            f"{manifest}: package name {declared.group(1)!r} does not match the "
            f"directory-derived {crate!r}; refusing to guess which the patch should call"
        )
    return crate


def executor_patch(crate: str) -> str:
    """The patched `CliExecutor::new` body, calling into `crate`."""
    return (
        '            .expect("HttpConfig::build_client failed");\n'
        "        // ENG-10226 (patched post-generation; delete when ENG-10234 lands\n"
        "        // upstream): `HttpClient::with_executor` skips `apply_custom_headers`\n"
        "        // by contract — the executor's transport stack owns those headers — so\n"
        "        // the SDK's identity trio must be re-supplied on the global-header\n"
        "        // channel that does reach the wire. Entries the caller already\n"
        "        // supplies win over the SDK defaults.\n"
        "        let mut global_headers = global_headers;\n"
        f"        let mut identity: Vec<(String, String)> = {crate}::ClientConfig::default()\n"
        "            .custom_headers\n"
        "            .into_iter()\n"
        "            .filter(|(name, _)| {\n"
        "                !global_headers\n"
        "                    .iter()\n"
        "                    .any(|(have, _)| have.eq_ignore_ascii_case(name))\n"
        "            })\n"
        "            .collect();\n"
        "        identity.sort();\n"
        "        global_headers.extend(identity);\n"
        "        Self {"
    )


def _version_tuple(source: str, path: str) -> re.Match:
    """The single X-Fern-SDK-Version tuple, or a diagnostic."""
    hits = list(VERSION_TUPLE.finditer(source))
    if not hits:
        raise SystemExit(f"{path}: no X-Fern-SDK-Version literal found")
    if len(hits) > 1:
        raise SystemExit(
            f"{path}: {len(hits)} X-Fern-SDK-Version literals found; refusing to guess"
        )
    return hits[0]


def read_sdk_version(source: str, path: str) -> str:
    """The version the X-Fern-SDK-Version header would currently report."""
    return _version_tuple(source, path).group(2)


def set_sdk_version(source: str, version: str, path: str) -> str:
    """`source` with the literal replaced; every other byte preserved."""
    match = _version_tuple(source, path)
    line = f'{match.group(1)}("X-Fern-SDK-Version".to_string(), "{version}".to_string()),'
    return source[: match.start()] + line + source[match.end() :]


def executor_state(source: str, path: str, crate: str) -> str:
    """"patched" or "unpatched"; SystemExit on any shape the anchors miss.

    The two shapes are mutually exclusive — the patch breaks up the anchor's
    contiguous text — so anything but exactly one of them means regeneration
    moved `CliExecutor::new` and the anchors need updating by hand.

    A body patched against a *different* SDK crate reads as neither shape, and
    fails loudly here. That is the intended behaviour: after the ENG-10291
    rename the tree briefly held a merge calling the old `hedra_sdk`, which no
    longer compiles, and silently accepting it would have hidden a broken tree.
    """
    patched = source.count(executor_patch(crate))
    anchored = source.count(EXECUTOR_ANCHOR)
    if patched == 1 and anchored == 0:
        return "patched"
    if patched == 0 and anchored == 1:
        return "unpatched"
    raise SystemExit(
        f"{path}: CliExecutor::new matches neither the generated nor the patched "
        f"shape (anchor x{anchored}, patch x{patched}); refusing to guess"
    )


def patch_executor(source: str, path: str, crate: str) -> str:
    """`source` with the identity-header merge in place; idempotent."""
    if executor_state(source, path, crate) == "patched":
        return source
    return source.replace(EXECUTOR_ANCHOR, executor_patch(crate), 1)


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
    prog = argv[0] if argv else "patch-sdk-headers.py"
    args = argv[1:]

    if not args:
        raise SystemExit(USAGE.format(prog=prog))

    check = args[0] == "--check"
    if check:
        args = args[1:]

    if len(args) != 3:
        raise SystemExit(USAGE.format(prog=prog))

    version, config_path, executor_path = args
    if not SEMVER.match(version):
        raise SystemExit(f"{version!r} is not a Cargo-style SemVer version (major.minor.patch)")

    drift = []

    # Patch 1 first — see the sequencing note in the module docstring.
    source = _read(config_path)
    current = read_sdk_version(source, config_path)
    if check:
        if current == version:
            print(f'{config_path}: X-Fern-SDK-Version = "{current}"')
        else:
            print(f'{config_path}: X-Fern-SDK-Version = "{current}", expected "{version}"')
            drift.append(config_path)
    elif current == version:
        print(f'{config_path}: X-Fern-SDK-Version = "{version}" (unchanged)')
    else:
        _write(config_path, set_sdk_version(source, version, config_path))
        print(f'{config_path}: X-Fern-SDK-Version = "{current}" -> "{version}"')

    crate = sdk_crate(config_path)
    source = _read(executor_path)
    state = executor_state(source, executor_path, crate)
    if check:
        if state == "patched":
            print(f"{executor_path}: identity-header merge present")
        else:
            print(f"{executor_path}: identity-header merge missing from CliExecutor::new")
            drift.append(executor_path)
    elif state == "patched":
        print(f"{executor_path}: identity-header merge already present (unchanged)")
    else:
        _write(executor_path, patch_executor(source, executor_path, crate))
        print(f"{executor_path}: merged the identity headers into CliExecutor::new")

    if drift:
        print(
            f"{len(drift)} file(s) missing the ENG-10226 header patches: {', '.join(drift)}"
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
