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
