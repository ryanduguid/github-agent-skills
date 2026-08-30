"""Check public repository files for unsafe content."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path


CLONE_URL = "https://github.com/ryanduguid/github-agent-skills.git"
REPOSITORY_NAME = "github-agent-skills"
QUICK_START = re.compile(r"^## Quick start\s*$([\s\S]*?)(?=^## |\Z)", re.MULTILINE)
FENCED_COMMANDS = re.compile(r"```(?:powershell|shell)\n([\s\S]*?)```")
PRIVATE_CONFIG = re.compile(r"(?:^|/)(?:\.env(?:\.[^/]+)?|\.netrc|\.npmrc|credentials(?:\.[^/]+)?)$", re.IGNORECASE)
TEXT_RULES = (
    ("transcript-path", re.compile(r"(?:^|[\s/\\])(?:transcripts?|sessions?)[/\\][^\s]+", re.IGNORECASE)),
    ("raw-sdd-path", re.compile(r"\.superpowers[/\\]sdd[/\\][^/\\*\s]+[/\\]raw[/\\]", re.IGNORECASE)),
    ("private-user-path", re.compile(r"(?:[A-Za-z]:[/\\](?:Users|Documents and Settings)[/\\]|/(?:home|Users)/)[^/\\\s]+", re.IGNORECASE)),
    ("credential-assignment", re.compile(r"\b(?:api[_-]?key|access[_-]?token|auth[_-]?token|client[_-]?secret|password)\b\s*[:=]\s*\S+", re.IGNORECASE)),
    ("client-data", re.compile(r"\bclient[_-]?(?:data|records?|export|file)\b\s*[:=]", re.IGNORECASE)),
)


def permissions_block(workflow: str) -> list[str] | None:
    match = re.search(r"^permissions:\n((?:^[ ]{2}.*\n)+)", workflow, re.MULTILINE)
    return match.group(1).splitlines() if match else None


def quick_start_commands(readme: str) -> list[str]:
    section = QUICK_START.search(readme)
    if not section:
        return []
    return [line.strip() for block in FENCED_COMMANDS.findall(section.group(1)) for line in block.splitlines() if line.strip()]


def quick_start_failures(root: Path, readme: str) -> list[str]:
    commands = quick_start_commands(readme)
    if not commands:
        return ["README.md: missing Quick start commands"]
    failures = []
    for command in commands:
        parts = command.split()
        if parts == ["git", "clone", CLONE_URL] or parts == ["cd", REPOSITORY_NAME]:
            continue
        if parts[:4] == ["python", "-m", "unittest", "discover"]:
            try:
                target = parts[parts.index("-s") + 1]
            except (ValueError, IndexError):
                failures.append(f"README.md: invalid unittest command: {command}")
            else:
                if not (root / target).is_dir():
                    failures.append(f"README.md: missing test target: {target}")
            continue
        if len(parts) >= 2 and parts[0] == "python" and parts[1].endswith(".py"):
            if not (root / parts[1]).is_file():
                failures.append(f"README.md: missing Python script: {parts[1]}")
            continue
        if parts[:2] == ["pwsh", "-File"] and len(parts) >= 3:
            if not (root / parts[2]).is_file():
                failures.append(f"README.md: missing PowerShell script: {parts[2]}")
            continue
        failures.append(f"README.md: unsupported Quick start command: {command}")
    return failures


def tracked_paths(root: Path) -> list[Path]:
    result = subprocess.run(["git", "ls-files", "-z"], cwd=root, capture_output=True, check=True)
    return [Path(path) for path in result.stdout.decode().split("\0") if path]


def looks_like_text(content: bytes) -> bool:
    if not content or b"\0" in content:
        return False
    printable = sum(byte in (9, 10, 13) or 32 <= byte <= 126 for byte in content)
    return printable / len(content) >= 0.8


def scan_paths(root: Path, paths: list[Path]) -> list[str]:
    failures = []
    for path in paths:
        relative = path.as_posix()
        if PRIVATE_CONFIG.search(relative):
            failures.append(f"{relative}: private-config")
            continue
        content = (root / path).read_bytes()
        try:
            text = content.decode("utf-8")
        except UnicodeDecodeError:
            if looks_like_text(content):
                failures.append(f"{relative}: undecodable-text")
            continue
        for rule, pattern in TEXT_RULES:
            if pattern.search(text):
                failures.append(f"{relative}: {rule}")
                break
    return failures


def tracked_failures(root: Path) -> list[str]:
    paths = [
        path
        for path in tracked_paths(root)
        if not re.fullmatch(r"\.superpowers/sdd/[^/]+/task-\d+-report\.md", path.as_posix())
    ]
    return scan_paths(root, paths)


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    failures = tracked_failures(root)
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
