#!/usr/bin/env python3
"""Set the `version` key of a Cargo manifest's `[package]` table, in place.

Used by .github/workflows/version.yml to stamp one version across the three
crates that `dist-workspace.toml` lists as members.

Why not `tomllib` + a writer: the stdlib has no TOML writer, and every
third-party round-trip either reformats the file or drops the comments that
carry the TLS-feature and cargo-dist rationale in the root Cargo.toml. So this
edits the single `version = "..."` line inside `[package]` and touches nothing
else — a dependency's inline `version = "0.12"` is never in scope, because the
substitution is confined to the `[package]` table's own body.

Usage:  set-version.py <version> <manifest> [<manifest> ...]
"""

from __future__ import annotations

import re
import sys

# Cargo requires major.minor.patch; a pre-release/build suffix is allowed and
# is what cargo-dist keys "is this a prerelease?" off.
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
VERSION_KEY = re.compile(r'(?m)^version[ \t]*=[ \t]*"[^"]*"[ \t]*$')


def set_version(source: str, version: str, path: str) -> str:
    tables = PACKAGE_TABLE.findall(source)
    if not tables:
        raise SystemExit(f"{path}: no [package] table found")
    if len(tables) > 1:
        raise SystemExit(f"{path}: {len(tables)} [package] tables found; refusing to guess")

    body = tables[0]
    hits = VERSION_KEY.findall(body)
    if len(hits) != 1:
        raise SystemExit(
            f"{path}: expected exactly one top-level `version` key in [package], found {len(hits)}"
        )

    new_body = VERSION_KEY.sub(f'version = "{version}"', body, count=1)
    start, end = PACKAGE_TABLE.search(source).span(1)
    return source[:start] + new_body + source[end:]


def main(argv: list[str]) -> int:
    if len(argv) < 3:
        raise SystemExit(f"usage: {argv[0]} <version> <manifest> [<manifest> ...]")

    version = argv[1]
    if not SEMVER.match(version):
        raise SystemExit(f"{version!r} is not a Cargo-style SemVer version (major.minor.patch)")

    for path in argv[2:]:
        with open(path, encoding="utf-8") as fh:
            source = fh.read()
        updated = set_version(source, version, path)
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(updated)
        print(f"{path}: version = \"{version}\"")

    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
