# Australian Accounting Skills guidance review

Date: 2026-08-31

## Decision

Retain the tested cross-runtime contributor consolidation after reconciling it
onto the current remote `main`. The untouched repository originally passed its
documented gates, but its entry points inverted the approved shared-guide
contract: `AGENTS.md` delegated to `CLAUDE.md`, while `CLAUDE.md` did not import
`@AGENTS.md`.

Current fetched remote base:
`faae4fdff7b8e4c5c1dc12d6746559d9c0839715`
(`docs: define canonical release history (#66)`).

Current local head:
`dc4af1493a12cc888847fa889ebaea3fc3b8601d`
(`docs: consolidate cross-runtime contributor guidance`).

Pre-reconciliation recovery branch:
`codex/backup-showcase-before-reconcile-20260831` at
`41cbbc5e019731813a11320ade261d4b2b704688`.

The exact verified origin was
`https://github.com/ryanduguid/australian-accounting-skills.git` for fetch and
push. The worktree was clean before the read-only fetch. The local commit was
rebased onto fetched `origin/main` without conflict; no remote ref was changed.
`git merge-base --is-ancestor origin/main HEAD` exits 0.

## Test-first consolidation

The focused contributor-check test was added first. It requires `AGENTS.md` to
retain the substantive repository, safety, privacy, accuracy, verification,
maintenance, writing, and hand-off sections, and requires `CLAUDE.md` to be
exactly `@AGENTS.md`, with an optional final newline.

RED:

```powershell
python -m unittest tests.test_contributor_checks -v
```

The new contract test failed because four substantive sections still existed
only in `CLAUDE.md` and because `CLAUDE.md` was not an import.

The consolidation then:

- made `AGENTS.md` the shared cross-runtime contributor guide;
- moved the existing scope, privacy, accuracy, professional-boundary,
  maintenance, and hand-off material into it;
- retained the repository map, 19-skill inventory, commands, and the distinct
  17-card validation inventory; and
- reduced `CLAUDE.md` to `@AGENTS.md`.

GREEN: the focused suite passes 2 tests.

## Current verification

All checks below ran again at the reconciled head:

| Command | Result |
| --- | --- |
| `python -m unittest tests.test_contributor_checks -v` | Passed; 2 tests. |
| `python -m pip install --disable-pip-version-check --no-deps --requirement requirements-test.txt` | Passed; pinned `PyYAML==6.0.3` satisfied. |
| `python -m unittest discover -s tests -v` | Passed; 63 tests and 1 expected environment-dependent symlink skip. |
| `python scripts/validate_validation.py` | Passed; 17 fabricated cards and 19 skills. |
| `python tests/verify_skills_cli.py` | Passed; `skills@1.5.22` discovered all 19 expected skills. |
| `git diff --check origin/main...HEAD` | Passed. |

The current range contains exactly:

- `AGENTS.md`;
- `CLAUDE.md`; and
- `tests/test_contributor_checks.py`.

The upstream release-history documentation is preserved. The local range
changes no workflow, action pin, dependency, lockfile, manifest, release or
security policy, skill content, validation card, runtime behaviour, raw
capture, client data, or generated artifact. The worktree is clean. No push,
publication, repository setting change, or other external action was
performed; hosted CI was not observed or claimed.
