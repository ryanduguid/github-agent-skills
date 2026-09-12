# Agent instructions

`skills/` is the canonical source for portable skills. This repository requires Python 3.11+. After every skill edit, run `python scripts/validate_skills.py --sync` to refresh both runtime copies.

During incremental authoring, run `python scripts/validate_skills.py`; it validates the discovered approved subset and any existing generated copies. Before release, run `python scripts/validate_skills.py --strict` to require all five skills and both runtime copies.

Before changing a skill, read [CONTRIBUTING.md](CONTRIBUTING.md). Edit the canonical
`skills/` source, then synchronise; the runtime copies are generated.

Before handoff, run the contributor guide's unittest suite and strict validation,
and inspect both generated copies for parity. The incremental subset check alone
is not completion evidence. For the current CI sequence, read
[validate.yml](.github/workflows/validate.yml). Keep fabricated scenarios and
human authorisation boundaries intact.
