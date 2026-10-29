"""Replay original Python assertions against the native candidate.

Python is required here to execute the unchanged reference unittest objects;
the native command itself has no Python dependency. All written inputs are
fabricated temporary fixtures. The JSON probe is a development example only.
"""
import argparse
import hashlib
import importlib
import json
import os
import queue
import shutil
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, required=True)
    parser.add_argument("--probe", type=Path, required=True)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--record", type=Path, required=True)
    parser.add_argument("--freeze-corpus", type=Path)
    args = parser.parse_args()
    args.probe = args.probe.resolve()
    args.binary = args.binary.resolve()
    root = args.root.resolve()
    sys.path.insert(0, str(root))
    from scripts import check_public_files as public
    from scripts import validate_skills as skills

    names = ["test_validate_skills", "test_sync_skills", "test_public_repository", "test_fuzz_harness"]
    modules = [importlib.import_module("tests." + name) for name in names]
    originals = {name: getattr(public, name) for name in (
        "scan_paths", "tracked_failures", "decoded_text", "looks_like_text", "quick_start_commands",
        "quick_start_failures", "supported_quick_start", "permissions_block")}
    original_validate = skills.validate
    child = subprocess.Popen([str(args.probe)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                             stderr=subprocess.PIPE, text=True, encoding="utf-8")
    replies = queue.Queue()
    reader = threading.Thread(target=lambda: [replies.put(line) for line in child.stdout], daemon=True)
    reader.start()
    deadline = time.monotonic() + 600
    operations = {}

    def rpc(operation, **values):
        if time.monotonic() >= deadline:
            raise TimeoutError("original-contract deadline reached")
        operations[operation] = operations.get(operation, 0) + 1
        child.stdin.write(json.dumps({"operation": operation, **values}) + "\n")
        child.stdin.flush()
        reply = json.loads(replies.get(timeout=10))
        if "error" in reply:
            raise RuntimeError(f"native {operation}: {reply['error']}")
        return reply["result"]

    def native_validate(root, strict=False):
        return rpc("validate", root=str(root), strict=strict)

    def native_sync(root):
        return rpc("sync", root=str(root))

    records = []
    class Result(unittest.TextTestResult):
        def startTest(self, test):
            self.before = dict(operations)
            super().startTest(test)

        def stopTest(self, test):
            records.append({"test": test.id(), "native_operations": {
                key: value - self.before.get(key, 0) for key, value in operations.items()
                if value != self.before.get(key, 0)}})
            super().stopTest(test)

    try:
        modules[0].validate = native_validate
        modules[1].validate = native_validate
        modules[1].sync = native_sync
        public.scan_paths = lambda root, paths: rpc("scan_paths", root=str(root), paths=[str(path) for path in paths])
        public.tracked_failures = lambda root: rpc("tracked_failures", root=str(root))
        public.decoded_text = lambda content: tuple(rpc("decoded_text", bytes=list(content)))
        public.looks_like_text = lambda content: rpc("looks_like_text", bytes=list(content))
        public.quick_start_commands = lambda readme: rpc("quick_start_commands", readme=readme)
        public.quick_start_failures = lambda readme, workflow: rpc("quick_start_failures", readme=readme, workflow=workflow)
        public.supported_quick_start = lambda workflow: tuple(rpc("supported_quick_start", workflow=workflow))
        public.permissions_block = lambda workflow: rpc("permissions_block", workflow=workflow)
        suite = unittest.TestSuite(unittest.defaultTestLoader.loadTestsFromModule(module) for module in modules)
        result = unittest.TextTestRunner(verbosity=2, resultclass=Result).run(suite)
        assert result.wasSuccessful(), "original assertions failed"
        for name, function in originals.items():
            setattr(public, name, function)
        assert all(getattr(public, name) is function for name, function in originals.items())

        # Freeze independent original-Python expected classifications as byte arrays
        # so the public-file scanner never needs an exclusion for unsafe fixtures.
        cases = []
        boundaries = ["", " ", "_", "é", "中", "\u0301", "\u200c", "\u203f", "Ⅻ", "²", "\x1c", "\x1f", "İ", "ı", "ſ", "K"]
        texts = ["safe prose", "client" + "_data: fabricated", '"client' + '_data": "fabricated"', "api" + "_key = fabricated", "public_key = example", "example_key = example", "transcripts" + "/run.txt", "sessions" + "/run.txt", ".superpowers/sdd/run" + "/raw/output.txt", "/home" + "/fabricated/note", "C:" + "\\Users\\Fabricated\\note", "C:" + "/" + "Users/Fabricated/note", "\\" * 2 + "server\\Users\\Fabricated\\note", "prİvate" + "_key = fabricated", "prıvate" + "_key = fabricated", "clİent" + "_data: fabricated", "CLıENT" + "_DATA: fabricated"]
        texts += [left + text + right for text in ("api" + "_key = fabricated", "transcripts" + "/run.txt", "/home" + "/fabricated/note") for left in boundaries for right in boundaries]
        texts += ["    GH_TO" + "KEN: ${{ github.token }}\n", "    KEY: ${{ secrets.TEST_KEY }}\n", "    KEY: ${{ secrets.TEST_KEY }}-literal\n", "    id-to" + "ken: write # permission\n", "    id-to" + "ken: fabricated\n"]
        payloads = [text.encode() for text in texts]
        for text in texts[:17]:
            for encoding, bom in (("utf-16-le", b"\xff\xfe"), ("utf-16-be", b"\xfe\xff"), ("utf-32-le", b"\xff\xfe\0\0"), ("utf-32-be", b"\0\0\xfe\xff")):
                payloads.append(bom + text.encode(encoding))
        payloads += [b"", b"\0\0", b"safe\0text", b"\xff\xfe\0\0\xff\xff\xff\xff", b"\xff\xfe\x00\xd8", b"note=\xffvalue\n", b"\0\x01\x02\x03\0\xfe"]
        for count in range(11):
            payloads.append(b"a" * count + b"\xff" * (10 - count))
        with tempfile.TemporaryDirectory(prefix="native-skills-corpus-") as directory:
            fixture = Path(directory)
            # These path rules reject before attempting a read, including names
            # Windows cannot create. The payload cases below also exercise IO.
            for relative in ("clients\n", "client_data\n", ".ssh\n", ".aws\n", ".env\n", "customers\n", "exports/customer_records\n", ".ENV", "CLıENT_DATA\n"):
                expected = originals["scan_paths"](fixture, [Path(relative)])
                assert len(expected) == 1 and "unreadable" not in expected[0]
                assert rpc("scan_paths", root=str(fixture), paths=[relative]) == expected, repr(relative)
                cases.append({"path": relative, "bytes": [], "expected": expected,
                              "decoded_bytes": [], "decoding_failure": None, "path_only": True})
            for index, payload in enumerate(payloads):
                relative = ".github/workflows/fixture.yml" if index % 3 == 0 else "fixture.txt"
                path = fixture / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(payload)
                expected = originals["scan_paths"](fixture, [Path(relative)])
                actual = rpc("scan_paths", root=str(fixture), paths=[relative])
                assert actual == expected, (index, actual, expected)
                expected_decoding = originals["decoded_text"](payload)
                assert tuple(rpc("decoded_text", bytes=list(payload))) == expected_decoding, index
                cases.append({"path": relative, "bytes": list(payload), "expected": expected,
                              "decoded_bytes": None if expected_decoding[0] is None else list(expected_decoding[0].encode()),
                              "decoding_failure": expected_decoding[1]})
            # Unicode boundaries, splitlines, duplicate keys and literal frontmatter.
            for left in boundaries:
                for right in boundaries:
                    for marker in ("TODO", "fixme", "scaffoldİng", "scaffoldıng"):
                        name = "github-repository-audit"
                        path = fixture / ".claude/skills" / name / "SKILL.md"
                        path.parent.mkdir(parents=True, exist_ok=True)
                        path.write_text(f"---\nname: {name}\ndescription: Use when testing.\n---\n{left}{marker}{right}\n", encoding="utf-8")
                        assert rpc("validate", root=str(fixture)) == original_validate(fixture), (left, marker, right)
            for separator in ("\n", "\r", "\r\n", "\v", "\f", "\x1c", "\x1d", "\x1e", "\x85", "\u2028", "\u2029"):
                path.write_bytes(separator.join(["---", f"name: {name}", "description: Use when testing.", "---", "safe"]).encode())
                assert rpc("validate", root=str(fixture)) == original_validate(fixture), repr(separator)

        # Real process checks, including a different working directory and explicit root.
        cli = []
        with tempfile.TemporaryDirectory(prefix="native-skills-cli-") as directory:
            fixture = Path(directory)
            script = fixture / "scripts/validate_skills.py"
            script.parent.mkdir()
            script.write_bytes((root / "scripts/validate_skills.py").read_bytes())
            for flags in ([], ["--strict"], ["--sync"], ["--sync", "--strict"]):
                expected = subprocess.run([sys.executable, "-I", str(script), *flags], cwd=fixture, capture_output=True, timeout=20)
                actual = subprocess.run([str(args.binary), "validate-skills", "--root", str(fixture), *flags], cwd=root, capture_output=True, timeout=20)
                assert (actual.returncode, actual.stdout, actual.stderr.replace(b"\r\n", b"\n")) == (expected.returncode, expected.stdout, expected.stderr.replace(b"\r\n", b"\n")), flags
                cli.append({"flags": flags, "exit": actual.returncode})
            for flags in (["missing-command"], ["validate-skills", "--unknown"], ["check-public-files", "--sync"], ["validate-skills", "--root"]):
                actual = subprocess.run([str(args.binary), *flags], cwd=fixture, capture_output=True, timeout=20)
                assert actual.returncode == 2 and not actual.stdout, flags
            actual = subprocess.run([str(args.binary), "validate-skills"], cwd=fixture, capture_output=True, timeout=20)
            assert actual.returncode == 0 and not actual.stdout and not actual.stderr
            # Successful sync is compared on separate, identical input trees.
            for name in skills.APPROVED_NAMES:
                path = fixture / ".claude/skills" / name / "SKILL.md"
                path.parent.mkdir(parents=True)
                path.write_text(f"---\nname: {name}\ndescription: Use when testing.\n---\n", encoding="utf-8")
                (path.parent / "data.bin").write_bytes(b"\0\xff\x10")
            stale = fixture / ".agents/skills/retired/obsolete.txt"
            stale.parent.mkdir(parents=True)
            stale.write_bytes(b"stale")
            native_tree = fixture / "native-copy"
            native_tree.mkdir()
            for name in (".claude", ".agents", "scripts"):
                shutil.copytree(fixture / name, native_tree / name)
            expected = subprocess.run([sys.executable, "-I", str(script), "--sync", "--strict"], cwd=root, capture_output=True, timeout=20)
            actual = subprocess.run([str(args.binary), "validate-skills", "--root", str(native_tree), "--sync", "--strict"], cwd=root, capture_output=True, timeout=20)
            assert (actual.returncode, actual.stdout, actual.stderr) == (expected.returncode, expected.stdout, expected.stderr) == (0, b"", b"")
            def generated(tree):
                return {str(path.relative_to(tree)): path.read_bytes() for path in (tree / ".agents").rglob("*") if path.is_file()}
            assert generated(fixture) == generated(native_tree)
            cli.append({"flags": ["--sync", "--strict"], "exit": 0, "generated_files": len(generated(native_tree)), "bytes_equal": True})
        with tempfile.TemporaryDirectory(prefix="native-public-cli-") as directory:
            fixture = Path(directory)
            for name in ("README.md", ".github/workflows/validate.yml", "scripts/check_public_files.py"):
                path = fixture / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes((root / name).read_bytes())
            subprocess.run(["git", "init", "-q"], cwd=fixture, check=True, timeout=10)
            subprocess.run(["git", "add", "."], cwd=fixture, check=True, capture_output=True, timeout=10)
            for mutation in ("clean", "unsafe text", "permissions", "quick start"):
                if mutation == "unsafe text":
                    (fixture / "notes.md").write_text("api" + "_key = fabricated\n", encoding="utf-8")
                    subprocess.run(["git", "add", "notes.md"], cwd=fixture, check=True, capture_output=True, timeout=10)
                elif mutation == "permissions":
                    workflow = fixture / ".github/workflows/validate.yml"
                    workflow.write_text(workflow.read_text().replace("permissions:\n  contents: read", "permissions:\n  contents: write", 1))
                elif mutation == "quick start":
                    readme = fixture / "README.md"
                    readme.write_text(readme.read_text().replace("git clone " + public.CLONE_URL, "git clone missing"))
                expected = subprocess.run([sys.executable, "-I", str(fixture / "scripts/check_public_files.py")], cwd=root, capture_output=True, timeout=20)
                actual = subprocess.run([str(args.binary), "check-public-files", "--root", str(fixture)], cwd=root, capture_output=True, timeout=20)
                assert (actual.returncode, actual.stdout, actual.stderr.replace(b"\r\n", b"\n")) == (expected.returncode, expected.stdout, expected.stderr.replace(b"\r\n", b"\n")), mutation
                cli.append({"public_mutation": mutation, "exit": actual.returncode})
        platform_checks = []
        with tempfile.TemporaryDirectory(prefix="native-path-parity-") as directory:
            fixture = Path(directory)
            for name in skills.APPROVED_NAMES:
                path = fixture / skills.CANONICAL / name / "SKILL.md"
                path.parent.mkdir(parents=True)
                path.write_text(f"---\nname: {name}\ndescription: Use when testing.\n---\n", encoding="utf-8")
            references = fixture / skills.CANONICAL / name / "references"
            references.mkdir()
            assert rpc("sync", root=str(fixture)) == skills.sync(fixture) == []
            if os.name == "nt":
                outside = fixture / "outside"
                outside.mkdir()
                links = [references / name for name in ("Z-link", "a-link")]
                try:
                    for link in links:
                        subprocess.run(["cmd", "/c", "mklink", "/J", str(link), str(outside)],
                                       check=True, capture_output=True, timeout=10)
                    for operation in ("validate", "sync"):
                        expected = original_validate(fixture, strict=True) if operation == "validate" else skills.sync(fixture)
                        actual = rpc(operation, root=str(fixture), strict=True)
                        assert actual == expected, (operation, actual, expected)
                    platform_checks.append("Windows mixed-case junction diagnostic order matches Python for strict validation and sync")
                finally:
                    for link in links:
                        if link.exists():
                            link.rmdir()
            else:
                copy = fixture / skills.RUNTIME / name / "references"
                (references / "\ufffd.txt").write_bytes(b"unicode")
                (references / "\ufffd-dir").mkdir()
                assert rpc("sync", root=str(fixture)) == skills.sync(fixture) == []
                raw = os.fsdecode(b"\xff.txt")
                raw_dir = os.fsdecode(b"\xff-dir")
                (copy / raw).write_bytes(b"surplus")
                (copy / raw_dir).mkdir()
                expected = original_validate(fixture, strict=True)
                actual = rpc("validate", root=str(fixture), strict=True)
                assert len(expected) == len(actual) == 2, (actual, repr(expected))
                assert sorted(line.rsplit(": ", 1)[1] for line in expected) == sorted(line.rsplit(": ", 1)[1] for line in actual)
                (copy / raw).unlink()
                (copy / raw_dir).rmdir()
                assert rpc("validate", root=str(fixture), strict=True) == original_validate(fixture, strict=True) == []
                platform_checks.append("POSIX malformed-byte file and directory cannot collide with replacement-character names")
        if args.freeze_corpus:
            args.freeze_corpus.parent.mkdir(parents=True, exist_ok=True)
            args.freeze_corpus.write_text(json.dumps(cases, separators=(",", ":")) + "\n", encoding="utf-8")
        else:
            frozen = root / "tools/github-skills/tests/fixtures/public-cases.json"
            assert json.loads(frozen.read_text(encoding="utf-8")) == cases, "frozen oracle differs from the original Python results"
        args.record.write_text(json.dumps({"tests": result.testsRun, "skips": result.skipped,
            "operations": operations, "test_records": records, "extra_public_cases": len(cases),
            "extra_validation_cases": len(boundaries) ** 2 * 4 + 11, "cli": cli,
            "platform_checks": platform_checks, "original_helpers_restored": True,
            "original_sources": {name: hashlib.sha256((root / name).read_bytes()).hexdigest() for name in (
                "scripts/validate_skills.py", "scripts/check_public_files.py", *["tests/" + name + ".py" for name in names])}}, indent=2) + "\n")
    finally:
        child.stdin.close()
        try:
            child.wait(timeout=10)
        except subprocess.TimeoutExpired:
            child.kill()
            child.wait(timeout=10)
        reader.join(timeout=2)
        error = child.stderr.read()
        if error:
            print(error, file=sys.stderr)
        assert not reader.is_alive(), "probe output reader did not stop"


if __name__ == "__main__":
    main()
