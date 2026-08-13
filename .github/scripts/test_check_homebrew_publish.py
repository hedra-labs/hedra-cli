#!/usr/bin/env python3
"""Tests for check-homebrew-publish.py.

    python3 -m unittest discover -s .github/scripts

The guard's whole job is to notice a revert that no human is watching for, so
the cases that matter are the two directions in opposition: the pull-request
form must pass, and the `git push` form cargo-dist regenerates must fail. Both
are exercised against the real repository tree as well as synthetic ones, so a
future refactor that quietly stops matching anything is a test failure rather
than a green check on an unguarded repo.

The third direction matters just as much and is easier to forget: Homebrew
publishing switched off entirely — no publish job, no tap checkout — must stay
green. hedra-cli disables it on branches that are not ready to publish, and a
guard that demanded the job exist would hold those branches red.

Needs PyYAML, like the script under test. Otherwise stdlib only.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import tempfile
import textwrap
import unittest
from pathlib import Path

_SCRIPT = Path(__file__).resolve().parent / "check-homebrew-publish.py"
_REPO = _SCRIPT.parent.parent.parent
_spec = importlib.util.spec_from_file_location("check_homebrew_publish", _SCRIPT)
chp = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(chp)


# The generated job, as cargo-dist 0.32.0 emits it from `publish-jobs =
# ["homebrew"]`, trimmed to the parts the guard reads. This is the state a
# `dist init` or a cargo-dist bump would restore.
GENERATED_PUSH_JOB = """\
name: Release
on:
  push:
    tags: ["**[0-9]+.[0-9]+.[0-9]+*"]
jobs:
  publish-homebrew-formula:
    runs-on: "ubuntu-22.04"
    env:
      PLAN: ${{ needs.plan.outputs.val }}
      GITHUB_USER: "axo bot"
    steps:
      - uses: actions/checkout@v7
        with:
          persist-credentials: true
          repository: "hedra-labs/homebrew-tap"
          token: ${{ secrets.HOMEBREW_TAP_TOKEN }}
      - name: Commit formula files
        run: |
          git config --global user.name "${GITHUB_USER}"

          for release in $(echo "$PLAN" | jq --compact-output '.releases[]'); do
            filename=$(echo "$release" | jq '.artifacts[] | select(endswith(".rb"))' --raw-output)
            git add "Formula/${filename}"
            git commit -m "formula"
          done
          git push
"""

PR_JOB = """\
name: Publish the Homebrew formula by pull request
on:
  workflow_call:
    inputs:
      plan:
        required: true
        type: string
jobs:
  homebrew-pr:
    runs-on: ubuntu-22.04
    env:
      TAP: hedra-labs/homebrew-tap
    steps:
      - uses: actions/checkout@v7
        with:
          repository: hedra-labs/homebrew-tap
      - name: Commit the formulae onto a branch
        run: |
          # The built-in job ended here with a bare `git push`, which 422s.
          git checkout -B "$branch"
          git commit -m "formula"
          git push --force origin "HEAD:${branch}"
      - name: Open the tap pull request
        run: |
          gh pr create --repo "$TAP" --base main --head "$BRANCH" --title "$TITLE"
"""

UNRELATED_JOB = """\
name: CI
on: [pull_request]
jobs:
  rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - run: cargo test --locked
"""

CONFIG = """\
[workspace]
members = ["cargo:."]

[dist]
installers = ["shell", "homebrew"]
tap = "hedra-labs/homebrew-tap"
publish-jobs = {publish_jobs}
"""


def build(root: Path, publish_jobs: str, workflows: dict[str, str]) -> None:
    (root / "dist-workspace.toml").write_text(CONFIG.format(publish_jobs=publish_jobs))
    directory = root / ".github/workflows"
    directory.mkdir(parents=True, exist_ok=True)
    for name, body in workflows.items():
        (directory / name).write_text(body)


def run(root: Path) -> tuple[int, str]:
    out = io.StringIO()
    with contextlib.redirect_stdout(out):
        code = chp.main(["check-homebrew-publish.py", str(root)])
    return code, out.getvalue()


class TreeCase(unittest.TestCase):
    @contextlib.contextmanager
    def tree(self, publish_jobs: str, workflows: dict[str, str]):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            build(root, publish_jobs, workflows)
            yield root


class TestTheRepository(TreeCase):
    """The guard has to hold on the tree it ships in, or it guards nothing."""

    def test_this_repo_passes(self):
        code, out = run(_REPO)
        self.assertEqual(code, 0, out)
        self.assertIn("nothing publishes to the tap by pushing", out)

    def test_this_repo_actually_inspects_a_job(self):
        # Guards against the silent-no-op failure: a rules change that stops
        # matching the publish job would otherwise pass forever.
        _, out = run(_REPO)
        self.assertNotIn("Checked 0 job(s)", out)


class TestTheRevert(TreeCase):
    """The direction that matters: cargo-dist's pushing job coming back."""

    def test_the_generated_push_job_fails(self):
        with self.tree('["homebrew", "npm"]', {"release.yml": GENERATED_PUSH_JOB}) as root:
            code, out = run(root)
        self.assertEqual(code, 1)
        self.assertIn("bare `git push`", out)
        self.assertIn("publish-homebrew-formula", out)

    def test_the_builtin_in_publish_jobs_fails_on_its_own(self):
        # Caught at the root cause, before release.yml has been regenerated
        # from it — the window where the config and the workflow disagree.
        with self.tree('["homebrew", "npm"]', {"release.yml": UNRELATED_JOB}) as root:
            code, out = run(root)
        self.assertEqual(code, 1)
        self.assertIn('publish-jobs contains the built-in "homebrew" job', out)

    def test_an_explicit_push_to_main_fails(self):
        job = PR_JOB.replace('git push --force origin "HEAD:${branch}"', "git push origin HEAD:main")
        with self.tree('["./publish-homebrew-pr"]', {"publish-homebrew-pr.yml": job}) as root:
            code, out = run(root)
        self.assertEqual(code, 1)
        self.assertIn("pushes straight to", out)

    def test_a_tap_job_that_opens_no_pr_fails(self):
        job = PR_JOB.replace('gh pr create --repo "$TAP" --base main --head "$BRANCH" --title "$TITLE"', "true")
        with self.tree('["./publish-homebrew-pr"]', {"publish-homebrew-pr.yml": job}) as root:
            code, out = run(root)
        self.assertEqual(code, 1)
        self.assertIn("never runs `gh pr create`", out)

    def test_a_custom_job_whose_workflow_is_missing_fails(self):
        with self.tree('["./publish-homebrew-pr"]', {"ci.yml": UNRELATED_JOB}) as root:
            code, out = run(root)
        self.assertEqual(code, 1)
        self.assertIn("does not exist", out)

    def test_a_custom_job_that_is_not_reusable_fails(self):
        job = PR_JOB.replace("on:\n  workflow_call:", "on:\n  push:")
        job = job.replace("    inputs:\n      plan:\n        required: true\n        type: string\n", "")
        with self.tree('["./publish-homebrew-pr"]', {"publish-homebrew-pr.yml": job}) as root:
            code, out = run(root)
        self.assertEqual(code, 1)
        self.assertIn("reusable workflow", out)


class TestTheGoodForms(TreeCase):
    def test_the_pull_request_job_passes(self):
        with self.tree('["./publish-homebrew-pr", "npm"]', {"publish-homebrew-pr.yml": PR_JOB}) as root:
            code, out = run(root)
        self.assertEqual(code, 0, out)
        self.assertIn("Checked 1 job(s)", out)

    def test_homebrew_publishing_switched_off_passes(self):
        # The state main is in between disabling the push job and re-enabling
        # the PR one. A guard that demanded the job exist would fail here.
        with self.tree('["npm"]', {"ci.yml": UNRELATED_JOB, "release.yml": UNRELATED_JOB}) as root:
            code, out = run(root)
        self.assertEqual(code, 0, out)
        self.assertIn("Checked 0 job(s)", out)

    def test_no_tap_configured_passes(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            build(root, '["npm"]', {"ci.yml": UNRELATED_JOB})
            config = (root / "dist-workspace.toml").read_text()
            (root / "dist-workspace.toml").write_text(
                "\n".join(ln for ln in config.splitlines() if not ln.startswith("tap ="))
            )
            code, out = run(root)
        self.assertEqual(code, 0, out)
        self.assertIn("sets no `tap`", out)

    def test_a_comment_about_git_push_does_not_trip_it(self):
        # The replacement job explains at length why the bare push was wrong.
        with self.tree('["./publish-homebrew-pr"]', {"publish-homebrew-pr.yml": PR_JOB}) as root:
            self.assertIn("bare `git push`", (root / ".github/workflows/publish-homebrew-pr.yml").read_text())
            code, out = run(root)
        self.assertEqual(code, 0, out)


class TestPushDestination(unittest.TestCase):
    def test_reads_the_destination_of_a_refspec(self):
        self.assertEqual(chp.push_destination(" origin HEAD:main"), "main")
        self.assertEqual(chp.push_destination(" --force origin HEAD:formula/v1"), "formula/v1")
        self.assertEqual(chp.push_destination(" origin main"), "main")

    def test_a_lone_remote_is_not_a_refspec(self):
        self.assertIsNone(chp.push_destination(" origin"))

    def test_a_variable_destination_is_unreadable_rather_than_main(self):
        self.assertIsNone(chp.push_destination(' origin "HEAD:${branch}"'))

    def test_the_bare_form_is_recognised_with_and_without_flags(self):
        self.assertTrue(chp.BARE_PUSH.match("          git push"))
        self.assertTrue(chp.BARE_PUSH.match("git push --force"))
        self.assertFalse(chp.BARE_PUSH.match("git push origin HEAD:x"))


class TestUncommented(unittest.TestCase):
    def test_drops_whole_line_comments_only(self):
        script = textwrap.dedent(
            """\
            # git push
              # git push
            git push origin HEAD:x  # trailing
            """
        )
        kept = chp.uncommented(script)
        self.assertEqual(kept.strip(), "git push origin HEAD:x  # trailing")


if __name__ == "__main__":
    unittest.main()
