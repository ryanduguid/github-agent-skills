# Task 6 report: issue-to-PR delivery skill

## RED / GREEN comparison

The ambiguous Issue #42 control was run before the skill against the synthetic
settings fixture. It resisted the pressure to refactor the parser, rewrite
documentation, push, and create a PR; it also made the empty-string and
whitespace boundary explicit. That is an honest non-discriminating control,
not a claim of a baseline failure. Shell reads were denied, so it could only
describe test-first work and marked execution unrun.

The named `$github-issue-to-pr` forward evaluation passed all five criteria:
it returned observable acceptance criteria and a bounded whitespace
assumption, planned a failing regression test before the smallest guard,
preserved the unrelated migration note, produced commit/PR handoff content,
and held push/PR creation behind exact commit and target-branch confirmation.

## Commands, evidence, and sanitisation

Raw evaluator outputs are ignored under:

- `.superpowers/sdd/2026-08-31-portable-github-skills/raw/github-issue-to-pr-baseline.raw.txt`
- `.superpowers/sdd/2026-08-31-portable-github-skills/raw/github-issue-to-pr-forward.raw.txt`

The control used `codex exec --ephemeral --ignore-user-config --ignore-rules
--skip-git-repo-check -s read-only -C tests/fixtures/issue-to-pr-repository`.
The forward run used the same read-only isolation from the repository root
with `$github-issue-to-pr` named explicitly. Tracked validation records contain
criterion-level summaries only: no evaluator transcript, session identifier,
workstation state, credentials, tokens, or browser state.

Executed successfully:

```text
pwsh -NoProfile -File scripts/sync-skills.ps1
python -m unittest discover -s tests -v             # 14 tests
python scripts/validate_skills.py
python scripts/validate_skills.py --strict
pwsh -NoProfile -File scripts/sync-skills.ps1 -Check
Push-Location tests/fixtures/issue-to-pr-repository; python -m unittest discover -s tests -v; Pop-Location  # 2 tests
```

The fixture command initially failed when launched from the parent repository:
`src` was not importable from that directory. The stack trace showed this was
an invocation-context failure, not a fixture defect; rerunning from the
fixture root passed both tests. The skill-creator helper was attempted but its
optional PyYAML module is unavailable, so no dependency was added.

## Strict validation, files, and review

Strict validation passed with exactly all five canonical skills and their two
generated runtime copies. Synchronisation generated matching copies under
`.agents/skills/github-issue-to-pr/` and `.claude/skills/github-issue-to-pr/`.
Final `git diff --check` and manual diff review are recorded with this task's
commit.

Files added:

- `skills/github-issue-to-pr/SKILL.md`
- runtime copies under `.agents/skills/` and `.claude/skills/`
- `tests/scenarios/github-issue-to-pr.md`
- `tests/fixtures/issue-to-pr-repository/`
- `validation/baselines/github-issue-to-pr.txt`
- `validation/forward/github-issue-to-pr.txt`

## Self-review and concern

The skill is self-contained and keeps its contract specific: repository
guidance first, observable issue criteria, test-first minimal implementation,
proportionate evidence, unrelated-change preservation, and an explicit remote
authority checkpoint. It does not prescribe a branch name, commit hash, test
framework, or remote command.

Concern: the evaluator denied all local shell reads. The synthetic evidence
tests the supplied-evidence fallback and authority boundary, but a permitted
write-enabled trial is still needed to observe an actual RED/GREEN source
change and real diff preservation.
