import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[1] / "scripts" / "sync-skills.ps1"
NAME = "github-repository-audit"


class SyncSkillsTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        script = self.root / "scripts" / "sync-skills.ps1"
        script.parent.mkdir()
        shutil.copyfile(SCRIPT, script)
        source = self.root / "skills" / NAME / "SKILL.md"
        source.parent.mkdir(parents=True)
        source.write_text("canonical\n", encoding="utf-8")
        self.run_sync()

    def tearDown(self):
        self.tmp.cleanup()

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


if __name__ == "__main__":
    unittest.main()
