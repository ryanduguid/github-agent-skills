"""Regression coverage for aggregate workflow predecessor selection."""

import importlib.util
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).parents[1]
SPEC = importlib.util.spec_from_file_location("check_gates", ROOT / ".github/ci/check_gates.py")
CHECK_GATES = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK_GATES)
NEEDS = "    needs: [lint, validate, windows-sync-safety, coverage]\n"
BASE = (
    "name: Validate\n\non: pull_request\n\njobs:\n  validate-gates:\n" + NEEDS
    + "    runs-on: ubuntu-latest\n    steps: [{run: 'true'}]\n"
    + "".join(f"  {job}:\n    runs-on: ubuntu-latest\n    steps: [{{run: 'true'}}]\n"
              for job in ("lint", "validate", "windows-sync-safety", "coverage", "codecov", "fuzzing"))
)


class GatePredecessorTests(unittest.TestCase):
    def head_workflow(self, gate, name, declaration=""):
        head = (f"name: Validate\n\non: pull_request\n\njobs:\n  {gate}:\n"
                f"    name: {name}\n    runs-on: ubuntu-latest\n"
                "    steps: [{run: 'true'}]\n")
        if declaration:
            head = f"# removed-jobs: {declaration}\n" + head
        return head

    def run_gate(self, gate, name, declaration="", *, sources=None, head=None):
        if sources is None:
            sources = {".github/workflows/validate.yml": BASE}
        if head is None:
            head = self.head_workflow(gate, name, declaration)

        def git(*args):
            if args == ("ls-tree", "--name-only", "HEAD^1", "--", ".github/workflows/renamed.yml"):
                return "renamed.yml" if ".github/workflows/renamed.yml" in sources else ""
            if len(args) == 2 and args[0] == "show" and args[1].startswith("HEAD^1:"):
                return sources[args[1].split(":", 1)[1]]
            raise AssertionError(f"Unexpected Git request: {args}")

        with tempfile.TemporaryDirectory() as directory:
            workflow = Path(directory) / "renamed.yml"
            workflow.write_text(head, encoding="utf-8")
            with (patch.object(CHECK_GATES, "git", side_effect=git),
                  patch.object(CHECK_GATES, "base_workflow_paths", return_value=list(sources)),
                  patch.object(CHECK_GATES, "workflow_file", return_value=str(workflow)),
                  patch.dict(os.environ, {"RESULTS": json.dumps({}), "GITHUB_EVENT_NAME": "pull_request"})):
                CHECK_GATES.main([
                    ".github/workflows/renamed.yml", gate,
                    "--exempt", "codecov", "--exempt", "fuzzing",
                ])

    def test_path_listing_preserves_linebreaks_and_rejects_invalid_utf8(self):
        paths = [".github/workflows/validate\r.yml", ".github/workflows/validate\n.yml"]
        completed = subprocess.CompletedProcess([], 0, "\0".join(paths).encode("utf-8") + b"\0")
        with patch.object(CHECK_GATES.subprocess, "run", return_value=completed) as run:
            self.assertEqual(CHECK_GATES.base_workflow_paths(), paths + [""])
            self.assertNotIn("encoding", run.call_args.kwargs)
            self.assertNotIn("text", run.call_args.kwargs)
        with patch.object(CHECK_GATES.subprocess, "run",
                          return_value=subprocess.CompletedProcess([], 0, b"\xff\0")):
            with self.assertRaises(UnicodeDecodeError):
                CHECK_GATES.base_workflow_paths()

    def test_renamed_file_and_id_preserve_removal_checks_for_the_same_reported_name(self):
        with self.assertRaisesRegex(SystemExit, "validate.*windows-sync-safety"):
            self.run_gate("replacement", "validate-gates")

    def test_renamed_file_with_original_id_still_rejects_undeclared_removals(self):
        with self.assertRaisesRegex(
            SystemExit, "codecov, coverage, fuzzing, lint, validate, windows-sync-safety"
        ):
            self.run_gate("validate-gates", "validate-gates")

    def test_a_new_gate_with_a_distinct_reported_name_has_no_predecessor(self):
        self.run_gate("replacement", "new-gates")

    def test_digit_prefixed_static_names_can_identify_unrelated_new_gates(self):
        for name in ("2026 checks", "1st gate", "-1 release checks"):
            with self.subTest(name=name):
                self.run_gate("replacement", name)

    def test_unreadable_base_indentation_cannot_establish_absence(self):
        base = ("name: Base\n\njobs:\n    former:\n      name: validate-gates\n"
                "      runs-on: ubuntu-latest\n    critical:\n      runs-on: ubuntu-latest\n")
        with self.assertRaisesRegex(SystemExit, "Cannot read base workflow"):
            self.run_gate("replacement", "validate-gates", sources={".github/workflows/base.yml": base})

    def test_declared_removals_allow_the_same_reported_name(self):
        self.run_gate(
            "replacement", "validate-gates",
            "codecov, coverage, fuzzing, lint, validate, validate-gates, windows-sync-safety",
        )

    def test_explicit_base_context_survives_a_different_internal_id(self):
        base = BASE.replace("  validate-gates:\n", "  former:\n    name: validate-gates\n")
        with self.assertRaisesRegex(SystemExit, "These jobs were removed"):
            self.run_gate("replacement", "validate-gates", sources={".github/workflows/base.yml": base})

    def test_quoted_context_names_preserve_removal_checks(self):
        for name in ('"Validate gates"', "'Validate gates'", '"validate-\\u0067ates"'):
            with self.subTest(name=name):
                base = BASE.replace("  validate-gates:\n", f"  former:\n    name: {name}\n")
                with self.assertRaisesRegex(SystemExit, "These jobs were removed"):
                    self.run_gate("replacement", name, sources={".github/workflows/base.yml": base})

    def test_a_matching_context_and_an_unrelated_matching_id_are_ambiguous(self):
        other = "name: Other\n\njobs:\n  replacement:\n    runs-on: ubuntu-latest\n"
        with self.assertRaisesRegex(SystemExit, "More than one base workflow"):
            self.run_gate("replacement", "validate-gates", sources={
                ".github/workflows/validate.yml": BASE,
                ".github/workflows/other.yml": other,
            })

    def test_possible_expression_contexts_cannot_waive_removals(self):
        base = BASE.replace("  validate-gates:\n", "  former:\n    name: validate${{ inputs.suffix }}\n")
        with self.assertRaisesRegex(SystemExit, "These jobs were removed"):
            self.run_gate("replacement", "validate-gates", sources={".github/workflows/base.yml": base})

    def test_disjoint_expression_contexts_do_not_create_a_predecessor(self):
        base = "name: Other\n\njobs:\n  former:\n    name: Other (${{ matrix.language }})\n"
        self.run_gate("replacement", "new-gates", sources={".github/workflows/base.yml": base})

    def test_expression_literal_ends_cannot_overlap(self):
        for pattern, context, expected in (
            ("new-${{ matrix.flavour }}-gates", "new-gates", False),
            ("new-${{ matrix.flavour }}gates", "new-gates", True),
            ("}}${{ inputs.context", "protected", True),
        ):
            with self.subTest(pattern=pattern):
                self.assertEqual(CHECK_GATES.possible_context(pattern, context), expected)

    def test_tab_comments_and_blank_lines_do_not_end_jobs(self):
        for separator in ("\t# separator", "\t", " \t# separator"):
            with self.subTest(separator=separator):
                parsed = CHECK_GATES.parse_jobs(
                    "name: Fixture\non: pull_request\njobs:\n  replacement:\n" + separator
                    + "\n    name: protected\n  critical:\n    runs-on: linux\n")
                self.assertEqual(parsed, ({"replacement": "protected", "critical": "critical"}, []))

    def test_unknown_base_context_syntax_cannot_waive_removals(self):
        for name in ("*context", ">-\n      validate-gates", "validate-\n      gates"):
            with self.subTest(name=name):
                base = BASE.replace("  validate-gates:\n", f"  former:\n    name: {name}\n")
                with self.assertRaisesRegex(SystemExit, "These jobs were removed"):
                    self.run_gate("replacement", "validate-gates", sources={".github/workflows/base.yml": base})

    def test_unknown_head_contexts_fail_before_declaring_a_new_gate(self):
        for name in ("${{ inputs.context }}", "*context", ">-\n      validate-gates",
                     "validate-\n      gates", "validate-gates\n    name: different"):
            with self.subTest(name=name):
                with self.assertRaisesRegex(SystemExit, "Cannot determine the gate context"):
                    self.run_gate("replacement", name)

    def test_flow_mapping_head_cannot_hide_a_context(self):
        head = ("name: Validate\n\njobs:\n  replacement: {name: validate-gates, "
                "runs-on: ubuntu-latest, steps: [{run: 'true'}]}\n")
        with self.assertRaisesRegex(SystemExit, "Cannot determine the gate context"):
            self.run_gate("replacement", "validate-gates", head=head)

    def test_unsupported_property_indentation_cannot_hide_a_base_context(self):
        for indent in ("   ", "      "):
            with self.subTest(indent=len(indent)):
                base = ("name: Base\n\njobs:\n  former:\n" + indent + "name: validate-gates\n"
                        + indent + "runs-on: ubuntu-latest\n  critical:\n"
                        + indent + "runs-on: ubuntu-latest\n")
                with self.assertRaisesRegex(SystemExit, "These jobs were removed"):
                    self.run_gate("replacement", "validate-gates",
                                  sources={".github/workflows/base.yml": base})

    def test_unsupported_property_indentation_cannot_hide_a_head_context(self):
        for indent in ("   ", "      "):
            with self.subTest(indent=len(indent)):
                head = ("name: Head\n\njobs:\n  replacement:\n" + indent + "name: validate-gates\n"
                        + indent + "runs-on: ubuntu-latest\n")
                with self.assertRaisesRegex(SystemExit, "Cannot determine the gate context"):
                    self.run_gate("replacement", "validate-gates", head=head)

    def run_cli_fixture(self, *, occupied=False, quoted_path=False, control_paths=False,
                        tab_comment=False, context="validate-gates", declaration="",
                        filename="renamed.yml"):
        git_executable = shutil.which("git")
        self.assertIsNotNone(git_executable, "Git is required for the gate CLI regression")
        with tempfile.TemporaryDirectory(prefix="gate-predecessor-") as directory:
            outer = Path(directory).resolve()
            self.assertEqual(outer.parent, Path(tempfile.gettempdir()).resolve())
            repo, home = outer / "repo", outer / "home"
            repo.mkdir()
            home.mkdir()
            env = {key: os.environ[key] for key in ("PATH", "SystemRoot", "TEMP", "TMP")
                   if key in os.environ}
            env.update(HOME=str(home), USERPROFILE=str(home), GIT_CONFIG_NOSYSTEM="1",
                       GIT_CONFIG_GLOBAL=os.devnull, GITHUB_EVENT_NAME="pull_request", RESULTS="{}")

            git_prefix = [rtk, "proxy", "git"] if (rtk := shutil.which("rtk")) else [git_executable]

            def git_command(*args, raw=False):
                return subprocess.run([*git_prefix, *args], cwd=repo, env=env,
                                      capture_output=True, text=not raw,
                                      encoding=None if raw else "utf-8", check=True, timeout=10)

            workflows = repo / ".github/workflows"
            workflows.mkdir(parents=True)
            source_workflows = ROOT / ".github/workflows"
            for source in sorted([*source_workflows.glob("*.yml"), *source_workflows.glob("*.yaml")]):
                (workflows / source.name).write_bytes(source.read_bytes())
            base_name = "validate\r.yml" if control_paths else "v\u00e1lidate.yml" if quoted_path else "validate.yml"
            base = (source_workflows / "validate.yml").read_text(encoding="utf-8")
            source_argument = ("$'.github/workflows/validate\\r.yml'" if control_paths
                               else f".github/workflows/{base_name}")
            if quoted_path or control_paths:
                base = base.replace(".github/workflows/validate.yml", source_argument)
            if quoted_path or control_paths:
                (workflows / "validate.yml").unlink()
                (workflows / base_name).write_text(base, encoding="utf-8")
            if control_paths:
                (workflows / "validate\n.yml").write_text(
                    self.head_workflow("unrelated", "unrelated"), encoding="utf-8")
            if occupied:
                filename = "occupied.yml"
                (workflows / filename).write_text(
                    self.head_workflow("replacement", "unrelated"), encoding="utf-8")
            git_command("init", "--quiet", "--initial-branch=main")
            git_command("config", "core.quotePath", "true")
            git_command("add", "--", ".github/workflows")
            identity = ("-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid",
                        "-c", "commit.gpgsign=false")
            git_command(*identity, "commit", "--quiet", "-m", "Base fixture")
            git_command("checkout", "--quiet", "-b", "change")
            (workflows / base_name).unlink()
            aggregate = base.split("\n  validate-gates:\n", 1)[1]
            self.assertIn(NEEDS, aggregate)
            aggregate = aggregate.replace(NEEDS, "", 1).replace(
                f"{source_argument} validate-gates",
                f".github/workflows/{filename} replacement")
            head = ("name: Validate\n\non: pull_request\n\njobs:\n  replacement:\n"
                    f"    name: {context}\n" + aggregate)
            self.assertIn(f".github/workflows/{filename} replacement", head)
            if tab_comment:
                head = head.replace("  replacement:\n", "  replacement:\n\t# separator\n", 1)
            if declaration:
                head = f"# removed-jobs: {declaration}\n" + head
            (workflows / filename).write_text(head, encoding="utf-8")
            git_command("add", "--all", "--", ".github/workflows")
            git_command(*identity, "commit", "--quiet", "-m", "Rename fixture")
            git_command("checkout", "--quiet", "main")
            (repo / ".github/first-parent-control.txt").write_text("base advances\n", encoding="utf-8")
            git_command("add", "--", ".github/first-parent-control.txt")
            git_command(*identity, "commit", "--quiet", "-m", "Advance base fixture")
            git_command(*identity, "merge", "--quiet", "--no-ff", "change", "-m", "Merge fixture")
            self.assertEqual(len(git_command("rev-list", "--parents", "-n", "1", "HEAD").stdout.split()), 3)
            self.assertEqual(git_command("show", f"HEAD^1:.github/workflows/{base_name}").stdout, base)
            self.assertEqual(git_command("show", f"HEAD^2:.github/workflows/{filename}").stdout, head)
            changes = git_command("diff", "--no-renames", "--name-status", "-z", "HEAD^1", "HEAD", raw=True).stdout.decode("utf-8").split("\0")[:-1]
            expected = {("M", f".github/workflows/{filename}")} if filename == base_name else {
                ("D", f".github/workflows/{base_name}"),
                ("M" if occupied else "A", f".github/workflows/{filename}"),
            }
            self.assertEqual(set(zip(changes[::2], changes[1::2])), expected)
            if quoted_path:
                self.assertIn('".github/workflows/v\\303\\241lidate.yml"', git_command(
                    "ls-tree", "-r", "--name-only", "HEAD^1", "--", ".github/workflows/").stdout)
            if control_paths:
                paths = git_command("ls-tree", "-r", "--name-only", "-z", "HEAD^1", "--", ".github/workflows/", raw=True).stdout.decode("utf-8").split("\0")
                self.assertIn(".github/workflows/validate\r.yml", paths)
                self.assertIn(".github/workflows/validate\n.yml", paths)
            result = subprocess.run([
                sys.executable, "-I", str(ROOT / ".github/ci/check_gates.py"),
                f".github/workflows/{filename}", "replacement",
                "--exempt", "codecov", "--exempt", "fuzzing",
            ], cwd=repo, env=env, capture_output=True, text=True, timeout=10)
            return result

    def test_cli_rejects_the_rename_in_a_real_git_parent(self):
        result = self.run_cli_fixture()
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("These jobs were removed", result.stderr)
        self.assertIn("validate-gates", result.stderr)

    def test_cli_rejects_an_occupied_destination_bypass(self):
        result = self.run_cli_fixture(occupied=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("More than one base workflow", result.stderr)

    def test_cli_retains_context_after_a_tab_comment(self):
        result = self.run_cli_fixture(tab_comment=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("These jobs were removed", result.stderr)

    def test_cli_retains_removals_from_a_git_quoted_path(self):
        result = self.run_cli_fixture(quoted_path=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("lint", result.stderr)

    def test_cli_accepts_declared_removals_from_a_git_quoted_path(self):
        result = self.run_cli_fixture(quoted_path=True,
            declaration="codecov, coverage, fuzzing, lint, validate, validate-gates, windows-sync-safety")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_cli_accepts_an_unrelated_new_context(self):
        result = self.run_cli_fixture(quoted_path=True, context="new-gates")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_cli_deduplicates_an_existing_path(self):
        result = self.run_cli_fixture(filename="validate.yml",
            declaration="codecov, coverage, fuzzing, lint, validate, validate-gates, windows-sync-safety")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    @unittest.skipIf(os.name == "nt", "CR/LF filenames require a POSIX working tree")
    def test_distinct_cr_lf_paths_retain_predecessor_removals(self):
        result = self.run_cli_fixture(control_paths=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn("lint", result.stderr)

    @unittest.skipIf(os.name == "nt", "CR/LF filenames require a POSIX working tree")
    def test_distinct_cr_lf_paths_accept_declared_removals(self):
        result = self.run_cli_fixture(control_paths=True,
            declaration="codecov, coverage, fuzzing, lint, validate, validate-gates, windows-sync-safety")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    @unittest.skipIf(os.name == "nt", "CR/LF filenames require a POSIX working tree")
    def test_distinct_cr_lf_paths_do_not_invent_a_predecessor(self):
        result = self.run_cli_fixture(control_paths=True, context="new-gates")
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
