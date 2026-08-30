# Local gates

Prerequisites: Python 3.11+ and PowerShell 7+ (`pwsh`).

Run the incremental checks while authoring a skill:

```powershell
pwsh -File scripts/sync-skills.ps1
python -m unittest discover -s tests -v
python scripts/validate_skills.py
pwsh -File scripts/sync-skills.ps1 -Check
```

The final repository gate additionally requires `python scripts/validate_skills.py --strict`.

## Publication checkpoint measured 2026-08-31

Environment: Python 3.11.9 and PowerShell 7.6.4.

| Gate | Evidence measured locally | Status |
| --- | --- | --- |
| Unit suite | `python -m unittest discover -s tests -v` — 29 tests in 5.796s | Passed |
| Incremental validator | `python scripts/validate_skills.py` — exit 0 | Passed |
| Strict validator | `python scripts/validate_skills.py --strict` — exit 0 | Passed |
| Generated copies | `pwsh -File scripts/sync-skills.ps1 -Check` — exit 0 | Passed |
| Public tracked-file scanner | `python scripts/check_public_files.py` — exit 0 | Passed |
| Branch whitespace | `git diff --check cbbf081b0579973b5d0a8b00facabdb61a237925..HEAD` — exit 0 | Passed |
| Local branch review | Structured manual review of `cbbf081b0579973b5d0a8b00facabdb61a237925..22f9e36ed9d81302c64edc27f1f8291fba57f051` | No actionable finding |
| Remote publication | Repository creation, push, metadata, topics, default branch, and hosted CI | **Pending explicit action-time confirmation** |

The local results do not establish GitHub-hosted CI or remote repository state.
See `audits/publication-review.md` for the reviewed commit set, inventory,
remaining risks, and the unexecuted publication preview.
