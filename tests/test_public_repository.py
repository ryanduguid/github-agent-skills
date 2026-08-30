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

    def test_quick_start_rejects_any_command_list_change(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")
        commands = public_files.quick_start_commands(readme)
        mutations = {
            "wrong clone": ["git clone https://github.com/ryanduguid/missing-skills.git", *commands[1:]],
            "wrong directory": [commands[0], "cd missing-skills", *commands[2:]],
            "extra operand": [*commands[:4], commands[4] + " --input missing.json", *commands[5:]],
            "omitted line": commands[:4] + commands[5:],
            "reordered line": [commands[1], commands[0], *commands[2:]],
            "unknown command": [*commands, "python scripts/missing.py"],
        }

        for name, changed in mutations.items():
            with self.subTest(name=name):
                changed_readme = readme.replace("\n".join(commands), "\n".join(changed))
                self.assertTrue(public_files.quick_start_failures(ROOT, changed_readme))

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
        checks = (
            ("a", "notes/transcript.txt", b"transcripts" + b"/session-42.txt\n", "transcript-path"),
            ("b", "notes.txt", b".superpowers/sdd/run" + b"/raw/output.txt\n", "raw-sdd-path"),
            ("c", "notes.txt", ("C:" + "\\Users\\Pat\\notes").encode(), "private-user-path"),
            ("d", "notes.txt", ("/" + "home/pat/notes").encode(), "private-user-path"),
            ("e", "notes.txt", ("/" + "Users/pat/notes").encode(), "private-user-path"),
            ("f", "notes.txt", ("db" + "_password" + " = " + "do-not-echo").encode(), "credential-assignment"),
            ("g", "notes.txt", ("deploy" + "_secret" + " = " + "do-not-echo").encode(), "credential-assignment"),
            ("h", "notes.txt", ("auth" + "_token" + " = " + "do-not-echo").encode(), "credential-assignment"),
            ("i", "notes.txt", ("api" + "_key" + " = " + "do-not-echo").encode(), "credential-assignment"),
            ("j", "notes.txt", ("access" + "_key" + " = " + "do-not-echo").encode(), "credential-assignment"),
            ("k", "notes.txt", ("client" + "_secret" + " = " + "do-not-echo").encode(), "credential-assignment"),
            ("l", "notes.txt", ("private" + "_key" + " = " + "do-not-echo").encode(), "credential-assignment"),
            ("m", "notes.txt", ("client" + "_data" + ": " + "do-not-echo").encode(), "client-data"),
            ("n", "notes.txt", ("stripe" + "_key" + " = " + "do-not-echo").encode(), "credential-assignment"),
            ("o", "notes.txt", ("customer" + "_key" + " = " + "do-not-echo").encode(), "credential-assignment"),
        )

        for name, relative, content, rule in checks:
            with self.subTest(name=name):
                path = self.write(relative, content)
                failures = public_files.scan_paths(self.root, [path])
                self.assertEqual(failures, [f"{path.as_posix()}: {rule}"])
                self.assertNotIn("do-not-echo", "\n".join(failures))

    def test_rejects_private_configuration_paths(self):
        paths = [
            self.write(relative, b"safe placeholder\n")
            for relative in (".env", ".netrc", ".npmrc", "credentials.json", ".ssh/config", ".aws/credentials", "home/pat/.config/settings")
        ]

        self.assertEqual(
            public_files.scan_paths(self.root, paths),
            [f"{path.as_posix()}: private-config" for path in paths],
        )

    def test_rejects_client_customer_data_paths_but_not_policy_prose(self):
        paths = [self.write("client-data.csv", b"safe\n"), self.write("exports/customer_records.csv", b"safe\n")]
        policy = self.write("policy.md", b"Do not submit customer data or client records.\n")

        self.assertEqual(
            public_files.scan_paths(self.root, paths),
            [f"{path.as_posix()}: client-data" for path in paths],
        )
        self.assertEqual(public_files.scan_paths(self.root, [policy]), [])

    def test_allows_public_key_policy_example(self):
        path = self.write("policy.md", ("public" + "_key" + " = " + "example-value").encode())

        self.assertEqual(public_files.scan_paths(self.root, [path]), [])

    def test_rejects_undecodable_text(self):
        path = self.write("notes.txt", b"note=\xffvalue\n")

        self.assertEqual(public_files.scan_paths(self.root, [path]), ["notes.txt: undecodable-text"])

    def test_scans_bom_marked_utf16_and_flags_unclassifiable_nul_text(self):
        utf16 = self.write("utf16.txt", ("api" + "_key" + " = " + "do-not-echo").encode("utf-16"))
        nul_text = self.write("embedded-null.txt", b"note=\0value\n")

        self.assertEqual(public_files.scan_paths(self.root, [utf16]), ["utf16.txt: credential-assignment"])
        self.assertEqual(public_files.scan_paths(self.root, [nul_text]), ["embedded-null.txt: unclassifiable-text"])


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
