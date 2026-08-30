import tempfile
import unittest
from pathlib import Path

from scripts import check_public_files as public_files


ROOT = Path(__file__).parents[1]


class PublicRepositoryTests(unittest.TestCase):
    def test_public_file_scanner_exists(self):
        self.assertTrue((ROOT / "scripts/check_public_files.py").is_file())

    def test_quick_start_commands_are_semantically_traceable(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        check = getattr(public_files, "quick_start_failures", lambda *_: ["missing checker"])

        self.assertEqual(check(ROOT, readme), [])

    def test_quick_start_rejects_invalid_command_targets(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        check = getattr(public_files, "quick_start_failures", lambda *_: [])
        mutations = (
            (
                "git clone https://github.com/ryanduguid/github-agent-skills.git",
                "git clone https://github.com/ryanduguid/missing-skills.git",
            ),
            ("cd github-agent-skills", "cd missing-skills"),
            ("pwsh -File scripts/sync-skills.ps1", "pwsh -File scripts/missing.ps1"),
            ("-s tests -v", "-s missing-tests -v"),
            ("python scripts/validate_skills.py --strict", "python scripts/missing.py --strict"),
        )

        for source, replacement in mutations:
            with self.subTest(replacement=replacement):
                self.assertTrue(check(ROOT, readme.replace(source, replacement)))

    def test_quick_start_accepts_an_existing_python_script_target(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        check = public_files.quick_start_failures

        self.assertEqual(
            check(ROOT, readme.replace("python scripts/validate_skills.py --strict", "python scripts/check_public_files.py")),
            [],
        )

    def test_tracked_text_has_no_public_safety_failures(self):
        self.assertEqual(public_files.tracked_failures(ROOT), [])


class PublicFileScannerTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, relative, content):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return Path(relative)

    def test_rejects_unsafe_content_without_echoing_it(self):
        checks = {
            "transcript": ("notes/transcript.txt", b"transcripts" + b"/session-42.txt\n", "transcript-path"),
            "raw-sdd": ("notes.txt", b".superpowers/sdd/run" + b"/raw/output.txt\n", "raw-sdd-path"),
            "windows": ("notes.txt", ("C:" + "\\Users\\Pat\\notes").encode(), "private-user-path"),
            "posix": ("notes.txt", ("/" + "home/pat/notes").encode(), "private-user-path"),
            "macos": ("notes.txt", ("/" + "Users/pat/notes").encode(), "private-user-path"),
            "credential": ("notes.txt", ("api" + "_key = " + "do-not-echo").encode(), "credential-assignment"),
            "client": ("notes.txt", ("client" + "_data: " + "do-not-echo").encode(), "client-data"),
        }

        for name, (relative, content, rule) in checks.items():
            with self.subTest(name=name):
                path = self.write(relative, content)
                failures = public_files.scan_paths(self.root, [path])
                self.assertEqual(failures, [f"{path.as_posix()}: {rule}"])
                self.assertNotIn("do-not-echo", "\n".join(failures))

    def test_rejects_private_configuration_paths(self):
        path = self.write(".env", b"safe placeholder\n")

        self.assertEqual(public_files.scan_paths(self.root, [path]), [".env: private-config"])

    def test_rejects_undecodable_text(self):
        path = self.write("notes.txt", b"note=\xffvalue\n")

        self.assertEqual(public_files.scan_paths(self.root, [path]), ["notes.txt: undecodable-text"])


class WorkflowTests(unittest.TestCase):
    def test_workflow_has_read_only_permissions_and_required_checks(self):
        workflow = (ROOT / ".github/workflows/validate.yml").read_text(encoding="utf-8")
        permissions = public_files.permissions_block(workflow)
        self.assertEqual(permissions, ["  contents: read"])
        self.assertIn("actions/checkout@v7", workflow)
        self.assertIn("actions/setup-python@v7", workflow)
        self.assertIn("python-version: '3.11'", workflow)
        for command in (
            "python -m unittest discover -s tests -v",
            "python scripts/validate_skills.py",
            "python scripts/validate_skills.py --strict",
            "python scripts/check_public_files.py",
            "pwsh -File scripts/sync-skills.ps1 -Check",
        ):
            self.assertIn(command, workflow)


if __name__ == "__main__":
    unittest.main()
