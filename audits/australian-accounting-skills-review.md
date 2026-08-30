# Australian Accounting Skills guidance review

Date: 2026-08-31

## Decision

Change the cross-runtime contributor entry points. The untouched repository
passed every documented gate, but the preflight contract check exposed drift:
`AGENTS.md` delegated the full contributor guide to `CLAUDE.md`, and
`CLAUDE.md` did not import `@AGENTS.md`. This inverted the approved shared-entry
contract and was not covered by the baseline suite.

Repository base: `9955e59e783cdd0967e11d09e2e25cfb55809483`

Repository commit: `41cbbc5e019731813a11320ade261d4b2b704688`
(`docs: consolidate cross-runtime contributor guidance`)

## Untouched baseline

The following commands ran before any edit, from the Accounting Skills
worktree:

| Command | Result |
| --- | --- |
| `pip install --disable-pip-version-check --no-deps --requirement requirements-test.txt` | Passed; installed pinned `PyYAML==6.0.3`. |
| `python -m unittest discover -s tests -v` | Passed; 62 tests, 0 failures, 1 environment-dependent symlink skip. |
| `python scripts/validate_validation.py` | Passed; 17 fabricated cards, 19 skills, exact tracked inventory. |
| `python tests/verify_skills_cli.py` | Passed; `skills@1.5.22` discovered all 19 expected skills. |
| `git diff --check` | Passed with exit code 0. |

The passing baseline established that repository behaviour and existing CI
gates were healthy. It did not override the separately documented
cross-runtime entry-point requirement.

## Drift proof and test-first consolidation

Before the documentation changed, `AGENTS.md` said that `CLAUDE.md` was the
full contributor guide and instructed agents to read it. A literal search of
`CLAUDE.md` found no `@AGENTS.md` import.

The focused contributor-check test was added first. It requires `AGENTS.md` to
retain the substantive repository, safety, privacy, accuracy, map,
verification, maintenance, writing and hand-off sections, and requires
`CLAUDE.md` to be exactly `@AGENTS.md`, with an optional final newline.

RED command:

```powershell
python -m unittest tests.test_contributor_checks -v
```

Result: failed as intended. The new contract test reported the four sections
still held only in `CLAUDE.md` and rejected the existing non-importing
`CLAUDE.md`; the existing CI-command coverage test passed.

The consolidation then:

- made `AGENTS.md` the shared cross-runtime contributor guide;
- moved the existing Scope and data, Accuracy and professional boundaries,
  Maintaining skills, and Before hand-off guidance into `AGENTS.md`;
- retained the existing privacy restrictions, source-verification rules,
  professional and human-action boundaries, repository map, 19-skill
  inventory, check commands, writing rules and hand-off requirements, without
  altering the separate 17-card validation inventory; and
- reduced `CLAUDE.md` to `@AGENTS.md` only.

GREEN command:

```powershell
python -m unittest tests.test_contributor_checks -v
```

Result: passed, 2 tests, 0 failures.

## Final verification

After the final documentation wording was settled, the focused suite and the
complete requested gates ran again:

| Command | Result |
| --- | --- |
| `python -m unittest tests.test_contributor_checks -v` | Passed; 2 tests, 0 failures. |
| `pip install --disable-pip-version-check --no-deps --requirement requirements-test.txt` | Passed; pinned dependency already satisfied. |
| `python -m unittest discover -s tests -v` | Passed; 63 tests, 0 failures, 1 environment-dependent symlink skip. |
| `python scripts/validate_validation.py` | Passed; 17 fabricated cards, 19 skills, exact tracked inventory. |
| `python tests/verify_skills_cli.py` | Passed; all 19 expected skills discovered. |
| `git diff --check` | Passed with exit code 0; Git emitted only its existing LF-to-CRLF checkout warning. |

## Boundaries

The Accounting Skills commit changes only `AGENTS.md`, `CLAUDE.md`, and
`tests/test_contributor_checks.py`. It adds no generic community file and does
not modify `CONTRIBUTING.md`, security or release policy, dependencies,
workflows, manifests, skill content, validation cards, or runtime behaviour.
No push, publication, repository setting change, or other external action was
performed.
