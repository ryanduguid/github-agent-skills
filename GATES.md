# Local gates

Prerequisites: rustup, which installs the Rust release `rust-toolchain.toml`
pins, and Python 3.14+.

Run the incremental checks while authoring a skill:

```shell
cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills --sync
python -m unittest discover -s tests -v
```

The final repository gate additionally requires:

```shell
cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills --strict
cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- check-public-files
```

CI also runs `python -m ruff check .` with the linter pinned in
`requirements-lint.txt`.
