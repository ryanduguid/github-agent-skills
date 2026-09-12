"""Check public repository files for unsafe content."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path


CLONE_URL = "https://github.com/ryanduguid/github-agent-skills.git"
REPOSITORY_NAME = "github-agent-skills"
QUICK_START = re.compile(r"^## Quick start\s*$([\s\S]*?)(?=^## |\Z)", re.MULTILINE)
VALIDATE_JOB = re.compile(r"^  validate:\n((?:^(?:    .*)?$\n)+)", re.MULTILINE)
WORKFLOW_CHECK = re.compile(r"^ {8}run: (python .+?)\s*$", re.MULTILINE)
FENCED_COMMANDS = re.compile(r"```(?:powershell|shell)\n([\s\S]*?)```")
PRIVATE_CONFIG = re.compile(r"(?:^|/)(?:\.env(?:\.[^/]+)?|\.netrc|\.npmrc|credentials(?:\.[^/]+)?|\.(?:ssh|aws)(?:/|$)|(?:home|Users)/[^/]+/\.config(?:/|$))", re.IGNORECASE)
CLIENT_DATA_PATH = re.compile(r"(?:^|/)(?:clients?|customers?)(?:/|$)|(?:^|/)(?:client|customer)[_-](?:data|records?|export|files?)(?:[._-]|$)", re.IGNORECASE)
TEXT_RULES = (
    ("transcript-path", re.compile(r"(?:^|[\s/\\])(?:transcripts?|sessions?)[/\\][^\s]+", re.IGNORECASE)),
    ("raw-sdd-path", re.compile(r"\.superpowers[/\\]sdd[/\\][^/\\*\s]+[/\\]raw[/\\]", re.IGNORECASE)),
    ("private-user-path", re.compile(r"(?:[A-Za-z]:[/\\](?:Users|Documents and Settings)[/\\]|\\\\[^/\\\s]+[/\\]+(?:Users|Documents and Settings)[/\\]|(?<![A-Za-z0-9])/(?:home|Users)/)[^/\\\s]+", re.IGNORECASE)),
    ("credential-assignment", re.compile(r"(?<![A-Za-z0-9_-])['\"]?(?:[A-Za-z0-9_-]*(?:password|secret|token)[A-Za-z0-9_-]*|api[_ -]?key|access[_ -]?key|client[_ -]?secret|private[_ -]?key|(?!(?:public|example)[_-]?key\b)[A-Za-z0-9-]+[_-]key)['\"]?\s*[:=]\s*['\"]?\S+", re.IGNORECASE)),
    ("client-data", re.compile(r"^\s*(?:client|customer)[_-]?(?:data|records?|export|file|id|name)\s*[:=]", re.IGNORECASE | re.MULTILINE)),
)


def permissions_block(workflow: str) -> list[str] | None:
    match = re.search(r"^permissions:\n((?:^[ ]{2}.*\n)+)", workflow, re.MULTILINE)
    return match.group(1).splitlines() if match else None


def quick_start_commands(readme: str) -> list[str]:
    section = QUICK_START.search(readme)
    if not section:
        return []
    return [line.strip() for block in FENCED_COMMANDS.findall(section.group(1)) for line in block.splitlines() if line.strip()]


def supported_quick_start(workflow: str) -> tuple[str, ...]:
    """The quick start the workflow supports: clone this repository, then run its checks in order."""
    job = VALIDATE_JOB.search(workflow)
    checks = WORKFLOW_CHECK.findall(job.group(1)) if job else []
    return (f"git clone {CLONE_URL}", f"cd {REPOSITORY_NAME}", *checks)


def quick_start_failures(readme: str, workflow: str) -> list[str]:
    return [] if tuple(quick_start_commands(readme)) == supported_quick_start(workflow) else ["README.md: Quick start commands differ from the checks the Validate workflow runs"]


def tracked_paths(root: Path) -> list[Path]:
    result = subprocess.run(["git", "ls-files", "-z"], cwd=root, capture_output=True, check=True)
    return [Path(path) for path in result.stdout.decode().split("\0") if path]


def looks_like_text(content: bytes) -> bool:
    if not content:
        return False
    printable = sum(byte in (9, 10, 13) or 32 <= byte <= 126 for byte in content)
    return printable / len(content) >= 0.8


def decoded_text(content: bytes) -> tuple[str | None, str | None]:
    if content.startswith((b"\xff\xfe\0\0", b"\0\0\xfe\xff")):
        try:
            return content.decode("utf-32"), None
        except UnicodeDecodeError:
            return None, "unclassifiable-text"
    if content.startswith((b"\xff\xfe", b"\xfe\xff")):
        try:
            return content.decode("utf-16"), None
        except UnicodeDecodeError:
            return None, "unclassifiable-text"
    if b"\0" in content:
        return (None, "unclassifiable-text") if looks_like_text(content.replace(b"\0", b"")) else (None, None)
    try:
        return content.decode("utf-8"), None
    except UnicodeDecodeError:
        return (None, "undecodable-text") if looks_like_text(content) else (None, None)


def scan_paths(root: Path, paths: list[Path]) -> list[str]:
    failures = []
    for path in paths:
        relative = path.as_posix()
        if PRIVATE_CONFIG.search(relative):
            failures.append(f"{relative}: private-config")
            continue
        if CLIENT_DATA_PATH.search(relative):
            failures.append(f"{relative}: client-data")
            continue
        content = (root / path).read_bytes()
        text, decoding_failure = decoded_text(content)
        if decoding_failure:
            failures.append(f"{relative}: {decoding_failure}")
            continue
        if text is None:
            continue
        for rule, pattern in TEXT_RULES:
            candidate = text
            if rule == "credential-assignment" and path.parent.as_posix() == ".github/workflows" and path.suffix in (".yml", ".yaml"):
                # These Actions expressions reference the runtime token or a named secret; neither contains a credential value.
                candidate = re.sub(r"(?m)^[ \t]+(?:GH_TOKEN|GITHUB_TOKEN):[ \t]*\$\{\{[ \t]*(?:github\.token|secrets\.[A-Z][A-Z0-9_]*)[ \t]*\}\}[ \t]*\r?$", "", text)
            if pattern.search(candidate):
                failures.append(f"{relative}: {rule}")
                break
    return failures


def tracked_failures(root: Path) -> list[str]:
    return scan_paths(root, tracked_paths(root))


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    failures = tracked_failures(root)
    workflow = (root / ".github/workflows/validate.yml").read_text(encoding="utf-8")
    failures += quick_start_failures((root / "README.md").read_text(encoding="utf-8"), workflow)
    if permissions_block(workflow) != ["  contents: read"]:
        failures.append(".github/workflows/validate.yml: workflow permissions are not read-only contents")
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
