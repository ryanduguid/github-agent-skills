# Local gates

Prerequisite: Python 3.11+.

Run the incremental checks while authoring a skill:

```shell
python scripts/validate_skills.py --sync
python -m unittest discover -s tests -v
```

The final repository gate additionally requires:

```shell
python scripts/validate_skills.py --strict
python scripts/check_public_files.py
```
