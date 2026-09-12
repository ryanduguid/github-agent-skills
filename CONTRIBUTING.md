# Contributing

Keep each skill focused, evidence-based, and explicit about its authority
boundary. Edit only `skills/`, then regenerate the Codex and Claude Code copies:

```shell
python scripts/validate_skills.py --sync
python -m unittest discover -s tests -v
python scripts/validate_skills.py --strict
```

Include a focused test or scenario when behaviour changes, preserve unrelated
work, and describe the verification results in the pull request.
