# Task 4 report: profile curation skill

## Evaluation setup and sanitisation

The scenario and minimum synthetic fixture were written before the skill. The
fixture contains a profile README/runbook, mixed public-original, fork,
archived, and non-public inventory entries, supplied pin state, and five
verified synthetic portfolio destinations. It contains no live account,
client, or private-repository metadata.

Raw evaluator responses are ignored and retained only at:

- `.superpowers/sdd/2026-08-31-portable-github-skills/raw/github-profile-curator-baseline.raw.txt`
- `.superpowers/sdd/2026-08-31-portable-github-skills/raw/github-profile-curator-forward.raw.txt`

Tracked baseline and forward files contain only criterion-level outcomes and a
bounded observation. They omit evaluator transcripts, workstation paths,
model narration, browser state, and real private metadata.

## RED / GREEN comparison

RED: the no-skill control made a coherent five-project selection and respected
the non-public, fork/archive, pin, and widget boundaries, but put the
positioning statement outside its proposed README fragment. It therefore did
not demonstrate the runbook's required coherent opening and Selected work
copy together.

GREEN: the named `$github-profile-curator` evaluation returned one positioning
statement, five eligible public-original-active projects, evidence and a
verified destination for every selection, and a review-only profile fragment
containing the opening statement plus Selected work. It excluded the fork,
archive, non-public project, and unverified-destination project; it proposed
no pin mutation or vanity widget. All five forward criteria passed.

The evaluator denied read-only shell access in both runs. The scenario supplies
bounded synthetic evidence for this expected case; the skill requires facts
outside that evidence to remain unverified, without weakening the criteria.

## Commands and results

```text
codex exec --ephemeral --ignore-user-config --ignore-rules --skip-git-repo-check -s read-only -C . ... github-profile-curator scenario
# baseline: exit 0; captured the profile-copy narrative gap

pwsh -NoProfile -File scripts/sync-skills.ps1
python scripts/validate_skills.py
pwsh -NoProfile -File scripts/sync-skills.ps1 -Check
python -m unittest discover -s tests -v
# all passed; 14 unit tests passed

codex exec --ephemeral --ignore-user-config --ignore-rules --skip-git-repo-check -s read-only -C . ... "$github-profile-curator" + scenario
# forward: exit 0; all five criteria passed

python C:/Users/-/.codex/skills/.system/skill-creator/scripts/quick_validate.py skills/github-profile-curator
# not runnable: optional PyYAML module is unavailable; no dependency added
```

## Files

- `skills/github-profile-curator/SKILL.md`
- Generated runtime copies under `.agents/skills/github-profile-curator/` and
  `.claude/skills/github-profile-curator/`
- `tests/scenarios/github-profile-curator.md`
- `tests/fixtures/profile-curation-inventory.md`
- `validation/baselines/github-profile-curator.txt`
- `validation/forward/github-profile-curator.txt`

## Self-review and concern

The self-contained skill uses a trigger-only description, has no extra
dependencies or references, requires a single narrative, limits selection to
four through six evidence-backed projects, verifies destinations, honours a
maintained runbook, and preserves the review-only/pin boundary. It deliberately
does not prescribe a fixed narrative or select by popularity.

Concern: the synthetic `portfolio.test` destinations demonstrate supplied link
verification only; a live profile review must independently verify its real
targets before using them.
