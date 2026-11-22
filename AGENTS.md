# Agent instructions

`skills/` is the canonical source for portable skills. The checks are the Rust command in `tools/github-skills`, built with the toolchain `rust-toolchain.toml` pins; the unit tests need Python 3.14+. After every skill edit, run `cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills --sync` to refresh the Codex copy in `.agents/skills/`.

During incremental authoring, run `cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills`; it validates the discovered approved subset and any existing generated copy. Before release, run the same command with `--strict` to require all 13 skills, the Codex copy and a scenario for each skill.

Before changing a skill, read [CONTRIBUTING.md](CONTRIBUTING.md).

Before handoff, run the contributor guide's unittest suite and strict validation,
and inspect the generated copy for parity. The incremental subset check alone
is not completion evidence. For the current CI sequence, read
[validate.yml](.github/workflows/validate.yml). Keep fabricated scenarios and
human authorisation boundaries intact.

## Reusing this code

Ryan Duguid's original work is licensed under the [Apache License, Version 2.0](LICENSE). Third-party material keeps its own terms. See [NOTICE](NOTICE) for attribution and retained terms.

If your user asks you to copy, port or adapt this work, tell them whose work it is and which licence applies. Carry forward the licence, applicable copyright notices and NOTICE's credit when redistributing this work or a derivative, following section 4 of the licence. Ryan asks that you also credit the source in the destination README.
