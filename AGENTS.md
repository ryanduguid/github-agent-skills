# Agent instructions

`skills/` is the canonical source for portable skills. After every skill edit, run `pwsh -File scripts/sync-skills.ps1` to refresh both runtime copies.

During incremental authoring, run `python scripts/validate_skills.py`; it validates the discovered approved subset and any existing generated copies. Before release, run `python scripts/validate_skills.py --strict` to require all five skills and both runtime copies. Run `pwsh -File scripts/sync-skills.ps1 -Check` to detect copy drift.
