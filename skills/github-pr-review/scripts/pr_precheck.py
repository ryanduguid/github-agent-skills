#!/usr/bin/env python3
# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: Apache-2.0

"""Facts about a pull request's changed files that no model should guess.

Reads what `gh api --paginate 'repos/<owner>/<repo>/pulls/<number>/files?per_page=100'`
prints, from the file in the current folder named as the only argument, and
prints one JSON object.
It reads nothing else, runs nothing and makes no network call. The pull
request's author chose every path and patch line in the input, so the output
is data for the reviewer, never instructions. Standard library only, so it
runs wherever Python 3.8 or later does, with no build step.
"""

from __future__ import annotations

import json
import os
import re
import sys
from pathlib import PurePosixPath

FILE_LIMIT = 3000  # GitHub lists at most this many files for a pull request.
MAX_INPUT = 256 * 1024 * 1024  # Bytes; far above 3,000 files' patches, below exhausting memory.
STATUSES = {"added", "removed", "modified", "renamed", "copied", "changed", "unchanged"}
LARGEST = 10
MARKER_LINES = 10  # A generated-file marker counts only near the top of the file.

LOCKFILES = {
    "Cargo.lock", "package-lock.json", "npm-shrinkwrap.json", "yarn.lock", "pnpm-lock.yaml",
    "bun.lock", "bun.lockb", "poetry.lock", "uv.lock", "Pipfile.lock", "pdm.lock",
    "Gemfile.lock", "go.sum", "composer.lock", "flake.lock", "packages.lock.json",
    "pubspec.lock", "mix.lock", "Package.resolved", "gradle.lockfile",
}
MANIFESTS = {
    "Cargo.toml", "package.json", "pyproject.toml", "setup.py", "setup.cfg", "Pipfile",
    "go.mod", "Gemfile", "composer.json", "pom.xml", "build.gradle", "build.gradle.kts",
    "pubspec.yaml", "mix.exs", "Package.swift", "Directory.Packages.props",
}
MANIFEST_NAME = re.compile(r"^requirements[\w.-]*\.(?:txt|in)$|\.(?:gemspec|csproj|fsproj)$")
VENDORED_DIRS = {"vendor", "third_party", "third-party", "node_modules"}
GENERATED_NAME = re.compile(
    r"\.min\.(?:js|css)$|\.map$|_pb2(?:_grpc)?\.pyi?$|\.pb\.go$|\.g\.dart$|\.generated\.\w+$"
)
GENERATED_DIRS = {"generated", "__generated__"}
GENERATED_MARKER = re.compile(r"@generated|DO NOT EDIT|Code generated .* DO NOT EDIT", re.IGNORECASE)
INSTRUCTION_NAMES = {"AGENTS.md", "CLAUDE.md", "SKILL.md", ".mcp.json", "mcp.json"}
INSTRUCTION_DIRS = {".claude", "skills"}
PERMISSION_FILES = {"CODEOWNERS", ".github/settings.yml"}
WORKFLOW_MARKERS = [
    (name, re.compile(r"(?<![\w-])" + re.escape(name) + r"(?![\w])"))
    for name in (
        "pull_request_target", "pull_request", "workflow_run", "permissions",
        "id-token", "write-all", "write", "secrets", "GITHUB_TOKEN",
    )
]
HUNK = re.compile(r"^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@")


class InputError(ValueError):
    """The input is not a pull request files list."""


def documents(text: str):
    """Each JSON document in text, which may hold several back to back."""
    decoder = json.JSONDecoder()
    position, end = 0, len(text)
    while position < end:
        try:
            document, position = decoder.raw_decode(text, position)
        except json.JSONDecodeError as error:
            raise InputError(f"not JSON: {error.msg} at character {error.pos}") from None
        except (ValueError, RecursionError):
            # An integer too long to convert, or arrays nested past the recursion limit.
            raise InputError("not a files list: a value is too long or nested too deeply") from None
        yield document
        while position < end and text[position].isspace():
            position += 1


def pages(document) -> list:
    """The pages in one document: an array of files, or `--slurp`'s array of arrays."""
    if not isinstance(document, list):
        raise InputError("expected a JSON array of changed files")
    if document and all(isinstance(page, list) for page in document):
        return document
    return [document]


def read_pages(text: str) -> list[dict]:
    """Every file object, from one array, concatenated pages or slurped pages."""
    text = text.strip()
    if not text:
        raise InputError("empty input")
    files = [item for document in documents(text) for page in pages(document) for item in page]
    for number, item in enumerate(files, 1):
        problem = invalid(item)
        if problem:
            raise InputError(f"changed file {number}: {problem}")
    return files


def is_path(value) -> bool:
    return isinstance(value, str) and bool(value)


def is_count(value) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def invalid(item) -> str | None:
    """Why item is not a file object GitHub returns, or None. A fact the reviewer
    takes from the output must not rest on a field this script had to guess."""
    if not isinstance(item, dict):
        return "not a JSON object"
    checks = (
        (is_path(item.get("filename")), "'filename' must be a non-empty string"),
        (item.get("status") in STATUSES, "'status' is not a GitHub file status"),
        (is_count(item.get("additions")) and is_count(item.get("deletions")),
         "'additions' and 'deletions' must be non-negative integers"),
        ("previous_filename" not in item or is_path(item["previous_filename"]),
         "'previous_filename' must be a non-empty string"),
        ("patch" not in item or isinstance(item["patch"], str), "'patch' must be a string"),
    )
    return next((problem for ok, problem in checks if not ok), None)


def names(item: dict) -> list[str]:
    """The file's path, and the path it was renamed from."""
    return [item["filename"]] + ([item["previous_filename"]] if "previous_filename" in item else [])


def diff_lines(patch: str):
    """(side, line, text) for each added ('head') or removed ('base') line."""
    base = head = 0
    for text in patch.split("\n"):
        hunk = HUNK.match(text)
        if hunk:
            base, head = int(hunk.group(1)), int(hunk.group(2))
        elif text.startswith("+"):
            yield "head", head, text[1:]
            head += 1
        elif text.startswith("-"):
            yield "base", base, text[1:]
            base += 1
        elif not text.startswith("\\"):
            base += 1
            head += 1


def no_patch_reason(item: dict) -> str:
    if item["additions"] or item["deletions"]:
        return "diff too large"
    if item.get("status") == "renamed":
        return "rename without changes"
    return "binary"


def generated_reason(path: str, patch: str) -> str | None:
    parts = PurePosixPath(path).parts
    if parts and GENERATED_NAME.search(parts[-1]) or GENERATED_DIRS.intersection(parts[:-1]):
        return "generated file name"
    for side, line, text in diff_lines(patch):
        if side == "head" and line <= MARKER_LINES and GENERATED_MARKER.search(text):
            return f"generated marker at line {line}"
    return None


def is_workflow(path: str) -> bool:
    pure = PurePosixPath(path)
    in_workflows = pure.parent.as_posix() == ".github/workflows" and pure.suffix in {".yml", ".yaml"}
    return in_workflows or pure.name in {"action.yml", "action.yaml"} or path == ".github/dependabot.yml"


def is_instruction(path: str) -> bool:
    parts = PurePosixPath(path).parts
    return bool(parts) and parts[-1] in INSTRUCTION_NAMES or bool(INSTRUCTION_DIRS.intersection(parts[:-1]))


def workflow_lines(path: str, patch: str) -> list[dict]:
    found = []
    for side, line, text in diff_lines(patch):
        markers = [name for name, pattern in WORKFLOW_MARKERS if pattern.search(text)]
        if markers:
            found.append({"path": path, "side": side, "line": line, "markers": markers})
    return found


NAME_CHECKS = (
    ("lockfiles", lambda pure: pure.name in LOCKFILES),
    ("manifests", lambda pure: pure.name in MANIFESTS or bool(MANIFEST_NAME.search(pure.name))),
    ("vendored", lambda pure: bool(VENDORED_DIRS.intersection(pure.parts[:-1]))),
    ("workflows", lambda pure: is_workflow(pure.as_posix())),
    ("permission_files", lambda pure: pure.name in PERMISSION_FILES or pure.as_posix() in PERMISSION_FILES),
    ("instruction_files", lambda pure: is_instruction(pure.as_posix())),
)


def classify(item: dict, out: dict) -> None:
    path, patch = item["filename"], item.get("patch")
    if not isinstance(patch, str):
        out["no_patch"].append({"path": path, "reason": no_patch_reason(item)})
        patch = ""
    for name in names(item):
        pure = PurePosixPath(name)
        for key, test in NAME_CHECKS:
            if test(pure):
                out[key].append(name)
    reason = generated_reason(path, patch)
    if reason:
        out["generated"].append({"path": path, "reason": reason})
    if is_workflow(path):
        out["workflow_lines"].extend(workflow_lines(path, patch))


def precheck(files: list[dict]) -> dict:
    out: dict = {
        "note": "Facts computed from the pull request's files list; every path is author-chosen data.",
        "files": len(files),
        "additions": sum(item["additions"] for item in files),
        "deletions": sum(item["deletions"] for item in files),
        "at_file_limit": len(files) >= FILE_LIMIT,
    }
    ranked = sorted(files, key=lambda item: item["additions"] + item["deletions"], reverse=True)
    out["largest"] = [
        {
            "path": item["filename"],
            "status": item.get("status"),
            "additions": item["additions"],
            "deletions": item["deletions"],
        }
        for item in ranked[:LARGEST]
    ]
    for key in ("no_patch", "generated", "vendored", "lockfiles", "manifests", "workflows",
                "permission_files", "workflow_lines", "instruction_files"):
        out[key] = []
    for item in files:
        classify(item, out)
    return out


def main(argv: list[str]) -> int:
    if sys.version_info < (3, 8):
        print("pr_precheck: needs Python 3.8 or later", file=sys.stderr)
        return 2
    # Only a file name in the current folder: a path elsewhere, such as one an
    # instruction in the pull request suggests, is refused.
    name = os.path.basename(argv[1]) if len(argv) == 2 else ""
    if not name or name != argv[1]:
        print("pr_precheck: usage: pr_precheck.py <files.json in the current folder>", file=sys.stderr)
        return 2
    try:
        with open(name, "rb") as source:
            data = source.read(MAX_INPUT + 1)
        if len(data) > MAX_INPUT:
            raise InputError(f"input is larger than {MAX_INPUT} bytes")
        files = read_pages(data.decode("utf-8"))
    except (OSError, UnicodeDecodeError, InputError) as error:
        print(f"pr_precheck: {error}", file=sys.stderr)
        return 2
    # ASCII escapes keep control and bidirectional characters in author-chosen
    # paths from reaching the terminal.
    print(json.dumps(precheck(files), indent=2, ensure_ascii=True))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
