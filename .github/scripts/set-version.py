#!/usr/bin/env python3
"""Read, check, or set the `version` key of a Cargo manifest's `[package]` table.

The three crates `dist-workspace.toml` lists as workspace members are supposed
to carry the same version, but nothing in the generator keeps them that way.
fern-config's `packageIdentity` upserts seven keys inside the *root* [package]
table and version is not one of them, and `fern generate --version` stamps the
root crate alone — there is no channel at all to the member manifests. So every
regeneration resets hedra-sdk to 0.1.0 and hedra-types to 0.0.0 while the root
keeps its real version, and the drift can only be repaired after the fact
(ENG-10219).

Two callers, both in .github/workflows/:

  refresh-lockfile.yml  --read the root version, then set it on both members,
                        before the Cargo.lock staleness gate so the alignment
                        and the lockfile refresh land in one commit.
  ci.yml                --check the same invariant as the rust job's first
                        step, which catches a hand-made regeneration PR that
                        never touched a fern-bot/** branch.

Why not `tomllib` + a writer: the stdlib has no TOML writer, and every
third-party round-trip either reformats the file or drops the comments that
carry the TLS-feature and cargo-dist rationale in the root Cargo.toml. So this
edits the single `version = "..."` line inside `[package]` and touches nothing
else — a dependency's inline `version = "0.12"` is never in scope, because the
substitution is confined to the `[package]` table's own body.

Usage:
  set-version.py <version> <manifest> [<manifest> ...]  set the version in place
  set-version.py --check <version> <manifest> [...]     report drift, write nothing
  set-version.py --read <manifest>                      print [package].version
"""

from __future__ import annotations

import re
import sys

USAGE = """usage:
  {prog} <version> <manifest> [<manifest> ...]   set the version in place
  {prog} --check <version> <manifest> [...]      report drift, write nothing
  {prog} --read <manifest>                       print [package].version"""

# Cargo requires major.minor.patch; a pre-release/build suffix is allowed and
# is what cargo-dist keys "is this a prerelease?" off. The root crate's
# `1.0.0-dev` is a pre-release under this grammar, and propagates verbatim.
SEMVER = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-((?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)"
    r"(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*))?"
    r"(?:\+([0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*))?$"
)

# The [package] table runs until the next table header at column 0, or EOF.
PACKAGE_TABLE = re.compile(r"(?ms)^\[package\][ \t]*\n(.*?)(?=^\[|\Z)")

# A top-level `version = "..."` key. Anchored at column 0 so an inline
# dependency spec such as `reqwest = { version = "0.12", ... }` cannot match.
VERSION_KEY = re.compile(r'(?m)^version[ \t]*=[ \t]*"([^"]*)"[ \t]*$')


def _package_body(source: str, path: str) -> str:
    """The [package] table's body, or a diagnostic if there isn't exactly one."""
    tables = PACKAGE_TABLE.findall(source)
    if not tables:
        raise SystemExit(f"{path}: no [package] table found")
    if len(tables) > 1:
        raise SystemExit(f"{path}: {len(tables)} [package] tables found; refusing to guess")
    return tables[0]


def _version_key(body: str, path: str) -> str:
    """The single top-level `version` value in a [package] body."""
    hits = VERSION_KEY.findall(body)
    if len(hits) != 1:
        raise SystemExit(
            f"{path}: expected exactly one top-level `version` key in [package], found {len(hits)}"
        )
    return hits[0]


def read_version(source: str, path: str) -> str:
    """The manifest's `[package].version`."""
    return _version_key(_package_body(source, path), path)


def set_version(source: str, version: str, path: str) -> str:
    """`source` with `[package].version` replaced; every other byte preserved."""
    body = _package_body(source, path)
    _version_key(body, path)
    # A lambda replacement so a backslash or `\g` in `version` cannot be read as
    # a regex escape. SEMVER already excludes both; this keeps it true if the
    # grammar is ever loosened.
    new_body = VERSION_KEY.sub(lambda _: f'version = "{version}"', body, count=1)
    start, end = PACKAGE_TABLE.search(source).span(1)
    return source[:start] + new_body + source[end:]


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
    prog = argv[0] if argv else "set-version.py"
    args = argv[1:]

    if not args:
        raise SystemExit(USAGE.format(prog=prog))

    if args[0] == "--read":
        if len(args) != 2:
            raise SystemExit(f"usage: {prog} --read <manifest>")
        print(read_version(_read(args[1]), args[1]))
        return 0

    check = args[0] == "--check"
    if check:
        args = args[1:]

    if len(args) < 2:
        raise SystemExit(USAGE.format(prog=prog))

    version, paths = args[0], args[1:]
    if not SEMVER.match(version):
        raise SystemExit(f"{version!r} is not a Cargo-style SemVer version (major.minor.patch)")

    drift = []
    for path in paths:
        source = _read(path)
        current = read_version(source, path)

        if check:
            if current == version:
                print(f'{path}: version = "{current}"')
            else:
                print(f'{path}: version = "{current}", expected "{version}"')
                drift.append(path)
        elif current == version:
            # Not rewriting an already-correct manifest is what makes a re-run
            # leave the worktree clean, which is what terminates the
            # refresh-lockfile.yml push loop.
            print(f'{path}: version = "{version}" (unchanged)')
        else:
            _write(path, set_version(source, version, path))
            print(f'{path}: version = "{current}" -> "{version}"')

    if drift:
        print(f"{len(drift)} manifest(s) not at version {version!r}: {', '.join(drift)}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
