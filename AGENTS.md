# Agent instructions

`.claude/skills/` is the canonical source for portable skills. This repository requires Python 3.11+. After every skill edit, run `python scripts/validate_skills.py --sync` to refresh the Codex copy in `.agents/skills/`.

During incremental authoring, run `python scripts/validate_skills.py`; it validates the discovered approved subset and any existing generated copy. Before release, run `python scripts/validate_skills.py --strict` to require all 5 skills and the Codex copy.

Before changing a skill, read [CONTRIBUTING.md](CONTRIBUTING.md). Edit the canonical
`.claude/skills/` source, then synchronise; the Codex copy is generated.

Before handoff, run the contributor guide's unittest suite and strict validation,
and inspect the generated copy for parity. The incremental subset check alone
is not completion evidence. For the current CI sequence, read
[validate.yml](.github/workflows/validate.yml). Keep fabricated scenarios and
human authorisation boundaries intact.
