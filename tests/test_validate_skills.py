import tempfile
import unittest
from pathlib import Path

from scripts.validate_skills import validate


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
        self.write(Path(".claude/skills") / name / "SKILL.md", text)

    def valid_skill(self, name="github-repository-audit"):
        text = SKILL.format(name=name, description="assessing a repository")
        self.write(Path("skills") / name / "SKILL.md", text)
        self.copies(name, text)

    def valid_skill_set(self):
        for name in (
            "github-issue-to-pr",
            "github-profile-curator",
            "github-readme-polish",
            "github-release-prep",
            "github-repository-audit",
        ):
            text = SKILL.format(name=name, description="assessing a repository")
            self.write(Path("skills") / name / "SKILL.md", text)
            self.copies(name, text)

    def test_reports_missing_frontmatter_delimiters(self):
        self.write("skills/github-repository-audit/SKILL.md", "name: github-repository-audit\n")

        self.assertEqual(
            validate(self.root),
            ["skills/github-repository-audit/SKILL.md: missing YAML frontmatter delimiters"],
        )

    def test_reports_missing_name(self):
        self.write(
            "skills/github-repository-audit/SKILL.md",
            "---\ndescription: Use when assessing a repository.\n---\n",
        )

        self.assertEqual(
            validate(self.root),
            ["skills/github-repository-audit/SKILL.md: missing name"],
        )

    def test_reports_missing_description(self):
        self.write("skills/github-repository-audit/SKILL.md", "---\nname: github-repository-audit\n---\n")

        self.assertEqual(
            validate(self.root),
            ["skills/github-repository-audit/SKILL.md: missing description"],
        )

    def test_reports_directory_name_mismatch(self):
        self.write(
            "skills/github-repository-audit/SKILL.md",
            SKILL.format(name="github-readme-polish", description="polishing a README"),
        )

        self.assertEqual(
            validate(self.root),
            ["skills/github-repository-audit/SKILL.md: name must match directory name"],
        )

    def test_requires_trigger_description(self):
        self.write(
            "skills/github-repository-audit/SKILL.md",
            "---\nname: github-repository-audit\ndescription: Assess a repository.\n---\n",
        )

        self.assertEqual(
            validate(self.root),
            ["skills/github-repository-audit/SKILL.md: description must start with 'Use when'"],
        )

    def test_reports_forbidden_placeholders(self):
        self.write(
            "skills/github-repository-audit/SKILL.md",
            SKILL.format(name="github-repository-audit", description="assessing a repository") + "\nTODO: finish\n",
        )

        self.assertEqual(
            validate(self.root),
            ["skills/github-repository-audit/SKILL.md: forbidden placeholder 'TODO'"],
        )

    def test_rejects_unexpected_skill_names(self):
        self.write(
            "skills/not-approved/SKILL.md",
            SKILL.format(name="not-approved", description="doing something"),
        )

        self.assertEqual(validate(self.root), ["skills/not-approved: unexpected skill name"])

    def test_reports_mismatched_generated_copy(self):
        self.valid_skill()
        self.write(".claude/skills/github-repository-audit/SKILL.md", "different\n")

        self.assertEqual(
            validate(self.root),
            [".claude/skills/github-repository-audit/SKILL.md: differs from skills/github-repository-audit/SKILL.md"],
        )

    def test_incremental_mode_accepts_an_approved_subset(self):
        self.valid_skill()

        self.assertEqual(validate(self.root), [])

    def test_strict_mode_requires_all_five_skills(self):
        self.valid_skill()

        self.assertEqual(
            validate(self.root, strict=True),
            [
                "skills: missing required skill 'github-issue-to-pr'",
                "skills: missing required skill 'github-profile-curator'",
                "skills: missing required skill 'github-readme-polish'",
                "skills: missing required skill 'github-release-prep'",
            ],
        )

    def test_strict_mode_requires_every_canonical_file_in_each_runtime(self):
        self.valid_skill_set()
        self.write("skills/github-repository-audit/references/checklist.md", "canonical\n")
        self.write(".agents/skills/github-repository-audit/references/checklist.md", "canonical\n")

        self.assertEqual(
            validate(self.root, strict=True),
            [
                ".claude/skills/github-repository-audit/references: missing generated directory",
                ".claude/skills/github-repository-audit/references/checklist.md: missing generated copy",
            ],
        )

    def test_strict_mode_rejects_unexpected_runtime_files_and_directories(self):
        self.valid_skill_set()
        self.write(".agents/skills/github-repository-audit/extra.md", "extra\n")
        self.write(".claude/skills/not-approved/SKILL.md", "extra\n")

        self.assertEqual(
            validate(self.root, strict=True),
            [
                ".agents/skills/github-repository-audit/extra.md: unexpected generated file",
                ".claude/skills/not-approved/SKILL.md: unexpected generated file",
                ".claude/skills/not-approved: unexpected generated directory",
            ],
        )


if __name__ == "__main__":
    unittest.main()
