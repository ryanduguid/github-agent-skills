# Task 5 report — release preparation skill

## Result

Implemented `github-release-prep`, a read-only release-readiness and handoff
skill. It derives the release decision from repository-native guidance,
reconciles all available release surfaces, preserves changelog notes, and
separates remote publication from readiness.

## RED / GREEN comparison

The control correctly rejected publication but reduced the matching changelog
entry to prose, losing its `### Added` heading and bullet. The named forward
evaluation passed all five criteria, including source-faithful notes, exact
version drift (`2.4.0` versus `2.4.1`), absent checksum/provenance blockers,
honest unverified remote state, and an explicit post-readiness authority gate.

Criterion summaries are tracked in
`validation/baselines/github-release-prep.txt` and
`validation/forward/github-release-prep.txt`.

## Evaluation and sanitisation

Raw control: `.superpowers/sdd/2026-08-31-portable-github-skills/raw/github-release-prep-baseline.raw.txt`.
Raw named forward evaluation: `.superpowers/sdd/2026-08-31-portable-github-skills/raw/github-release-prep-forward.raw.txt`.

Both were read-only `codex exec` runs using only synthetic fixture values. Raw
outputs remain ignored by `.gitignore`; tracked validation files contain only
criterion summaries, no session identifiers, credentials, tokens, browser
state, or external repository data. The evaluator denied filesystem reads, so
both results explicitly used the bounded supplied evidence and marked unread
remote state unverified.

## Validation

Passed:

- `pwsh -File scripts/sync-skills.ps1`
- `python scripts/validate_skills.py`
- `pwsh -File scripts/sync-skills.ps1 -Check`
- `python -m unittest discover -s tests -v` (14 tests)
- named `$github-release-prep` forward evaluation

`--strict` is intentionally deferred: the planned fifth skill,
`github-issue-to-pr`, has not been implemented in this task.
The Skill Creator helper was also attempted, but its optional PyYAML dependency
is not installed in this environment; no dependency was added. The repository
validator above covers this skill's checked-in frontmatter, naming, and copy
parity.

## Files

- `skills/github-release-prep/SKILL.md`
- runtime copies under `.agents/skills/github-release-prep/` and
  `.claude/skills/github-release-prep/`
- `tests/fixtures/release-prep-repository/`
- `tests/scenarios/github-release-prep.md`
- release-prep baseline and forward criterion summaries

## Self-review and concerns

The skill adds no publishing command and does not treat an artifact's existence
or a successful build as integrity, provenance, current CI, or publication
evidence. Its generic surface list lets repository guidance define the actual
required files and checks. The initial control was already strong on the
primary safety boundary; the measured improvement is changelog-note fidelity,
not a claim that the named skill alone caused the publication refusal.
