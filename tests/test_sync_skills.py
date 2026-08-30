import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "scripts" / "sync-skills.ps1"
NAME = "github-repository-audit"
NAMES = (
    "github-issue-to-pr",
    "github-profile-curator",
    "github-readme-polish",
    "github-release-prep",
    "github-repository-audit",
)


class SyncSkillsTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.outside = tempfile.TemporaryDirectory()
        self.links = []
        self.root = Path(self.tmp.name)
        script = self.root / "scripts" / "sync-skills.ps1"
        script.parent.mkdir()
        shutil.copyfile(SCRIPT, script)
        source = self.root / "skills" / NAME / "SKILL.md"
        source.parent.mkdir(parents=True)
        source.write_text("canonical\n", encoding="utf-8")
        self.run_sync()

    def tearDown(self):
        for link in reversed(self.links):
            if os.path.lexists(link):
                if os.name == "nt":
                    subprocess.run(["cmd", "/c", "rmdir", str(link)], capture_output=True, check=False)
                else:
                    os.unlink(link)
        self.outside.cleanup()
        self.tmp.cleanup()

    def link_directory(self, target, link):
        target = Path(target)
        link = Path(link)
        target.mkdir(parents=True, exist_ok=True)
        link.parent.mkdir(parents=True, exist_ok=True)
        if os.name == "nt":
            result = subprocess.run(
                ["cmd", "/c", "mklink", "/J", str(link), str(target)],
                capture_output=True,
                text=True,
                check=False,
            )
            if result.returncode:
                self.skipTest(f"NTFS junctions are unavailable: {result.stderr}")
        else:
            os.symlink(target, link, target_is_directory=True)
        self.links.append(link)

    def run_sync(self, *args):
        return subprocess.run(
            ["pwsh", "-File", str(self.root / "scripts" / "sync-skills.ps1"), *args],
            capture_output=True,
            text=True,
            check=False,
        )

    def test_check_rejects_changed_generated_file(self):
        (self.root / ".agents" / "skills" / NAME / "SKILL.md").write_text("changed\n", encoding="utf-8")

        result = self.run_sync("-Check")

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            (self.root / ".agents" / "skills" / NAME / "SKILL.md").read_text(encoding="utf-8"),
            "changed\n",
        )

    def test_check_accepts_empty_canonical_set(self):
        shutil.rmtree(self.root / "skills")
        shutil.rmtree(self.root / ".agents")
        shutil.rmtree(self.root / ".claude")

        result = self.run_sync("-Check")

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_check_rejects_unexpected_generated_directory(self):
        unexpected = self.root / ".claude" / "skills" / "not-approved" / "SKILL.md"
        unexpected.parent.mkdir(parents=True)
        unexpected.write_text("extra\n", encoding="utf-8")

        result = self.run_sync("-Check")

        self.assertNotEqual(result.returncode, 0, result.stderr)

    def test_check_rejects_unexpected_generated_file(self):
        unexpected = self.root / ".claude" / "skills" / NAME / "extra.md"
        unexpected.write_text("extra\n", encoding="utf-8")

        result = self.run_sync("-Check")

        self.assertNotEqual(result.returncode, 0, result.stderr)

    def test_sync_creates_exact_five_byte_identical_skill_trees(self):
        shutil.rmtree(self.root / "skills")
        for name in NAMES:
            skill = self.root / "skills" / name
            (skill / "SKILL.md").parent.mkdir(parents=True)
            (skill / "SKILL.md").write_bytes(f"{name}\n".encode())
            (skill / "references" / "check.bin").parent.mkdir()
            (skill / "references" / "check.bin").write_bytes(b"\x00\xff\x10")

        result = self.run_sync()

        self.assertEqual(result.returncode, 0, result.stderr)
        for runtime in (".agents/skills", ".claude/skills"):
            destination = self.root / runtime
            self.assertEqual({path.name for path in destination.iterdir()}, set(NAMES))
            for source in (self.root / "skills").rglob("*"):
                if source.is_file():
                    self.assertEqual(
                        (destination / source.relative_to(self.root / "skills")).read_bytes(),
                        source.read_bytes(),
                    )

    def test_sync_rejects_symlinked_destination_before_mutating_external_files(self):
        shutil.rmtree(self.root / ".agents")
        outside_skill = Path(self.outside.name) / NAME / "SKILL.md"
        outside_skill.parent.mkdir()
        outside_skill.write_text("outside\n", encoding="utf-8")
        agents = self.root / ".agents"
        agents.mkdir()
        try:
            os.symlink(self.outside.name, agents / "skills", target_is_directory=True)
        except OSError as error:
            junction = subprocess.run(
                ["cmd", "/c", "mklink", "/J", str(agents / "skills"), self.outside.name],
                capture_output=True,
                text=True,
                check=False,
            )
            if junction.returncode:
                self.skipTest(f"directory links are unavailable: {error}; {junction.stderr}")

        result = self.run_sync()

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(outside_skill.read_text(encoding="utf-8"), "outside\n")

    def test_sync_rejects_linked_approved_child_before_any_mutation(self):
        linked_child = self.root / ".claude" / "skills" / NAME
        shutil.rmtree(linked_child)
        outside_child = Path(self.outside.name) / "linked-child"
        sentinel = outside_child / "sentinel.txt"
        sentinel.parent.mkdir(parents=True)
        sentinel.write_text("outside\n", encoding="utf-8")
        self.link_directory(outside_child, linked_child)
        agents_copy = self.root / ".agents" / "skills" / NAME / "SKILL.md"
        agents_copy.write_text("agents-before\n", encoding="utf-8")

        result = self.run_sync()

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(sentinel.read_text(encoding="utf-8"), "outside\n")
        self.assertEqual(agents_copy.read_text(encoding="utf-8"), "agents-before\n")

    def test_sync_rejects_nested_linked_destination_descendant(self):
        linked_descendant = self.root / ".agents" / "skills" / NAME / "references" / "linked"
        outside_descendant = Path(self.outside.name) / "nested-destination"
        sentinel = outside_descendant / "sentinel.txt"
        sentinel.parent.mkdir(parents=True)
        sentinel.write_text("outside\n", encoding="utf-8")
        self.link_directory(outside_descendant, linked_descendant)
        claude_copy = self.root / ".claude" / "skills" / NAME / "SKILL.md"
        claude_copy.write_text("claude-before\n", encoding="utf-8")

        result = self.run_sync()

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(sentinel.read_text(encoding="utf-8"), "outside\n")
        self.assertEqual(claude_copy.read_text(encoding="utf-8"), "claude-before\n")

    def test_sync_rejects_nested_canonical_link_before_copy(self):
        canonical_link = self.root / "skills" / NAME / "references" / "linked"
        outside_source = Path(self.outside.name) / "nested-canonical"
        sentinel = outside_source / "sentinel.txt"
        sentinel.parent.mkdir(parents=True)
        sentinel.write_text("outside\n", encoding="utf-8")
        self.link_directory(outside_source, canonical_link)
        agents_copy = self.root / ".agents" / "skills" / NAME / "SKILL.md"
        agents_copy.write_text("agents-before\n", encoding="utf-8")

        result = self.run_sync()

        self.assertNotEqual(result.returncode, 0, result.stderr)
        self.assertEqual(sentinel.read_text(encoding="utf-8"), "outside\n")
        self.assertEqual(agents_copy.read_text(encoding="utf-8"), "agents-before\n")


if __name__ == "__main__":
    unittest.main()
