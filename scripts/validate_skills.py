"""Validate the canonical skills in .claude/skills and the generated Codex copy."""

from __future__ import annotations

import argparse
import filecmp
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
RUNTIME_ROOTS = (".agents/skills",)


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
    lines = path.read_text(encoding="utf-8").splitlines()
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
    skills = sorted((path for path in canonical.iterdir() if path.is_dir()), key=lambda path: path.name) if canonical.is_dir() else []
    valid: list[tuple[str, Path]] = []

    for directory in skills:
        relative = directory.relative_to(root).as_posix()
        if directory.name not in APPROVED_NAMES:
            failures.append(f"{relative}: unexpected skill name")
            continue
        path = directory / "SKILL.md"
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
            for runtime in RUNTIME_ROOTS:
                copy = root / runtime / name / "SKILL.md"
                copy_name = copy.relative_to(root).as_posix()
                if copy.is_file() and not filecmp.cmp(source, copy, shallow=False):
                    failures.append(f"{copy_name}: differs from {source.relative_to(root).as_posix()}")
    else:
        trees = {tree: [] for tree in (canonical, *(root / runtime for runtime in RUNTIME_ROOTS))}
        for tree in trees:
            if _is_link(tree):
                failures.append(f"{tree.relative_to(root).as_posix()}: is a link, not an ordinary file or directory")
            elif tree.is_dir():
                trees[tree] = _ordinary_entries(tree, root, failures)
        expected_directories = {path.relative_to(canonical).as_posix() for path in trees[canonical] if path.is_dir()}
        expected_files = {path.relative_to(canonical).as_posix() for path in trees[canonical] if path.is_file()}
        for runtime in RUNTIME_ROOTS:
            destination = root / runtime
            actual_directories = {path.relative_to(destination).as_posix() for path in trees[destination] if path.is_dir()}
            actual_files = {path.relative_to(destination).as_posix() for path in trees[destination] if path.is_file()}
            for relative in sorted(expected_directories - actual_directories):
                failures.append(f"{runtime}/{relative}: missing generated directory")
            for relative in sorted(expected_files):
                source = canonical / relative
                copy = destination / relative
                copy_name = f"{runtime}/{relative}"
                if not copy.is_file():
                    failures.append(f"{copy_name}: missing generated copy")
                elif not filecmp.cmp(source, copy, shallow=False):
                    failures.append(f"{copy_name}: differs from {CANONICAL}/{relative}")
            for relative in sorted(actual_files - expected_files):
                failures.append(f"{runtime}/{relative}: unexpected generated file")
            for relative in sorted(actual_directories - expected_directories):
                failures.append(f"{runtime}/{relative}: unexpected generated directory")
    return failures


def sync(root: Path) -> list[str]:
    """Regenerate the Codex copy from .claude/skills/, refusing before any mutation if a link is found."""
    root = Path(root)
    canonical = root / CANONICAL
    destinations = [root / runtime for runtime in RUNTIME_ROOTS]
    failures: list[str] = []
    trees: dict[Path, list[Path]] = {}
    for tree in (canonical, *destinations):
        if _is_link(tree):
            failures.append(f"{tree.relative_to(root).as_posix()}: is a link, not an ordinary file or directory")
        else:
            trees[tree] = _ordinary_entries(tree, root, failures) if tree.is_dir() else []
    if failures:
        return failures

    approved = [
        path.relative_to(canonical)
        for path in trees[canonical]
        if path.relative_to(canonical).parts[0] in APPROVED_NAMES
    ]
    for destination in destinations:
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
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    failures = sync(root) if args.sync else []
    if not failures:
        failures = validate(root, strict=args.strict)
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
