# Contributing

Keep each skill focused, evidence-based, and explicit about its authority
boundary. Edit only `.claude/skills/`, then regenerate the Codex copy:

```shell
python scripts/validate_skills.py --sync
python -m unittest discover -s tests -v
python scripts/validate_skills.py --strict
python scripts/check_public_files.py
```

CI also lints the Python files with Ruff, using the configuration in
`pyproject.toml`. To run the same check locally, install the hash-pinned
linter and run it:

```shell
python -m pip install --require-hashes --requirement requirements-lint.txt
python -m ruff check .
```

Include a focused test or scenario when behaviour changes, preserve unrelated
work, and describe the verification results in the pull request.
