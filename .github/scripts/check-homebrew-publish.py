#!/usr/bin/env python3
"""Assert the Homebrew publish path opens a pull request instead of pushing.

hedra-labs/homebrew-tap's `main` carries the org ruleset "Protect default
branches" — deletion, non_fast_forward, and pull_request with
required_approving_review_count: 1 — whose only bypass actor is
OrganizationAdmin. cargo-dist's built-in `homebrew` publish job ends in a bare
`git push` to that branch, which returns HTTP 422 for any non-admin token. The
one successful push in the tap's history landed fourteen minutes before the
ruleset did, so nothing has ever exercised the built-in job against the rules in
force today; the failure is waiting for the next release, not visible in the
last one (ENG-10303).

The fix is `publish-jobs = ["./publish-homebrew-pr"]` — a cargo-dist custom
publish job, so the publish logic lives in a file dist does not generate. This
script exists because that wiring has one silent failure mode: `dist init` or a
cargo-dist bump rewrites release.yml from the config, and anyone who re-answers
the Homebrew question restores the built-in pushing job. Fern regeneration will
not do it (`.github/` is .fernignore'd) and no CI job runs `dist generate
--check`, so nothing else in this repo would notice — until a release either
422s or, if the ruleset is ever relaxed, quietly pushes again.

Every rule below is conditional, never an existence check. Homebrew publishing
is legitimately absent on branches where it has been disabled (`publish-jobs =
["npm"]`, no publish job at all), and this must stay green there rather than
holding those branches red. What it refuses is the *pushing* form coming back.

Requires PyYAML: the rules are about workflow structure — which job checks the
tap out, and what that same job runs — and a regex sweep over generated YAML is
exactly the fragility the guard exists to prevent. Modelled on
fern-config/scripts/test_release_workflow_paths.py.

Usage:
  check-homebrew-publish.py [<repo-root>]     defaults to this repo
"""

from __future__ import annotations

import re
import shlex
import sys
import tomllib
from pathlib import Path

import yaml

# cargo-dist's own name for the job it generates from `publish-jobs = ["homebrew"]`.
BUILTIN_HOMEBREW_JOB = "homebrew"

# `git push` carrying nothing but flags: no remote, no refspec, so it pushes the
# checked-out branch to its upstream. This is the generated form, verbatim.
BARE_PUSH = re.compile(r"^\s*git\s+push\s*(?:-[^\s]+\s*)*$")
PUSH_LINE = re.compile(r"^\s*git\s+push\b(?P<args>.*)$")


def uncommented(script: str) -> str:
    """Drop whole-line shell comments.

    The workflows explain at length why a bare `git push` is wrong, and a check
    that read comments would fail on the very file that fixes the bug.
    """
    return "\n".join(ln for ln in script.splitlines() if not ln.lstrip().startswith("#"))


def push_destination(args: str) -> str | None:
    """The branch a `git push` writes to, as far as it can be read statically.

    `origin HEAD:main` and `origin main` both answer "main"; a refspec built from
    a shell variable answers None, which callers treat as "not provably main".
    """
    try:
        tokens = [t for t in shlex.split(args, comments=True) if not t.startswith("-")]
    except ValueError:
        return None
    if not tokens:
        return None
    refspec = tokens[-1]
    if len(tokens) == 1 and ":" not in refspec:
        # A lone token is the remote (`git push origin`), not a refspec.
        return None
    dest = refspec.split(":")[-1]
    return dest if "$" not in dest else None


def job_scripts(job: dict) -> list[str]:
    """Every `run:` block in a job, comments stripped."""
    steps = job.get("steps") or []
    return [uncommented(s["run"]) for s in steps if isinstance(s, dict) and isinstance(s.get("run"), str)]


def touches_tap(job: dict, tap: str) -> bool:
    """Whether a job checks the tap out, or otherwise names it in what it runs.

    The checkout is how both the generated job and its replacement reach the
    tap; the text sweep over `env:` and `run:` is the backstop for a rewrite
    that clones or adds a remote by hand instead.
    """
    for step in job.get("steps") or []:
        if not isinstance(step, dict):
            continue
        uses = step.get("uses")
        if isinstance(uses, str) and uses.startswith("actions/checkout"):
            if (step.get("with") or {}).get("repository") == tap:
                return True
    haystack = list(job_scripts(job)) + [str(v) for v in (job.get("env") or {}).values()]
    return any(tap in text for text in haystack)


def check_publish_jobs(publish_jobs: list[str], workflows: Path, failures: list[str]) -> None:
    if BUILTIN_HOMEBREW_JOB in publish_jobs:
        failures.append(
            'dist-workspace.toml: publish-jobs contains the built-in "homebrew" job, which '
            "ends in a bare `git push` to a branch that requires a pull request (HTTP 422). "
            'Use the custom job instead: publish-jobs = ["./publish-homebrew-pr", ...].'
        )

    # A custom job names a reusable workflow release.yml will `uses:`. A missing
    # file is a release-time failure that nothing else in CI would catch.
    for job in publish_jobs:
        if not job.startswith("./"):
            continue
        path = workflows / f"{job[2:]}.yml"
        if not path.exists():
            failures.append(f"dist-workspace.toml: publish-jobs names {job!r}, but {path} does not exist.")
            continue
        doc = yaml.safe_load(path.read_text()) or {}
        # `on` is YAML 1.1's boolean true, which is why this is not doc["on"].
        triggers = doc.get(True) if True in doc else doc.get("on")
        if not isinstance(triggers, dict) or "workflow_call" not in triggers:
            failures.append(f"{path}: a custom publish job must be a reusable workflow (`on: workflow_call`).")


def check_workflow(path: Path, tap: str, failures: list[str]) -> int:
    """Rules for every job that reaches the tap. Returns how many it checked."""
    doc = yaml.safe_load(path.read_text()) or {}
    checked = 0

    for name, job in (doc.get("jobs") or {}).items():
        if not isinstance(job, dict) or not touches_tap(job, tap):
            continue
        checked += 1
        scripts = job_scripts(job)
        joined = "\n".join(scripts)

        for line in joined.splitlines():
            match = PUSH_LINE.match(line)
            if not match:
                continue
            if BARE_PUSH.match(line):
                failures.append(
                    f"{path}: job {name!r} runs a bare `git push` to {tap}. That branch requires a "
                    "pull request; the push returns HTTP 422. Push a branch and open a PR instead."
                )
                continue
            dest = push_destination(match.group("args"))
            if dest in ("main", "HEAD", "refs/heads/main"):
                failures.append(
                    f"{path}: job {name!r} pushes straight to {tap}'s {dest!r}, which requires a "
                    "pull request. Push a branch and open a PR instead."
                )

        if "gh pr create" not in joined:
            failures.append(
                f"{path}: job {name!r} reaches {tap} but never runs `gh pr create`. Publishing the "
                "formula has to go through a pull request — see ENG-10303."
            )

    return checked


def main(argv: list[str]) -> int:
    if len(argv) > 2:
        print(__doc__.strip().splitlines()[-1].strip(), file=sys.stderr)
        return 2
    root = Path(argv[1]) if len(argv) == 2 else Path(__file__).resolve().parent.parent.parent

    config = tomllib.loads((root / "dist-workspace.toml").read_text())
    dist = config.get("dist", {})
    tap = dist.get("tap")
    publish_jobs = dist.get("publish-jobs", [])
    workflows = root / ".github/workflows"

    failures: list[str] = []
    check_publish_jobs(publish_jobs, workflows, failures)

    if not tap:
        # No tap configured means no Homebrew publish to get wrong.
        print("dist-workspace.toml sets no `tap`; nothing to check.")
    else:
        checked = sum(check_workflow(p, tap, failures) for p in sorted(workflows.glob("*.yml")))
        # Deliberately not an error. Zero jobs is the correct reading on a branch
        # where Homebrew publishing is switched off, and on main between the PR
        # that disables it and the one that re-enables it.
        print(f"Checked {checked} job(s) that reach {tap}.")

    if failures:
        print()
        for failure in failures:
            print(f"FAIL: {failure}")
        return 1

    print("OK: nothing publishes to the tap by pushing.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
