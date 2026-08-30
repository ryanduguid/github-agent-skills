"""Validate canonical skills and their generated runtime copies."""

from __future__ import annotations

import argparse
import filecmp
import re
import sys
from pathlib import Path


APPROVED_NAMES = (
    "github-issue-to-pr",
    "github-profile-curator",
    "github-readme-polish",
    "github-release-prep",
    "github-repository-audit",
)
RUNTIME_ROOTS = (".agents/skills", ".claude/skills")
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
    canonical = root / "skills"
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
                failures.append(f"skills: missing required skill '{name}'")

    if not strict:
        for name, source in valid:
            for runtime in RUNTIME_ROOTS:
                copy = root / runtime / name / "SKILL.md"
                copy_name = copy.relative_to(root).as_posix()
                if copy.is_file() and not filecmp.cmp(source, copy, shallow=False):
                    failures.append(f"{copy_name}: differs from {source.relative_to(root).as_posix()}")
    else:
        expected_directories = {path.relative_to(canonical).as_posix() for path in canonical.rglob("*") if path.is_dir()}
        expected_files = {path.relative_to(canonical).as_posix() for path in canonical.rglob("*") if path.is_file()}
        for runtime in RUNTIME_ROOTS:
            destination = root / runtime
            actual_directories = {path.relative_to(destination).as_posix() for path in destination.rglob("*") if path.is_dir()} if destination.is_dir() else set()
            actual_files = {path.relative_to(destination).as_posix() for path in destination.rglob("*") if path.is_file()} if destination.is_dir() else set()
            for relative in sorted(expected_directories - actual_directories):
                failures.append(f"{runtime}/{relative}: missing generated directory")
            for relative in sorted(expected_files):
                source = canonical / relative
                copy = destination / relative
                copy_name = f"{runtime}/{relative}"
                if not copy.is_file():
                    failures.append(f"{copy_name}: missing generated copy")
                elif not filecmp.cmp(source, copy, shallow=False):
                    failures.append(f"{copy_name}: differs from skills/{relative}")
            for relative in sorted(actual_files - expected_files):
                failures.append(f"{runtime}/{relative}: unexpected generated file")
            for relative in sorted(actual_directories - expected_directories):
                failures.append(f"{runtime}/{relative}: unexpected generated directory")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate portable Agent Skills.")
    parser.add_argument(
        "--strict",
        action="store_true",
        help="require all five canonical skills and both generated runtime copies (default: validate discovered approved skills and existing copies)",
    )
    args = parser.parse_args()
    failures = validate(Path(__file__).resolve().parents[1], strict=args.strict)
    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
