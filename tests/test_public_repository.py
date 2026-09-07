import subprocess
import tempfile
import unittest
from pathlib import Path

from scripts import check_public_files as public_files


ROOT = Path(__file__).parents[1]


class PublicRepositoryTests(unittest.TestCase):
    def test_quick_start_commands_are_semantically_traceable(self):
        readme = (ROOT / "README.md").read_text(encoding="utf-8")

        self.assertEqual(public_files.quick_start_failures(readme), [])

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
                self.assertTrue(public_files.quick_start_failures(changed_readme))

    def test_tracked_text_has_no_public_safety_failures(self):
        self.assertEqual(public_files.tracked_failures(ROOT), [])

    def test_tracked_task_reports_are_scanned(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(["git", "init", "-q"], cwd=root, check=True)
            report = root / ".superpowers" / "sdd" / "run" / "task-4-report.md"
            report.parent.mkdir(parents=True)
            report.write_bytes(b"auth" + b"_token" + b" = " + b"do-not-echo\n")
            subprocess.run(["git", "add", str(report.relative_to(root))], cwd=root, check=True)

            failures = public_files.tracked_failures(root)

            self.assertEqual(failures, [f"{report.relative_to(root).as_posix()}: credential-assignment"])
            self.assertNotIn("do-not-echo", "\n".join(failures))


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

    def test_allows_public_github_url_and_rejects_unsafe_content_without_echoing_it(self):
        public_url = self.write(
            "public-source.md",
            b"https://api.github.com/users/ryanduguid/repos?per_page=100&type=all&sort=full_name&direction=asc\n",
        )
        self.assertEqual(public_files.scan_paths(self.root, [public_url]), [])

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
            ("p", "notes.json", ("{\"api" + "_key\"" + ":\"do-not-echo\"}").encode(), "credential-assignment"),
            ("q", "notes.json", ("{\"to" + "ken\"" + ":\"do-not-echo\"}").encode(), "credential-assignment"),
            ("r", "notes.toml", ("\"access" + "_key\"" + " = \"do-not-echo\"").encode(), "credential-assignment"),
            ("s", "notes.yaml", ("'client" + "_secret'" + ": 'do-not-echo'").encode(), "credential-assignment"),
            ("t", "notes.txt", ("C:" + "/" + "Users/Pat/notes").encode(), "private-user-path"),
            ("u", "notes.txt", ("file:" + "///" + "Users/pat/notes").encode(), "private-user-path"),
            ("v", "notes.txt", ("https://example.test/download?path=" + "/" + "home/pat/notes").encode(), "private-user-path"),
            ("unc", "notes.txt", ("\\\\server\\Users\\Pat\\notes").encode(), "private-user-path"),
            (
                "w",
                "notes.txt",
                (
                    "https://api.github.com/users/ryanduguid/repos\nLocal copy: "
                    + "C:"
                    + "\\Users\\Pat\\notes"
                ).encode(),
                "private-user-path",
            ),
            (
                "unc-mixed",
                "notes.txt",
                (
                    "https://api.github.com/users/ryanduguid/repos\nLocal copy: "
                    + "\\\\server\\Users\\Pat\\notes"
                ).encode(),
                "private-user-path",
            ),
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

    def test_allows_credential_key_policy_prose_without_an_assignment(self):
        path = self.write(
            "policy.md",
            ("Reject quoted credential keys such as \"api" + "_key\" or \"to" + "ken\" when they are assigned values.\n").encode(),
        )

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
        self.assertIn(
            "actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1",
            workflow,
        )
        self.assertIn(
            "actions/setup-python@5fda3b95a4ea91299a34e894583c3862153e4b97",
            workflow,
        )
        self.assertIn("python-version: '3.11'", workflow)
        for command in (
            "python -m unittest discover -s tests -v",
            "python scripts/validate_skills.py --strict",
            "python scripts/check_public_files.py",
        ):
            self.assertIn(command, workflow)

    def test_workflow_runs_junction_safety_tests_on_windows(self):
        workflow = (ROOT / ".github/workflows/validate.yml").read_text(encoding="utf-8")

        self.assertIn("runs-on: windows-latest", workflow)
        self.assertIn(
            "python -m unittest tests.test_sync_skills.SyncSkillsTests.test_sync_rejects_linked_approved_child_before_any_mutation "
            "tests.test_sync_skills.SyncSkillsTests.test_sync_rejects_nested_linked_destination_descendant "
            "tests.test_sync_skills.SyncSkillsTests.test_sync_rejects_nested_canonical_link_before_copy -v",
            workflow,
        )


if __name__ == "__main__":
    unittest.main()
