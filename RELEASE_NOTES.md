# Release notes

## Unreleased

The canonical skills moved from `.claude/skills/` to `skills/`, so
`gh skill install` finds them without `--allow-hidden-dirs`.

### Licence

Relicensed from Apache-2.0 to FSL-1.1-ALv2 (Functional Source License, with
Apache-2.0 as the future licence). Released versions keep their licence.

### Plugin manifests

The repository now has Claude Code and Codex plugin manifests and marketplace
files for its 5 skills, with no MCP server and no credentials.

### Specification checks

The validator now checks description and `compatibility` lengths from the
Agent Skills specification, refuses frontmatter values a YAML parser could read
differently, and fails on a relative reference that is not a file in the skill
directory.

### Hidden characters

`validate-skills` refuses a `SKILL.md` containing an invisible or
bidirectional control character (Trojan Source, CVE-2021-42574), meaning any
Unicode 18.0 Default_Ignorable_Code_Point, and names the line, column and code
point of the first one. A leading byte order mark, which
was already refused as missing frontmatter delimiters, is now reported this way.
Skill instructions are plain text, so an emoji built with a zero width joiner,
VARIATION SELECTOR-16 or tag characters, such as a family emoji or a
subdivision flag, also fails.

### Recorded runs

Run files can record a skill digest covering every file in the skill
directory. `github-skills validate-skills --skill-digest <skill>` prints
the line to record, and `--strict` fails on a run whose digest no longer
matches the skill files. Runs without a digest, which today is all of them, are
listed rather than failed.

### Rust checks

The Rust command in `tools/github-skills` replaces `scripts/validate_skills.py`
and `scripts/check_public_files.py`, with the same checks and failure messages;
invalid arguments and processing errors exit 2. Run it with
`cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- <command>`.
Strict validation now also requires one scenario in `tests/scenarios/` for
each skill, a rule the unit tests held before. The Validate workflow runs the
Rust tests on Linux and Windows, measures their line coverage for Codecov and
fuzzes the scanner weekly with cargo-fuzz instead of Atheris. The Python 3.14
requirement now applies only to the unit tests.

## v0.1.0

The first tagged release. It packages five Agent Skills for evidence-based
GitHub work, the scripts that check them, and the CI that runs those checks.
Python 3.14 or later is required, and the scripts use only the standard
library.

### Skills

- `github-repository-audit` assesses a repository from inspected evidence.
- `github-readme-polish` improves a README without adding claims the
  repository cannot support.
- `github-profile-curator` selects and presents a coherent profile portfolio.
- `github-release-prep` prepares an evidence-backed release handoff.
- `github-issue-to-pr` turns an issue into a narrow, tested local change.

Each skill treats issue and repository content as evidence, not instructions.
None of them grants authority to push, open pull requests, publish releases or
change GitHub settings; those actions still need explicit confirmation.

`.claude/skills/` is the canonical source, which Claude Code reads in place.
`python scripts/validate_skills.py --sync` writes a byte-identical copy to
`.agents/skills/` for Codex, and `--strict` checks that all five skills and the
copy are present, valid and identical. The validator rejects symbolic links
and junctions in the skill trees and skill files that are not UTF-8.

`validation/` records one run without the skill and one with it for each of
the five scenarios in `tests/scenarios/`, scored against each scenario's
rubric.

### Public-file checker

`python scripts/check_public_files.py` scans every tracked file. It fails on private configuration paths, client data paths,
transcript paths, personal user directories and credential-shaped
assignments, including quoted keys in JSON, YAML and TOML. It also fails when
the README Quick start no longer matches the checks the Validate workflow runs,
or when the workflow's permissions are wider than read-only contents.

### Continuous integration

The Validate workflow runs on every pull request and push to main:

- the unit tests, strict validation and the public-file check on Python 3.14
  and 3.15;
- the synchronisation safety tests on Windows, where junctions behave
  differently;
- Ruff over the Python files, installed from hash-pinned requirements;
- the unit tests under branch coverage, which must stay at 100%, with the
  report uploaded to Codecov.

One aggregate check, `validate-gates`, passes only when every required job
succeeds, and fails if a job is left out of it. A separate workflow rejects
pull requests and pushes that carry AI authorship credit. OpenSSF Scorecard,
CodeQL, SonarCloud and Codacy also report on the repository.

### Fuzzing

An Atheris target in `fuzz/` feeds the public-file scanner arbitrary bytes and
checks its text decoding, Quick start extraction and safety patterns. CI runs
it for 60 seconds after each push to main, weekly and on demand. The unit tests
replay the checked-in corpus on every platform.

### Licence

Apache-2.0, with attribution kept in `NOTICE`.
