# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: FSL-1.1-ALv2

"""Workflow contracts. The Rust check-public-files command enforces the
read-only permissions and the README Repository checks parity."""

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).parents[1]
WORKFLOW = (ROOT / ".github/workflows/validate.yml").read_text(encoding="utf-8")
USES_FIELD = re.compile(r"""\buses['"]?\s*:""")
PINNED_USE = re.compile(
    r"""\s*(?:-\s+)?(['"]?)uses\1\s*:\s*(['"]?)"""
    r"([A-Za-z0-9]+(?:-[A-Za-z0-9]+)*/[A-Za-z0-9_.-]+(?:/[A-Za-z0-9_.-]+)*)"
    r"@[0-9a-f]{40}\2(?:[ \t]+#[^\r\n]*)?[ \t]*"
)


def action_pins(workflow):
    """Return the actions that `uses` keys pin to a full commit SHA, and every
    line with a `uses` key that is not pinned that way, including forms this
    check cannot read, such as a flow mapping. A comment is not a step.

    Backslashes and explicit YAML keys are refused throughout this workflow."""
    actions, unpinned = set(), []
    for line in workflow.splitlines():
        if line.lstrip().startswith("#"):
            continue
        # shortcut: no escaped or explicit keys; use a YAML parser if needed.
        if "\\" in line or re.match(r"\s*(?:-\s+)?\?(?:\s|$)", line):
            unpinned.append(line.strip())
            continue
        code = re.split(r"[ \t]+#", line, maxsplit=1)[0]
        if not USES_FIELD.search(code):
            continue
        match = PINNED_USE.fullmatch(line)
        if match and all(part not in (".", "..") for part in match.group(3).split("/")):
            actions.add(match.group(3))
        else:
            unpinned.append(line.strip())
    return actions, unpinned


class WorkflowTests(unittest.TestCase):
    def test_action_pins_refuse_every_reference_that_is_not_a_commit_sha(self):
        sha = "3d3c42e5aac5ba805825da76410c181273ba90b1"
        pinned = [
            f"      - uses: owner/action@{sha} # v1.2.3",
            f"        uses:  owner/action@{sha}",
            f"      - uses: 'owner/action@{sha}'",
            f'      - uses: "owner/path/action@{sha}"',
            f"      - uses : owner/action@{sha}",
            f'      - "uses": owner/action@{sha}',
            "      # - uses: owner/action@v1",
            r'      # "u\u0073es": owner/action@v1',
        ]
        self.assertEqual(action_pins("\n".join(pinned)), ({"owner/action", "owner/path/action"}, []))
        for line in (
            "      - uses: owner/action",
            "      - uses:  owner/action@v1",
            "      - uses: owner/action@v1 # v1",
            "      - uses : owner/action@v1",
            '      - "uses": owner/action@v1',
            "      - 'uses\": owner/action@v1",
            f"      - uses: owner/action@{sha[:-1]}",
            f"      - uses: 'owner/action@{sha}\"",
            f"      - uses: action@{sha}",
            f"      - {{uses: owner/action@{sha}}}",
        ):
            self.assertEqual(action_pins(line), (set(), [line.strip()]), line)

    def test_action_pins_reject_scalar_suffixes_unsupported_keys_and_invalid_names(self):
        sha = "3d3c42e5aac5ba805825da76410c181273ba90b1"
        for line in (
            f"      - uses: owner/action@{sha}#junk",
            f"      - uses: 'owner/action@{sha}#junk'",
            f'      - uses: "owner/action@{sha}#junk"',
            r'      - "u\u0073es": owner/action@v1',
            r'      - "\x75ses": owner/action@v1',
            "      - ? uses\n        : owner/action@v1",
            "      - ?\n          uses\n        : owner/action@v1",
            f"      - uses: ./action@{sha}",
            f"      - uses: café/action@{sha}",
            f"      - uses: owner/café@{sha}",
            f"      - uses: owner/../action@{sha}",
        ):
            with self.subTest(line=line):
                actions, unpinned = action_pins(line)
                self.assertEqual(actions, set())
                self.assertTrue(unpinned)

    def test_action_pins_allow_dependabot_to_change_every_action_sha(self):
        bumped = re.sub(r"@[0-9a-f]{40}", "@" + "a" * 40, WORKFLOW)
        self.assertEqual(action_pins(bumped), action_pins(WORKFLOW))

    def test_action_pins_ignore_inline_comment_key_examples(self):
        for line in (
            "name: CI # uses: owner/action@v1",
            "run: echo ok\t# uses: owner/action@v1",
        ):
            with self.subTest(line=line):
                self.assertEqual(action_pins(line), (set(), []))

    def test_workflow_pins_actions_and_runs_the_required_checks(self):
        workflow = (ROOT / ".github/workflows/validate.yml").read_text(encoding="utf-8")
        # Every action is pinned to a full commit SHA; Dependabot moves the SHAs.
        actions, unpinned = action_pins(workflow)
        self.assertEqual(unpinned, [])
        self.assertLessEqual({"actions/checkout", "actions/setup-python"}, actions)
        self.assertIn("python: [\"3.14\", \"3.15\"]", workflow)
        checker = "cargo run --manifest-path tools/github-skills/Cargo.toml --locked --"
        for command in (
            "python -m unittest discover -s tests -v",
            f"{checker} validate-skills --strict",
            f"{checker} check-public-files",
        ):
            self.assertIn(command, workflow)

    def test_workflow_lints_with_the_hash_pinned_ruff_the_guide_names(self):
        workflow = (ROOT / ".github/workflows/validate.yml").read_text(encoding="utf-8")
        lint_job = workflow.split("  lint:", 1)[1].split("\n\n", 1)[0]
        guide = (ROOT / "CONTRIBUTING.md").read_text(encoding="utf-8")

        self.assertIn("--require-hashes", lint_job)
        self.assertIn("--requirement requirements-lint.txt", lint_job)
        self.assertIn("run: python -m ruff check .", lint_job)
        self.assertIn("python -m ruff check .", guide)
        self.assertIn("--require-hashes --requirement requirements-lint.txt", guide)
        self.assertRegex(workflow, r"(?m)^    needs: \[lint, ")

    def test_workflow_runs_junction_safety_tests_on_windows(self):
        workflow = (ROOT / ".github/workflows/validate.yml").read_text(encoding="utf-8")
        native_job = workflow.split("\n  native:", 1)[1].split("\n  coverage:", 1)[0]

        self.assertIn("os: [ubuntu-latest, windows-latest]", native_job)
        self.assertIn("working-directory: tools/github-skills", native_job)
        self.assertIn("- run: cargo test --locked", native_job)


if __name__ == "__main__":
    unittest.main()
