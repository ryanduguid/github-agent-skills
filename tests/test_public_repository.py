import re
import subprocess
import unittest
from pathlib import Path


ROOT = Path(__file__).parents[1]
QUICK_START = re.compile(r"^## Quick start\s*$([\s\S]*?)(?=^## |\Z)", re.MULTILINE)
FENCED_COMMANDS = re.compile(r"```(?:powershell|shell)\n([\s\S]*?)```")
CREDENTIAL_PATTERNS = (
    re.compile(r"\b(?:ghp|github_pat)_[A-Za-z0-9_]{20,}\b", re.IGNORECASE),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
)
PRIVATE_PATH_PATTERNS = (
    re.compile(r"C:[\\/]Users[\\/]"),
)


class PublicRepositoryTests(unittest.TestCase):
    def tracked_paths(self):
        result = subprocess.run(
            ["git", "ls-files", "-z"], cwd=ROOT, capture_output=True, check=True
        )
        return [Path(path) for path in result.stdout.decode().split("\0") if path]

    def test_quick_start_commands_reference_existing_repository_files(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        quick_start = QUICK_START.search(readme)
        self.assertIsNotNone(quick_start, "README needs a Quick start section")
        commands = FENCED_COMMANDS.findall(quick_start.group(1))
        self.assertTrue(commands, "Quick start needs a PowerShell or shell command block")
        for command in "\n".join(commands).splitlines():
            parts = command.split()
            if parts[:2] == ["python", "scripts/validate_skills.py"]:
                self.assertTrue((ROOT / parts[1]).is_file(), command)
            if parts[:3] == ["pwsh", "-File", "scripts/sync-skills.ps1"]:
                self.assertTrue((ROOT / parts[2]).is_file(), command)

    def test_tracked_text_has_no_credentials_or_private_evidence_paths(self):
        failures = []
        for path in self.tracked_paths():
            candidate = ROOT / path
            try:
                text = candidate.read_text(encoding="utf-8")
            except UnicodeDecodeError:
                continue
            if (
                ".superpowers/sdd/" in path.as_posix() and "/raw/" in path.as_posix()
            ) or any(pattern.search(text) for pattern in CREDENTIAL_PATTERNS + PRIVATE_PATH_PATTERNS):
                failures.append(path.as_posix())
        self.assertEqual(failures, [], "unsafe public content: " + ", ".join(failures))

    def test_workflow_has_read_only_permissions_and_required_checks(self):
        workflow = (ROOT / ".github/workflows/validate.yml").read_text(encoding="utf-8")
        permissions = re.search(r"^permissions:\n((?:^[ ]{2}.*\n)+)", workflow, re.MULTILINE)
        self.assertIsNotNone(permissions, "workflow needs explicit permissions")
        self.assertEqual(permissions.group(1).splitlines(), ["  contents: read"])
        self.assertIn("actions/checkout@v7", workflow)
        self.assertIn("actions/setup-python@v7", workflow)
        self.assertIn("python-version: '3.11'", workflow)
        for command in (
            "python -m unittest discover -s tests -v",
            "python scripts/validate_skills.py",
            "python scripts/validate_skills.py --strict",
            "pwsh -File scripts/sync-skills.ps1 -Check",
        ):
            self.assertIn(command, workflow)


if __name__ == "__main__":
    unittest.main()
