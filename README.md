# Portable GitHub Skills

Five focused, portable Agent Skills for evidence-based GitHub work. The
canonical source is `skills/`; generated runtime copies support both Codex and
Claude Code.

## Skills

- `github-repository-audit` — assess a repository from inspected evidence.
- `github-readme-polish` — improve README clarity without unsupported claims.
- `github-profile-curator` — select and present a coherent profile portfolio.
- `github-release-prep` — prepare an evidence-backed release handoff.
- `github-issue-to-pr` — turn an issue into a narrow, tested local change.

## Requirements

Python 3.11+ and PowerShell 7+ (`pwsh`) are required. The repository uses the
Python standard library; no package installation is needed.

## Quick start

```powershell
git clone https://github.com/ryanduguid/github-agent-skills.git
cd github-agent-skills
pwsh -File scripts/sync-skills.ps1
python -m unittest discover -s tests -v
python scripts/validate_skills.py --strict
python scripts/check_public_files.py
pwsh -File scripts/sync-skills.ps1 -Check
```

## Runtimes and installation

`skills/` is the only canonical source. Run the synchroniser after changing a
canonical skill; it regenerates byte-identical copies in `.agents/skills/` for
Codex and `.claude/skills/` for Claude Code. Do not edit either generated
directory directly.

For a project-local installation, copy the generated directory for the runtime
you use into that project's corresponding discovery path. To keep a local
clone as the source of truth, update `skills/`, rerun the synchroniser, and
copy the generated runtime directory again. Use only the runtime you need.

## Validation

During authoring, run `python scripts/validate_skills.py`. Before sharing a
change, run the Quick start checks: the unit suite, strict validator, and
sync-drift check. `GATES.md` lists the same repository gate.

## Boundary

These skills help inspect and prepare local work. They do not grant authority
to push, create pull requests or releases, alter GitHub settings, or make any
other remote mutation. Those actions require explicit confirmation at the
action boundary.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).

## Licence

MIT. See [LICENSE](LICENSE).
