"""Validate the canonical skills in .claude/skills and the generated Codex copy."""

from __future__ import annotations

import argparse
import filecmp
import hashlib
import re
import shutil
import stat
import sys
from pathlib import Path

APPROVED_NAMES = (
    "github-issue-to-pr",
    "github-profile-curator",
    "github-readme-polish",
    "github-release-prep",
    "github-repository-audit",
)
CANONICAL = ".claude/skills"
RUNTIME = ".agents/skills"
RUN_DIRECTORIES = ("validation/baselines", "validation/forward")
# A recorded run names its skill by file stem and records the skill digest as
# "- Skill digest v1:" followed by the backquoted digest, on the same line or
# the next.
RUN_DIGEST = re.compile(r"^- Skill digest v1:\s+`([0-9a-f]{64})`\.$", re.MULTILINE)
RUN_DIGEST_LABEL = re.compile(r"^- Skill digest\b", re.MULTILINE)


def _is_link(path: Path) -> bool:
    if path.is_symlink():
        return True
    try:
        attributes = getattr(path.lstat(), "st_file_attributes", 0)
    except FileNotFoundError:
        return False
    return bool(attributes & getattr(stat, "FILE_ATTRIBUTE_REPARSE_POINT", 0))


def _ordinary_entries(tree: Path, root: Path, failures: list[str]) -> list[Path]:
    """Every path under tree, never descending into a link, which is reported instead."""
    found: list[Path] = []
    pending = [tree]
    while pending:
        for path in sorted(pending.pop().iterdir()):
            if _is_link(path):
                failures.append(f"{path.relative_to(root).as_posix()}: is a link, not an ordinary file or directory")
                continue
            found.append(path)
            if path.is_dir():
                pending.append(path)
    return found


PLACEHOLDER = re.compile(r"\b(TBD|TODO|FIXME|placeholder|scaffold(?:ing)?)\b", re.IGNORECASE)


def frontmatter(path: Path) -> tuple[dict[str, str] | None, str | None]:
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except UnicodeDecodeError:
        return None, "not valid UTF-8 text"
    if not lines or lines[0] != "---":
        return None, "missing YAML frontmatter delimiters"
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None, "missing YAML frontmatter delimiters"
    values = {}
    for line in lines[1:end]:
        key, separator, value = line.partition(":")
        if separator and key in {"name", "description"}:
            values[key] = value.strip()
    return values, None


def validate(root: Path, strict: bool = False) -> list[str]:
    """Return deterministic contract failures; strict enforces fixed-five parity."""
    root = Path(root)
    failures: list[str] = []
    canonical = root / CANONICAL
    # Both modes, not strict alone. is_dir() and is_file() follow a link or a
    # junction, so an approved skill whose directory pointed outside the repository
    # reported success from the documented incremental command as long as its text
    # matched. Stop at the first linked tree: nothing below it can be trusted.
    for tree in (canonical, root / RUNTIME):
        for path in (tree, *tree.parents):  # pragma: no branch - the walk always stops at root
            if _is_link(path):
                failures.append(
                    f"{path.relative_to(root).as_posix()}: is a link, not an ordinary file or directory"
                )
                break
            if path == root:
                break
    if failures:
        return failures
    # Retain links for rejection even when their targets are missing or are files.
    skills = (
        sorted(
            (path for path in canonical.iterdir() if _is_link(path) or path.is_dir()),
            key=lambda path: path.name,
        )
        if canonical.is_dir()
        else []
    )
    valid: list[tuple[str, Path]] = []

    for directory in skills:
        relative = directory.relative_to(root).as_posix()
        if directory.name not in APPROVED_NAMES:
            failures.append(f"{relative}: unexpected skill name")
            continue
        if _is_link(directory):
            failures.append(f"{relative}: is a link, not an ordinary file or directory")
            continue
        path = directory / "SKILL.md"
        if _is_link(path):
            failures.append(f"{relative}/SKILL.md: is a link, not an ordinary file or directory")
            continue
        if not path.is_file():
            failures.append(f"{relative}: missing SKILL.md")
            continue
        metadata, error = frontmatter(path)
        file_name = path.relative_to(root).as_posix()
        if error:
            failures.append(f"{file_name}: {error}")
            continue
        if not metadata.get("name"):
            failures.append(f"{file_name}: missing name")
            continue
        if not metadata.get("description"):
            failures.append(f"{file_name}: missing description")
            continue
        if metadata["name"] != directory.name:
            failures.append(f"{file_name}: name must match directory name")
            continue
        if not metadata["description"].startswith("Use when"):
            failures.append(f"{file_name}: description must start with 'Use when'")
            continue
        match = PLACEHOLDER.search(path.read_text(encoding="utf-8"))
        if match:
            failures.append(f"{file_name}: forbidden placeholder '{match.group().upper()}'")
            continue
        valid.append((directory.name, path))

    if strict:
        present = {directory.name for directory in skills}
        for name in APPROVED_NAMES:
            if name not in present:
                failures.append(f"{CANONICAL}: missing required skill '{name}'")

    if not strict:
        for name, source in valid:
            copy = root / RUNTIME / name / "SKILL.md"
            copy_name = copy.relative_to(root).as_posix()
            # The skill directory too: is_file() and filecmp follow a linked parent.
            linked = next((path for path in (copy.parent, copy) if _is_link(path)), None)
            if linked is not None:
                failures.append(
                    f"{linked.relative_to(root).as_posix()}: is a link, not an ordinary file or directory"
                )
            elif copy.is_file() and not filecmp.cmp(source, copy, shallow=False):
                failures.append(f"{copy_name}: differs from {source.relative_to(root).as_posix()}")
    else:
        destination = root / RUNTIME
        trees = {tree: [] for tree in (canonical, destination)}
        failures_before_traversal = len(failures)
        for tree in trees:
            # A linked tree was already rejected before any skill was read.
            if tree.is_dir():
                trees[tree] = _ordinary_entries(tree, root, failures)
        if len(failures) != failures_before_traversal:
            return list(dict.fromkeys(failures))
        expected_directories = {path.relative_to(canonical).as_posix() for path in trees[canonical] if path.is_dir()}
        expected_files = {path.relative_to(canonical).as_posix() for path in trees[canonical] if path.is_file()}
        actual_directories = {path.relative_to(destination).as_posix() for path in trees[destination] if path.is_dir()}
        actual_files = {path.relative_to(destination).as_posix() for path in trees[destination] if path.is_file()}
        for relative in sorted(expected_directories - actual_directories):
            failures.append(f"{RUNTIME}/{relative}: missing generated directory")
        for relative in sorted(expected_files):
            source = canonical / relative
            copy = destination / relative
            copy_name = f"{RUNTIME}/{relative}"
            if not copy.is_file():
                failures.append(f"{copy_name}: missing generated copy")
            elif not filecmp.cmp(source, copy, shallow=False):
                failures.append(f"{copy_name}: differs from {CANONICAL}/{relative}")
        for relative in sorted(actual_files - expected_files):
            failures.append(f"{RUNTIME}/{relative}: unexpected generated file")
        for relative in sorted(actual_directories - expected_directories):
            failures.append(f"{RUNTIME}/{relative}: unexpected generated directory")
    return failures


def skill_digest(directory: Path) -> str:
    """Skill digest v1 of a canonical skill directory, as recorded in run files.

    The manifest has one line per regular file under the directory, at any
    depth: the lowercase hex SHA-256 of the file's bytes, 2 spaces, the path
    relative to the directory with '/' separators, then a line feed. Lines are
    sorted by that path in code point order, which is UTF-8 byte order. The
    digest is the lowercase hex SHA-256 of the UTF-8 manifest. Bytes are hashed
    as stored, with no line-ending conversion. Directories add nothing of their
    own. Callers validate the tree first, because links are skipped here.
    """
    files = sorted(
        (path for path in _ordinary_entries(directory, directory, []) if path.is_file()),
        key=lambda path: path.relative_to(directory).as_posix(),
    )
    manifest = "".join(
        f"{hashlib.sha256(path.read_bytes()).hexdigest()}  {path.relative_to(directory).as_posix()}\n"
        for path in files
    )
    return hashlib.sha256(manifest.encode("utf-8")).hexdigest()


def check_runs(root: Path) -> tuple[list[str], list[str]]:
    """Return failures for stale or malformed recorded runs, and notices for runs without a skill digest."""
    root = Path(root)
    failures: list[str] = []
    notices: list[str] = []
    current: dict[str, str] = {}
    for directory in RUN_DIRECTORIES:
        for run in sorted((root / directory).glob("*.txt")):
            name = run.relative_to(root).as_posix()
            if run.stem not in APPROVED_NAMES:
                failures.append(f"{name}: names no approved skill")
                continue
            text = run.read_text(encoding="utf-8")
            labels = RUN_DIGEST_LABEL.findall(text)
            if not labels:
                notices.append(f"{name}: no skill digest recorded; its results are not tied to a skill revision")
                continue
            recorded = RUN_DIGEST.findall(text)
            if len(labels) != 1 or len(recorded) != 1:
                failures.append(f"{name}: malformed skill digest; record exactly one '- Skill digest v1:' line")
                continue
            if run.stem not in current:
                current[run.stem] = skill_digest(root / CANONICAL / run.stem)
            if recorded[0] != current[run.stem]:
                failures.append(
                    f"{name}: stale; recorded skill digest {recorded[0]} differs from "
                    f"{CANONICAL}/{run.stem} digest {current[run.stem]}"
                )
    return failures, notices


def sync(root: Path) -> list[str]:
    """Regenerate the Codex copy from .claude/skills/, refusing before any mutation if a link is found."""
    root = Path(root)
    canonical = root / CANONICAL
    destination = root / RUNTIME
    failures: list[str] = []
    trees: dict[Path, list[Path]] = {}
    for tree in (canonical, destination):
        for path in (tree, *tree.parents):  # pragma: no branch - the walk always stops at root
            if _is_link(path):
                failures.append(f"{path.relative_to(root).as_posix()}: is a link, not an ordinary file or directory")
                break
            if path == root:
                if tree == canonical and not tree.exists():
                    # A missing canonical tree scanned as an empty source list, so the
                    # sync below deleted every generated skill, copied nothing and
                    # returned success.
                    failures.append(
                        f"{tree.relative_to(root).as_posix()}: is missing"
                    )
                elif tree.exists() and not tree.is_dir():
                    # An empty tree here would let the sync below reach
                    # mkdir(exist_ok=True), which raises FileExistsError against a
                    # regular file. Report it as a validation failure instead.
                    failures.append(
                        f"{tree.relative_to(root).as_posix()}: is not an ordinary directory"
                    )
                else:
                    trees[tree] = _ordinary_entries(tree, root, failures) if tree.is_dir() else []
                break
    if failures:
        return failures

    approved = [
        path.relative_to(canonical)
        for path in trees[canonical]
        if path.relative_to(canonical).parts[0] in APPROVED_NAMES
    ]
    for path in sorted(trees[destination], key=lambda path: len(path.parts), reverse=True):
        if path.is_dir():
            path.rmdir()
        else:
            path.unlink()
    destination.mkdir(parents=True, exist_ok=True)
    for relative in approved:
        if (canonical / relative).is_dir():
            (destination / relative).mkdir(exist_ok=True)
        else:
            shutil.copyfile(canonical / relative, destination / relative)
    return []


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate portable Agent Skills.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="require all five canonical skills and the generated Codex copy (default: validate discovered approved skills and an existing copy)",
    )
    parser.add_argument(
        "--sync",
        action="store_true",
        help="regenerate the Codex copy from .claude/skills/ before validating",
    )
    parser.add_argument(
        "--skill-digest",
        choices=APPROVED_NAMES,
        metavar="NAME",
        help="after strict validation, print the skill digest line to record in a new run file for skill NAME",
    )
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    failures = sync(root) if args.sync else []
    if not failures:
        failures = validate(root, strict=args.strict or args.skill_digest is not None)
    if not failures and args.strict:
        failures, notices = check_runs(root)
        for notice in notices:
            print(notice)
    if not failures and args.skill_digest:
        print(f"- Skill digest v1:\n  `{skill_digest(root / CANONICAL / args.skill_digest)}`.")
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
