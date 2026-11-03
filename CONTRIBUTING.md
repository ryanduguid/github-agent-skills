# Contributing

Keep each skill focused, evidence-based, and explicit about its authority
boundary. Edit only `.claude/skills/`, then regenerate the Codex copy:

```shell
cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills --sync
python -m unittest discover -s tests -v
cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills --strict
cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- check-public-files
```

To change the checker itself, run the checks its
[README](tools/github-skills/README.md) lists.

CI also lints the Python tests with Ruff, using the configuration in
`pyproject.toml`. To run the same check locally, install the hash-pinned
linter and run it:

```shell
python -m pip install --require-hashes --requirement requirements-lint.txt
python -m ruff check .
```

Include a focused test or scenario when behaviour changes, preserve unrelated
work, and describe the verification results in the pull request.
