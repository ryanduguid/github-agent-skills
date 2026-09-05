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
| Unit suite | `python -m unittest discover -s tests -v`: 35 tests in 21.230s | Passed |
| Incremental validator | `python scripts/validate_skills.py`: exit 0 | Passed |
| Strict validator | `python scripts/validate_skills.py --strict`: exit 0 | Passed |
| Generated copies | `pwsh -File scripts/sync-skills.ps1 -Check`: exit 0 | Passed |
| Public tracked-file scanner | `python scripts/check_public_files.py`: exit 0 | Passed |
| Windows sync safety | Real NTFS junction tests for linked generated and canonical descendants; external sentinels unchanged | Passed |
| Branch whitespace | `git diff --check`: exit 0 | Passed |
| Local branch review | All fixed ancestors through `d30aa347916787cf4bbb26a69f9b002f6a75b932` plus the reviewed terminal working-tree fix | Findings corrected |
| Remote publication | Repository creation, push, metadata, topics, default branch, and hosted CI | **Pending explicit action-time confirmation** |

The local results do not establish GitHub-hosted CI or remote repository state.
Because the tracked review cannot contain its own commit hash, the ignored
final-fix report supplies the terminal immutable checkpoint and complete
reachable manifest for the action-time handoff.
