import hashlib
import io
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest.mock import patch

from scripts.validate_skills import APPROVED_NAMES, check_runs, skill_digest, validate

ROOT = Path(__file__).parents[1]

SKILL = """---
name: {name}
description: Use when {description}.
---

# {name}
"""


class ValidateSkillsTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, relative, text):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def copies(self, name, text):
        self.write(Path(".agents/skills") / name / "SKILL.md", text)

    def valid_skill(self, name="github-repository-audit"):
        text = SKILL.format(name=name, description="assessing a repository")
        self.write(Path(".claude/skills") / name / "SKILL.md", text)
        self.copies(name, text)

    def valid_skill_set(self):
        for name in APPROVED_NAMES:
            text = SKILL.format(name=name, description="assessing a repository")
            self.write(Path(".claude/skills") / name / "SKILL.md", text)
            self.copies(name, text)

    def link_directory(self, link, target):
        """Point link at target with a symlink, or an NTFS junction where symlinks need elevation."""
        try:
            os.symlink(target, link, target_is_directory=True)
        except OSError as error:
            junction = subprocess.run(
                ["cmd", "/c", "mklink", "/J", str(link), str(target)],
                capture_output=True, text=True, check=False,
            )
            if junction.returncode:
                self.skipTest(f"directory links are unavailable: {error}; {junction.stderr}")

    @staticmethod
    def unlink_directory(link):
        if os.name == "nt":
            subprocess.run(["cmd", "/c", "rmdir", str(link)], capture_output=True, check=False)
        else:
            os.unlink(link)

    def test_reports_missing_frontmatter_delimiters(self):
        self.write(".claude/skills/github-repository-audit/SKILL.md", "name: github-repository-audit\n")

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: missing YAML frontmatter delimiters"],
        )

    def test_reports_frontmatter_without_a_closing_delimiter(self):
        self.write(
            ".claude/skills/github-repository-audit/SKILL.md",
            "---\nname: github-repository-audit\ndescription: Use when auditing.\n",
        )

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: missing YAML frontmatter delimiters"],
        )

    def test_ignores_other_frontmatter_lines(self):
        text = SKILL.format(name="github-repository-audit", description="assessing a repository")
        text = text.replace("---\nname:", "---\nlicence: MIT\nno separator here\nname:", 1)
        self.write(".claude/skills/github-repository-audit/SKILL.md", text)
        self.copies("github-repository-audit", text)

        self.assertEqual(validate(self.root), [])

    def test_reports_a_skill_directory_without_skill_md(self):
        (self.root / ".claude/skills/github-repository-audit").mkdir(parents=True)

        self.assertEqual(validate(self.root), [".claude/skills/github-repository-audit: missing SKILL.md"])

    def test_rejects_a_skill_md_that_is_a_link(self):
        target = self.root / "outside.md"
        target.write_text(SKILL.format(name="github-repository-audit", description="x"), encoding="utf-8")
        link = self.root / ".claude/skills/github-repository-audit/SKILL.md"
        link.parent.mkdir(parents=True)
        try:
            os.symlink(target, link)
        except OSError as error:
            self.skipTest(f"file links are unavailable: {error}")

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: is a link, not an ordinary file or directory"],
        )

    def test_rejects_a_linked_canonical_tree_before_reading_it(self):
        outside = self.root / "outside"
        (outside / "github-repository-audit").mkdir(parents=True)
        (self.root / ".claude").mkdir()
        link = self.root / ".claude/skills"
        self.link_directory(link, outside)
        try:
            for strict in (False, True):
                with self.subTest(strict=strict):
                    self.assertEqual(
                        validate(self.root, strict=strict),
                        [".claude/skills: is a link, not an ordinary file or directory"],
                    )
        finally:
            # Before tearDown removes the temporary root, as the other link tests do.
            self.unlink_directory(link)

    def test_main_validates_and_reports_failures(self):
        from scripts import validate_skills

        stderr = io.StringIO()
        with (
            patch.object(sys, "argv", ["validate_skills.py", "--strict"]),
            patch.object(validate_skills, "validate", return_value=["x: failure"]) as check,
            redirect_stderr(stderr),
        ):
            self.assertEqual(validate_skills.main(), 1)
        check.assert_called_once()
        self.assertTrue(check.call_args.kwargs["strict"])
        self.assertEqual(stderr.getvalue(), "x: failure\n")

        with (
            patch.object(sys, "argv", ["validate_skills.py", "--sync"]),
            patch.object(validate_skills, "sync", return_value=[]) as regenerate,
            patch.object(validate_skills, "validate", return_value=[]),
        ):
            self.assertEqual(validate_skills.main(), 0)
        regenerate.assert_called_once()

        with (
            patch.object(sys, "argv", ["validate_skills.py", "--sync"]),
            patch.object(validate_skills, "sync", return_value=["y: refused"]),
            patch.object(validate_skills, "validate") as not_run,
            redirect_stderr(io.StringIO()),
        ):
            self.assertEqual(validate_skills.main(), 1)
        not_run.assert_not_called()

    def test_reports_a_skill_file_that_is_not_utf8(self):
        path = self.root / ".claude/skills/github-repository-audit/SKILL.md"
        path.parent.mkdir(parents=True)
        for encoded in (SKILL.format(name="github-repository-audit", description="x").encode("utf-16"),
                        b"\xff\xfe\x00garbage"):
            with self.subTest(prefix=encoded[:4]):
                path.write_bytes(encoded)
                self.assertEqual(
                    validate(self.root),
                    [".claude/skills/github-repository-audit/SKILL.md: not valid UTF-8 text"],
                )

    def test_reports_missing_name(self):
        self.write(
            ".claude/skills/github-repository-audit/SKILL.md",
            "---\ndescription: Use when assessing a repository.\n---\n",
        )

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: missing name"],
        )

    def test_reports_missing_description(self):
        self.write(".claude/skills/github-repository-audit/SKILL.md", "---\nname: github-repository-audit\n---\n")

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: missing description"],
        )

    def test_reports_directory_name_mismatch(self):
        self.write(
            ".claude/skills/github-repository-audit/SKILL.md",
            SKILL.format(name="github-readme-polish", description="polishing a README"),
        )

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: name must match directory name"],
        )

    def test_requires_trigger_description(self):
        self.write(
            ".claude/skills/github-repository-audit/SKILL.md",
            "---\nname: github-repository-audit\ndescription: Assess a repository.\n---\n",
        )

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: description must start with 'Use when'"],
        )

    def test_reports_forbidden_placeholders(self):
        self.write(
            ".claude/skills/github-repository-audit/SKILL.md",
            SKILL.format(name="github-repository-audit", description="assessing a repository") + "\nTODO: finish\n",
        )

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: forbidden placeholder 'TODO'"],
        )

    def test_rejects_unexpected_skill_names(self):
        self.write(
            ".claude/skills/not-approved/SKILL.md",
            SKILL.format(name="not-approved", description="doing something"),
        )

        self.assertEqual(validate(self.root), [".claude/skills/not-approved: unexpected skill name"])

    def test_reports_mismatched_generated_copy(self):
        self.valid_skill()
        self.write(".agents/skills/github-repository-audit/SKILL.md", "different\n")

        self.assertEqual(
            validate(self.root),
            [".agents/skills/github-repository-audit/SKILL.md: differs from .claude/skills/github-repository-audit/SKILL.md"],
        )

    def test_strict_mode_reports_mismatched_generated_copy(self):
        self.valid_skill_set()
        self.write(".agents/skills/github-repository-audit/SKILL.md", "different\n")

        self.assertEqual(
            validate(self.root, strict=True),
            [".agents/skills/github-repository-audit/SKILL.md: differs from .claude/skills/github-repository-audit/SKILL.md"],
        )

    def test_incremental_mode_accepts_an_approved_subset(self):
        self.valid_skill()

        self.assertEqual(validate(self.root), [])

    def test_rejects_a_dangling_canonical_skill_link_in_both_modes(self):
        self.valid_skill_set()
        canonical = self.root / ".claude/skills"
        link = canonical / "github-repository-audit"
        (link / "SKILL.md").unlink()
        link.rmdir()
        self.link_directory(link, self.root / "missing-target")
        try:
            for strict in (False, True):
                with self.subTest(strict=strict), patch(
                    "scripts.validate_skills.filecmp.cmp", return_value=True
                ) as compare:
                    failures = validate(self.root, strict=strict)
                    self.assertEqual(
                        failures,
                        [".claude/skills/github-repository-audit: is a link, not an ordinary file or directory"],
                    )
                    if strict:
                        compare.assert_not_called()
        finally:
            self.unlink_directory(link)

    def test_rejects_a_canonical_skill_link_to_a_file_in_both_modes(self):
        self.valid_skill_set()
        canonical = self.root / ".claude/skills"
        target = self.root / "target.txt"
        target.write_text("fabricated fixture\n", encoding="utf-8")
        link = canonical / "github-repository-audit"
        (link / "SKILL.md").unlink()
        link.rmdir()
        try:
            os.symlink(target, link)
        except OSError as error:
            if os.name == "nt" and error.winerror == 1314:
                self.skipTest(f"file symlinks require an unavailable privilege: {error}")
            raise
        try:
            for strict in (False, True):
                with self.subTest(strict=strict), patch(
                    "scripts.validate_skills.filecmp.cmp", return_value=True
                ) as compare:
                    failures = validate(self.root, strict=strict)
                    self.assertEqual(
                        failures,
                        [".claude/skills/github-repository-audit: is a link, not an ordinary file or directory"],
                    )
                    if strict:
                        compare.assert_not_called()
        finally:
            link.unlink()

    def test_ordinary_files_are_not_discovered_as_skills(self):
        self.valid_skill_set()
        self.write(".claude/skills/README.md", "Skill notes.\n")
        self.write(".agents/skills/README.md", "Skill notes.\n")

        for strict in (False, True):
            with self.subTest(strict=strict):
                self.assertEqual(validate(self.root, strict=strict), [])

    def test_an_ordinary_file_cannot_satisfy_a_required_skill(self):
        self.valid_skill_set()
        name = "github-repository-audit"
        for tree in (".claude/skills", ".agents/skills"):
            path = self.root / tree / name
            (path / "SKILL.md").unlink()
            path.rmdir()
            path.write_text("Skill notes.\n", encoding="utf-8")

        self.assertEqual(validate(self.root), [])
        self.assertEqual(
            validate(self.root, strict=True),
            [f".claude/skills: missing required skill '{name}'"],
        )

    def test_incremental_mode_rejects_a_linked_canonical_skill(self):
        """The link check ran only in strict mode, so the documented incremental
        command reported success for an approved skill whose directory pointed
        outside the repository, as long as its text matched."""
        self.valid_skill()
        outside = Path(self.tmp.name) / "outside"
        (outside / "github-repository-audit").mkdir(parents=True)
        (outside / "github-repository-audit" / "SKILL.md").write_text(
            (self.root / ".claude" / "skills" / "github-repository-audit" / "SKILL.md").read_text(encoding="utf-8"),
            encoding="utf-8",
        )
        link = self.root / ".claude" / "skills" / "github-repository-audit"
        shutil.rmtree(link)
        self.link_directory(link, outside / "github-repository-audit")
        try:
            failures = validate(self.root)
        finally:
            self.unlink_directory(link)

        self.assertIn(
            ".claude/skills/github-repository-audit: is a link, not an ordinary file or directory",
            failures,
        )

    def test_incremental_mode_rejects_a_linked_generated_skill(self):
        """Only the generated SKILL.md leaf was checked, and is_file() and filecmp
        follow a linked parent, so a generated skill directory pointing outside the
        repository at a matching copy passed."""
        self.valid_skill()
        outside = Path(self.tmp.name) / "outside-generated"
        outside.mkdir()
        (outside / "SKILL.md").write_text(
            (self.root / ".claude" / "skills" / "github-repository-audit" / "SKILL.md").read_text(encoding="utf-8"),
            encoding="utf-8",
        )
        link = self.root / ".agents" / "skills" / "github-repository-audit"
        shutil.rmtree(link)
        self.link_directory(link, outside)
        try:
            failures = validate(self.root)
        finally:
            self.unlink_directory(link)

        self.assertIn(".agents/skills/github-repository-audit: is a link, not an ordinary file or directory", failures)

    def test_strict_mode_rejects_linked_runtime_directory(self):
        self.valid_skill_set()
        # The link points back at its own tree, so descending into it would never end.
        outside = self.root / ".agents" / "skills"
        link = outside / "github-repository-audit"
        shutil.rmtree(link)
        self.link_directory(link, outside)
        try:
            failures = validate(self.root, strict=True)
        finally:
            self.unlink_directory(link)

        self.assertIn(".agents/skills/github-repository-audit: is a link, not an ordinary file or directory", failures)

    def test_strict_mode_reports_both_linked_trees_before_stopping_comparison(self):
        self.valid_skill_set()
        name = "github-repository-audit"
        self.write(f".claude/skills/{name}/references/checklist.md", "canonical\n")
        linked_files = (
            (self.root / f".claude/skills/{name}/references", "checklist.md"),
            (self.root / f".agents/skills/{name}", "SKILL.md"),
        )
        links = []
        try:
            for index, (link, filename) in enumerate(linked_files):
                outside = self.root / f"outside-generated-{index}"
                outside.mkdir()
                (outside / filename).write_bytes((link / filename).read_bytes())
                (link / filename).unlink()
                link.rmdir()
                self.link_directory(link, outside)
                links.append(link)
            with patch("scripts.validate_skills.filecmp.cmp") as compare:
                failures = validate(self.root, strict=True)
        finally:
            for link in reversed(links):
                self.unlink_directory(link)

        for link, _ in linked_files:
            self.assertIn(
                f"{link.relative_to(self.root).as_posix()}: is a link, not an ordinary file or directory",
                failures,
            )
        compare.assert_not_called()

    def test_strict_mode_reports_a_linked_canonical_skill_once(self):
        self.valid_skill_set()
        name = "github-repository-audit"
        link = self.root / f".claude/skills/{name}"
        outside = self.root / "outside-canonical"
        outside.mkdir()
        (outside / "SKILL.md").write_bytes((link / "SKILL.md").read_bytes())
        (link / "SKILL.md").unlink()
        link.rmdir()
        self.link_directory(link, outside)
        try:
            with patch("scripts.validate_skills.filecmp.cmp") as compare:
                failures = validate(self.root, strict=True)
        finally:
            self.unlink_directory(link)

        message = f".claude/skills/{name}: is a link, not an ordinary file or directory"
        self.assertEqual(failures.count(message), 1)
        compare.assert_not_called()

    def test_strict_mode_requires_all_five_skills(self):
        self.valid_skill()

        self.assertEqual(
            validate(self.root, strict=True),
            [
                ".claude/skills: missing required skill 'github-issue-to-pr'",
                ".claude/skills: missing required skill 'github-profile-curator'",
                ".claude/skills: missing required skill 'github-readme-polish'",
                ".claude/skills: missing required skill 'github-release-prep'",
            ],
        )

    def test_strict_mode_reports_missing_runtime_root(self):
        self.valid_skill_set()
        shutil.rmtree(self.root / ".agents")
        failures = validate(self.root, strict=True)
        self.assertTrue(any("missing generated" in failure for failure in failures))

    def test_strict_mode_requires_every_canonical_file_in_each_runtime(self):
        self.valid_skill_set()
        self.write(".claude/skills/github-repository-audit/references/checklist.md", "canonical\n")

        self.assertEqual(
            validate(self.root, strict=True),
            [
                ".agents/skills/github-repository-audit/references: missing generated directory",
                ".agents/skills/github-repository-audit/references/checklist.md: missing generated copy",
            ],
        )

    def test_strict_mode_rejects_unexpected_runtime_files_and_directories(self):
        self.valid_skill_set()
        self.write(".agents/skills/github-repository-audit/extra.md", "extra\n")
        self.write(".agents/skills/not-approved/SKILL.md", "extra\n")

        self.assertEqual(
            validate(self.root, strict=True),
            [
                ".agents/skills/github-repository-audit/extra.md: unexpected generated file",
                ".agents/skills/not-approved/SKILL.md: unexpected generated file",
                ".agents/skills/not-approved: unexpected generated directory",
            ],
        )


class RecordedRunDigestTests(unittest.TestCase):
    SKILL_NAME = "github-repository-audit"
    RUN = "validation/forward/github-repository-audit.txt"

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.skill = self.root / ".claude/skills" / self.SKILL_NAME
        self.write(self.skill / "SKILL.md", b"---\nname: x\n---\n")
        self.write(self.skill / "references/checklist.md", b"check\n")

    def tearDown(self):
        self.tmp.cleanup()

    @staticmethod
    def write(path, data):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(data)

    def record(self, digest_lines, relative=RUN):
        self.write(self.root / relative, f"Scenario: x\n\nAudit metadata:\n{digest_lines}- Run exit status: 0.\n".encode())

    def record_current(self, relative=RUN):
        self.record(f"- Skill digest v1:\n  `{skill_digest(self.skill)}`.\n", relative)

    def test_digest_hashes_a_sorted_manifest_of_every_file_in_the_skill(self):
        def sha(data):
            return hashlib.sha256(data).hexdigest()

        self.write(self.skill / "scripts/run.py", b"print(1)\r\n")
        manifest = (
            f"{sha(b'---\nname: x\n---\n')}  SKILL.md\n"
            f"{sha(b'check\n')}  references/checklist.md\n"
            f"{sha(b'print(1)\r\n')}  scripts/run.py\n"
        )

        self.assertEqual(skill_digest(self.skill), sha(manifest.encode()))

    def test_matching_digests_pass(self):
        self.record_current()
        self.record_current(f"validation/baselines/{self.SKILL_NAME}.txt")

        self.assertEqual(check_runs(self.root), ([], []))

    def test_a_digest_on_one_line_is_accepted(self):
        self.record(f"- Skill digest v1: `{skill_digest(self.skill)}`.\n")

        self.assertEqual(check_runs(self.root), ([], []))

    def test_a_changed_skill_md_makes_the_run_stale(self):
        self.record_current()
        recorded = skill_digest(self.skill)
        self.write(self.skill / "SKILL.md", b"---\nname: y\n---\n")

        self.assertEqual(
            check_runs(self.root),
            (
                [
                    f"{self.RUN}: stale; recorded skill digest {recorded} differs from "
                    f".claude/skills/{self.SKILL_NAME} digest {skill_digest(self.skill)}"
                ],
                [],
            ),
        )

    def test_a_changed_or_added_asset_makes_the_run_stale(self):
        for change in ("references/checklist.md", "scripts/new.py"):
            with self.subTest(change=change):
                self.record_current()
                self.write(self.skill / change, b"changed\n")

                failures, notices = check_runs(self.root)

                self.assertEqual(len(failures), 1)
                self.assertIn(f"{self.RUN}: stale;", failures[0])
                self.assertEqual(notices, [])

    def test_a_crlf_conversion_makes_the_run_stale(self):
        self.record_current()
        self.write(self.skill / "references/checklist.md", b"check\r\n")

        self.assertEqual(len(check_runs(self.root)[0]), 1)

    def test_a_run_without_a_digest_is_a_notice_not_a_failure(self):
        self.record("")

        self.assertEqual(
            check_runs(self.root),
            ([], [f"{self.RUN}: no skill digest recorded; its results are not tied to a skill revision"]),
        )

    def test_a_malformed_or_repeated_digest_fails(self):
        digest = skill_digest(self.skill)
        for lines in (
            f"- Skill digest v2:\n  `{digest}`.\n",
            f"- Skill digest v1:\n  `{digest[:-1]}`.\n",
            f"- Skill digest v1: `{digest}`.\n- Skill digest v1: `{digest}`.\n",
        ):
            with self.subTest(lines=lines):
                self.record(lines)

                self.assertEqual(
                    check_runs(self.root),
                    ([f"{self.RUN}: malformed skill digest; record exactly one '- Skill digest v1:' line"], []),
                )

    def test_a_run_named_after_no_approved_skill_fails(self):
        self.record("", "validation/baselines/unknown-skill.txt")

        self.assertEqual(check_runs(self.root), (["validation/baselines/unknown-skill.txt: names no approved skill"], []))

    def test_main_strict_prints_notices_and_fails_on_stale_runs(self):
        from scripts import validate_skills

        for result, code in (([], ["r: no skill digest recorded"]), 0), ((["r: stale"], []), 1):
            with self.subTest(result=result):
                stdout, stderr = io.StringIO(), io.StringIO()
                with (
                    patch.object(sys, "argv", ["validate_skills.py", "--strict"]),
                    patch.object(validate_skills, "validate", return_value=[]),
                    patch.object(validate_skills, "check_runs", return_value=result),
                    redirect_stdout(stdout),
                    redirect_stderr(stderr),
                ):
                    self.assertEqual(validate_skills.main(), code)
                self.assertEqual(stdout.getvalue(), "".join(f"{line}\n" for line in result[1]))
                self.assertEqual(stderr.getvalue(), "".join(f"{line}\n" for line in result[0]))

    def test_main_prints_the_digest_line_after_strict_validation(self):
        from scripts import validate_skills

        stdout = io.StringIO()
        with (
            patch.object(sys, "argv", ["validate_skills.py", "--skill-digest", self.SKILL_NAME]),
            patch.object(validate_skills, "validate", return_value=[]) as check,
            patch.object(validate_skills, "check_runs") as runs_not_checked,
            patch.object(validate_skills, "skill_digest", return_value="0" * 64) as digest,
            redirect_stdout(stdout),
        ):
            self.assertEqual(validate_skills.main(), 0)
        self.assertTrue(check.call_args.kwargs["strict"])
        runs_not_checked.assert_not_called()
        self.assertEqual(digest.call_args.args[0].as_posix().split("/")[-3:], [".claude", "skills", self.SKILL_NAME])
        self.assertEqual(stdout.getvalue(), f"- Skill digest v1:\n  `{'0' * 64}`.\n")


class ScenarioCoverageTests(unittest.TestCase):
    def test_every_canonical_skill_has_one_scenario_of_the_same_name(self):
        skills = {path.name for path in (ROOT / ".claude" / "skills").iterdir() if path.is_dir()}
        scenarios = {path.stem for path in (ROOT / "tests" / "scenarios").glob("*.md")}

        self.assertEqual(scenarios, skills)


if __name__ == "__main__":
    unittest.main()
