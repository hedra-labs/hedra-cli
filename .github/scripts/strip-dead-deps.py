#!/usr/bin/env python3
"""Strip the dependencies the generator emits but nothing uses from the root Cargo.toml.

Cargo 1.100.0 added two manifest lints, `unused_dependencies` and
`redundant_homepage`, both `warn` by default. setup-rust-toolchain@v2 exports
`CARGO_BUILD_WARNINGS=deny`, which turns any cargo warning into an error, and
`RUSTFLAGS="-A warnings"` silences rustc only — so a manifest that trips either
lint fails every cargo command in CI. Fern's CLI generator template emits three
dependencies the root crate does not use:

  [dependencies]        hmac = "0.12"        no module imports it
  [build-dependencies]  serde, serde_yaml    the root crate has no build script

#186 removed them by hand, but the root Cargo.toml is generator-owned and Fern
Replay never recorded that commit as a customization (it landed on main and was
merged into the open regeneration branch of #185, ahead of the boundary the
next regeneration scanned from), so regeneration #189 put all three back. Its
refresh-lockfile run then died in `cargo fetch` on the lint warnings, before it
could align the crate versions, re-apply the SDK header patches, or refresh
Cargo.lock. The homepage half is fixed generator-side (hedra-labs/fern-config
#127 sets `packageIdentity.homepage` to the docs URL); the generator has no
option to drop template dependencies, so this is re-applied after every
regeneration — the same design as set-version.py and patch-sdk-headers.py.

Each removal is guarded so it can never delete a dependency that is in use:

  * `[build-dependencies]` goes whole, and only when the crate has no build
    script (no `build.rs` beside the manifest and no `build` key in
    [package]). Without one, every build dependency is unused by definition.
  * A name in DEAD_DEPENDENCIES goes from `[dependencies]` only when no `.rs`
    file under the manifest's directory mentions it as a word.

When the generator stops emitting them, this is a no-op — the desired end
state, so not an error. A dead dependency this script does not know about is
still caught: cargo's own lint fails the required `rust` job loudly.

Caller: refresh-lockfile.yml, before the Cargo.lock staleness gate, so the
strip, the version alignment, the header patches and the lockfile refresh land
in one commit on the fern-bot branch.

Delete this script, its tests and the workflow step once Fern's CLI template
stops emitting the three dependencies.

Usage:
  strip-dead-deps.py <Cargo.toml>   strip in place, print what was removed
"""

from __future__ import annotations

import os
import re
import sys

USAGE = "usage: {prog} <Cargo.toml>"

# Dependencies the generator's template adds to [dependencies] that the root
# crate never imports. Removed only when no Rust source mentions them.
DEAD_DEPENDENCIES = ("hmac",)

# Directories never scanned for Rust sources: build output, VCS metadata, and
# dot-directories such as .agents/worktrees, whose checkouts are other trees.
SKIP_DIRS = {"target", "node_modules"}


def table(name: str) -> re.Pattern[str]:
    """A table header at column 0 plus its body, up to the next header or EOF."""
    return re.compile(rf"(?ms)^\[{re.escape(name)}\][ \t]*\n(.*?)(?=^\[|\Z)")


PACKAGE_TABLE = table("package")
DEPENDENCIES_TABLE = table("dependencies")
BUILD_DEPENDENCIES_TABLE = table("build-dependencies")

# A top-level `build = ...` key inside [package] — an explicit build script
# path, or `build = false`. Either way the table is left alone: `false` is a
# deliberate statement this script should not second-guess.
BUILD_KEY = re.compile(r"(?m)^build[ \t]*=")


def has_build_script(text: str, root: str) -> bool:
    package = PACKAGE_TABLE.search(text)
    if package and BUILD_KEY.search(package.group(1)):
        return True
    return os.path.exists(os.path.join(root, "build.rs"))


def rust_sources_mention(root: str, name: str) -> list[str]:
    """Paths of `.rs` files under `root` that mention `name` as a whole word."""
    word = re.compile(rf"\b{re.escape(name)}\b")
    hits = []
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(
            d for d in dirnames if d not in SKIP_DIRS and not d.startswith(".")
        )
        for filename in sorted(filenames):
            if not filename.endswith(".rs"):
                continue
            path = os.path.join(dirpath, filename)
            with open(path, encoding="utf-8", errors="replace") as fh:
                if word.search(fh.read()):
                    hits.append(path)
    return hits


def strip_build_dependencies(text: str) -> tuple[str, bool]:
    match = BUILD_DEPENDENCIES_TABLE.search(text)
    if not match:
        return text, False
    before, after = text[: match.start()], text[match.end() :]
    if after:
        # Mid-file: the body ran up to the next header and already carries the
        # blank line that separated the two tables, so the text before this
        # header keeps its own separator and the layout is unchanged.
        return before + after, True
    # At EOF: drop the separator that preceded this table, and end the file
    # exactly as it ended before.
    trailing = len(text) - len(text.rstrip("\n"))
    return before.rstrip("\n") + "\n" * trailing, True


def strip_dependency(text: str, name: str) -> tuple[str, bool]:
    match = DEPENDENCIES_TABLE.search(text)
    if not match:
        return text, False
    entry = re.compile(rf"(?m)^{re.escape(name)}[ \t]*=.*\n?")
    body, count = entry.subn("", match.group(1), count=1)
    if not count:
        return text, False
    return text[: match.start(1)] + body + text[match.end(1) :], True


def strip(path: str) -> list[str]:
    """Strip the dead dependencies from `path` in place; return what was removed."""
    root = os.path.dirname(os.path.abspath(path))
    try:
        with open(path, encoding="utf-8", newline="") as fh:
            text = fh.read()
    except OSError as exc:
        raise SystemExit(f"{path}: {exc.strerror or exc}")

    removed = []
    if has_build_script(text, root):
        if BUILD_DEPENDENCIES_TABLE.search(text):
            print(f"{path}: the crate has a build script; keeping [build-dependencies]")
    else:
        text, changed = strip_build_dependencies(text)
        if changed:
            removed.append("[build-dependencies]")

    for name in DEAD_DEPENDENCIES:
        users = rust_sources_mention(root, name)
        if users:
            print(f"{path}: `{name}` is mentioned in {', '.join(users)}; keeping it")
            continue
        text, changed = strip_dependency(text, name)
        if changed:
            removed.append(name)

    if removed:
        with open(path, "w", encoding="utf-8", newline="") as fh:
            fh.write(text)
    return removed


def main(argv: list[str]) -> int:
    prog = os.path.basename(argv[0]) if argv else "strip-dead-deps.py"
    if len(argv) != 2 or argv[1].startswith("-"):
        print(USAGE.format(prog=prog), file=sys.stderr)
        return 2
    removed = strip(argv[1])
    if removed:
        print(f"{argv[1]}: removed {', '.join(removed)}")
    else:
        print(f"{argv[1]}: no dead dependencies to remove")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
