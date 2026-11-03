# Agent instructions

`.claude/skills/` is the canonical source for portable skills. The checks are the Rust command in `tools/github-skills`, built with the toolchain `rust-toolchain.toml` pins; the unit tests need Python 3.14+. After every skill edit, run `cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills --sync` to refresh the Codex copy in `.agents/skills/`.

During incremental authoring, run `cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills`; it validates the discovered approved subset and any existing generated copy. Before release, run the same command with `--strict` to require all 5 skills, the Codex copy and a scenario for each skill.

Before changing a skill, read [CONTRIBUTING.md](CONTRIBUTING.md). Edit the canonical
`.claude/skills/` source, then synchronise; the Codex copy is generated.

Before handoff, run the contributor guide's unittest suite and strict validation,
and inspect the generated copy for parity. The incremental subset check alone
is not completion evidence. For the current CI sequence, read
[validate.yml](.github/workflows/validate.yml). Keep fabricated scenarios and
human authorisation boundaries intact.
