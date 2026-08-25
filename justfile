# hedra-cli task runner.
#
# Releasing this repo is one `git push` of a tag — which is exactly why it is
# easy to get wrong. The version is *stamped by fern*, not chosen here, the tag
# has to equal it, and the tag is the only gate: pushing it publishes to GitHub
# Releases, npm (@hedra/cli) and the Homebrew tap with no further approval.
#
# Everything under the `release` group is that knowledge, made executable.
# `just release-check` writes nothing and can be run at any time.

set shell := ["bash", "-euo", "pipefail", "-c"]

sdk_dir := "hedra-cli-sdk"
types_dir := "hedra-cli-types"
scripts := ".github/scripts"
remote := "origin"
branch := "main"

# Required status checks on `main` — keep in sync with branch protection.
required_checks := "rust regen-shape"

[private]
default:
    @just --list --unsorted

# ---------------------------------------------------------------- version ---

# Print the release version (the root crate's [package].version).
[group('version')]
version:
    @python3 {{ scripts }}/set-version.py --read Cargo.toml

# Every fern regeneration resets hedra-cli-sdk to 0.1.0 and hedra-cli-types to
# 0.0.0 while the root keeps its real version (ENG-10219). refresh-lockfile.yml
# repairs that on fern-bot/** branches; this is the same repair by hand, for a
# regeneration run locally.
[doc('Re-align the member manifests on the root version, then refresh Cargo.lock.')]
[group('version')]
sync-version:
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(python3 {{ scripts }}/set-version.py --read Cargo.toml)"
    python3 {{ scripts }}/set-version.py "$version" \
        {{ sdk_dir }}/Cargo.toml {{ types_dir }}/Cargo.toml
    cargo fetch
    echo "aligned all three manifests on $version"

# Force a version across all three manifests. Rare — fern normally stamps it.
[group('version')]
set-version v:
    #!/usr/bin/env bash
    set -euo pipefail
    python3 {{ scripts }}/set-version.py "{{ v }}" \
        Cargo.toml {{ sdk_dir }}/Cargo.toml {{ types_dir }}/Cargo.toml
    cargo fetch
    echo "set all three manifests to {{ v }}"

# Re-apply the X-Fern-* header patches the generator drops (ENG-10234).
[group('version')]
patch-headers:
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(python3 {{ scripts }}/set-version.py --read Cargo.toml)"
    python3 {{ scripts }}/patch-sdk-headers.py "$version" \
        {{ sdk_dir }}/src/config.rs src/http.rs src/sdk_executor.rs

# --------------------------------------------------------------------- ci ---

# There is deliberately no formatting recipe: CI checks none, and the generated
# tree is not rustfmt-clean, so a repo-wide `cargo fmt` would reformat generated
# files.
[doc('Everything the required `rust` job runs, in its order.')]
[group('ci')]
ci: check-versions check-headers check-homebrew test-scripts check-default check-rustls test

# The three workspace crates carry the same version.
[group('ci')]
check-versions:
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(python3 {{ scripts }}/set-version.py --read Cargo.toml)"
    python3 {{ scripts }}/set-version.py --check "$version" \
        {{ sdk_dir }}/Cargo.toml {{ types_dir }}/Cargo.toml \
        || { echo "hint: just sync-version" >&2; exit 1; }

# The SDK header patches survived the last regeneration.
[group('ci')]
check-headers:
    #!/usr/bin/env bash
    set -euo pipefail
    version="$(python3 {{ scripts }}/set-version.py --read Cargo.toml)"
    python3 {{ scripts }}/patch-sdk-headers.py --check "$version" \
        {{ sdk_dir }}/src/config.rs src/http.rs src/sdk_executor.rs \
        || { echo "hint: just patch-headers" >&2; exit 1; }

# The Homebrew publish path still opens a PR rather than pushing (ENG-10303).
[group('ci')]
check-homebrew:
    @python3 -c "import yaml" 2>/dev/null || python3 -m pip install --quiet --disable-pip-version-check --break-system-packages pyyaml
    @python3 {{ scripts }}/check-homebrew-publish.py

# Unit tests for the CI scripts themselves — unfiltered, per ENG-10310.
[group('ci')]
test-scripts:
    @python3 -m unittest discover -s {{ scripts }}

[group('ci')]
check-default:
    cargo check --locked --all-targets

# The feature set release binaries are actually built with.
[group('ci')]
check-rustls:
    cargo check --locked --all-targets --no-default-features --features rustls

[group('ci')]
test:
    cargo test --locked

# ---------------------------------------------------------------- release ---

# Show what the next tag would ship, without touching anything.
[group('release')]
release-plan:
    #!/usr/bin/env bash
    set -euo pipefail
    dist plan
    just _clean-dist-lockfiles

# `dist plan` writes a Cargo.lock into each member crate, and neither is
# gitignored — crate-layout.sh calls a stray one out by name as something that
# waves a broken layout through. Remove them, but never a tracked file.
[private]
_clean-dist-lockfiles:
    #!/usr/bin/env bash
    set -euo pipefail
    for f in {{ sdk_dir }}/Cargo.lock {{ types_dir }}/Cargo.lock; do
        if [ -f "$f" ] && ! git ls-files --error-unmatch "$f" >/dev/null 2>&1; then
            rm -f "$f"
        fi
    done

# Read-only preflight. Every condition that must hold before a tag is pushed.
[group('release')]
release-check:
    #!/usr/bin/env bash
    set -uo pipefail
    fail=0
    ok()   { printf '  \033[32mok\033[0m    %s\n' "$*"; }
    bad()  { printf '  \033[31mFAIL\033[0m  %s\n' "$*"; fail=1; }
    warn() { printf '  \033[33mwarn\033[0m  %s\n' "$*"; }

    version="$(python3 {{ scripts }}/set-version.py --read Cargo.toml)"
    tag="v$version"
    echo "preflight for $tag"

    # 1. The tag must name a commit on the released branch. Nothing in
    #    release.yml enforces this: `dist host` takes github.ref_name as the tag
    #    and github.sha as the release target, so a tag on any commit ships.
    head_branch="$(git rev-parse --abbrev-ref HEAD)"
    if [ "$head_branch" = "{{ branch }}" ]; then
        ok "on {{ branch }}"
    else
        bad "on '$head_branch', not {{ branch }}"
    fi

    if git diff --quiet && git diff --cached --quiet; then
        ok "working tree clean"
    else
        bad "working tree dirty — the tag would not describe what you built"
    fi

    git fetch --quiet {{ remote }} {{ branch }} 2>/dev/null || warn "could not fetch {{ remote }}"
    if [ "$(git rev-parse HEAD)" = "$(git rev-parse {{ remote }}/{{ branch }} 2>/dev/null)" ]; then
        ok "HEAD matches {{ remote }}/{{ branch }}"
    else
        bad "HEAD differs from {{ remote }}/{{ branch }} — push or pull first"
    fi

    # 2. Advisory, not a blocker — cargo-dist reads the ROOT crate's version
    #    alone, exactly as ci.yml's own comment says. Verified rather than
    #    assumed: with the members drifted to 0.1.0/0.0.0, `dist plan` still
    #    announces v3.0.0, because a library crate with no binaries is not
    #    dist-able and never joins the announcement. Drift is still a real
    #    regeneration bug (ENG-10219) that fails a PR, so it is worth saying —
    #    it just cannot break the release you are about to cut.
    if python3 {{ scripts }}/set-version.py --check "$version" \
            {{ sdk_dir }}/Cargo.toml {{ types_dir }}/Cargo.toml >/dev/null 2>&1; then
        ok "all three manifests at $version"
    else
        warn "member manifests drifted — harmless here; fix with: just sync-version"
    fi

    # 3. Re-tagging is not a release. cargo-dist would collide with a GitHub
    #    Release that already exists, and npm rejects a republished version.
    if git rev-parse -q --verify "refs/tags/$tag" >/dev/null; then
        bad "local tag $tag already exists"
    else
        ok "local tag $tag is free"
    fi
    if [ -z "$(git ls-remote --tags {{ remote }} "refs/tags/$tag" 2>/dev/null)" ]; then
        ok "remote tag $tag is free"
    else
        bad "$tag already exists on {{ remote }} — bump the version instead"
    fi

    # 4. release.yml is generated from dist-workspace.toml. An edit there without
    #    `dist generate` leaves the workflow stale and fails the plan check.
    if command -v dist >/dev/null 2>&1; then
        if dist generate --check >/dev/null 2>&1; then
            ok "release.yml matches dist-workspace.toml"
        else
            bad "release.yml is stale — run: just dist-generate"
        fi
        if dist plan >/dev/null 2>&1; then
            ok "dist plan succeeds"
        else
            bad "dist plan fails — run: just release-plan"
        fi
        just _clean-dist-lockfiles
    else
        warn "dist not installed; skipped the generate/plan checks"
    fi

    # 5. The tag bypasses CI entirely — release.yml never waits for it. So the
    #    only moment anyone can notice a red HEAD is right now.
    if command -v gh >/dev/null 2>&1; then
        sha="$(git rev-parse HEAD)"
        repo="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null)"
        for check in {{ required_checks }}; do
            concl="$(gh api "repos/$repo/commits/$sha/check-runs" \
                --jq ".check_runs[] | select(.name==\"$check\") | .conclusion" 2>/dev/null | head -1)"
            case "$concl" in
                success) ok "check '$check' green on HEAD" ;;
                "")      bad "check '$check' has not run on HEAD" ;;
                *)       bad "check '$check' is '$concl' on HEAD" ;;
            esac
        done
    else
        warn "gh not installed; skipped the CI status checks"
    fi

    echo
    if [ "$fail" -eq 0 ]; then
        echo "ready to release $tag — run: just release"
    else
        echo "preflight failed; $tag is not safe to push"
        exit 1
    fi

# The confirmation is inside the body, not a [confirm] attribute: `just` prompts
# for that attribute *before* running dependencies, so the preflight would not
# have gated the prompt. Typing the tag back is deliberate — the push is
# irreversible on npm, where a version can never be republished.
[doc('Tag HEAD and push it. This ships: GitHub Release, npm, Homebrew tap PR.')]
[group('release')]
release:
    #!/usr/bin/env bash
    set -euo pipefail
    just release-check
    version="$(python3 {{ scripts }}/set-version.py --read Cargo.toml)"
    tag="v$version"
    echo
    echo "Pushing $tag publishes to GitHub Releases, npm (@hedra/cli) and the"
    echo "Homebrew tap. Nothing gates it after this point and npm versions"
    echo "cannot be republished."
    read -r -p "Type $tag to confirm: " reply
    if [ "$reply" != "$tag" ]; then
        echo "aborted"
        exit 1
    fi
    git tag -a "$tag" -m "Release $tag"
    git push {{ remote }} "$tag"
    echo "pushed $tag — watch it with: just release-watch"

# Follow the release run the tag kicked off.
[group('release')]
release-watch:
    #!/usr/bin/env bash
    set -euo pipefail
    id="$(gh run list --workflow=release.yml --limit 1 --json databaseId -q '.[0].databaseId')"
    gh run watch "$id"

# Regenerate release.yml after any dist-workspace.toml edit.
[group('release')]
dist-generate:
    #!/usr/bin/env bash
    set -euo pipefail
    dist generate
    git status --short .github/workflows/release.yml
