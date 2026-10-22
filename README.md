# Portable GitHub skills

[![validate](https://img.shields.io/github/actions/workflow/status/ryanduguid/github-agent-skills/validate.yml?branch=main&label=validate&labelColor=000000&style=flat)](https://github.com/ryanduguid/github-agent-skills/actions/workflows/validate.yml)
[![Codecov](https://img.shields.io/codecov/c/github/ryanduguid/github-agent-skills?label=coverage&labelColor=000000&style=flat)](https://codecov.io/gh/ryanduguid/github-agent-skills)
[![CodeQL](https://img.shields.io/github/actions/workflow/status/ryanduguid/github-agent-skills/github-code-scanning%2Fcodeql?branch=main&label=CodeQL&labelColor=000000&style=flat)](https://github.com/ryanduguid/github-agent-skills/actions/workflows/github-code-scanning/codeql)
[![SonarCloud](https://img.shields.io/sonar/quality_gate/ryanduguid_github-agent-skills?server=https%3A%2F%2Fsonarcloud.io&label=SonarCloud&labelColor=000000&style=flat)](https://sonarcloud.io/summary/new_code?id=ryanduguid_github-agent-skills)
[![Codacy code quality](https://img.shields.io/codacy/grade/d1745a44b9934138ab60eb717cb99281?label=Codacy&labelColor=000000&style=flat)](https://app.codacy.com/gh/ryanduguid/github-agent-skills/dashboard)<br>
[![OpenSSF Scorecard](https://img.shields.io/ossf-scorecard/github.com/ryanduguid/github-agent-skills?label=Scorecard&labelColor=000000&style=flat)](https://scorecard.dev/viewer/?uri=github.com/ryanduguid/github-agent-skills)
[![OpenSSF Best Practices](https://img.shields.io/cii/level/15424?label=Best%20Practices&labelColor=000000&style=flat)](https://www.bestpractices.dev/projects/15424)<br>
[![release](https://img.shields.io/github/v/release/ryanduguid/github-agent-skills?color=66023C&labelColor=000000&style=flat)](https://github.com/ryanduguid/github-agent-skills/releases/latest)
[![licence: Apache-2.0](https://img.shields.io/badge/licence-Apache--2.0-66023C?labelColor=000000&style=flat&color=66023C)](LICENSE)

Five focused, portable Agent Skills for evidence-based GitHub work. The
canonical source is `.claude/skills/`, which Claude Code reads in place; a
generated copy in `.agents/skills/` serves Codex.

## Skills

- `github-repository-audit`: assess a repository from inspected evidence.
- `github-readme-polish`: improve README clarity without unsupported claims.
- `github-profile-curator`: select and present a coherent profile portfolio.
- `github-release-prep`: prepare an evidence-backed release handoff.
- `github-issue-to-pr`: turn an issue into a narrow, tested local change.

## Requirements

Python 3.14+ is required. The repository uses the Python standard library; no
package installation is needed.

## Quick start

```shell
git clone https://github.com/ryanduguid/github-agent-skills.git
cd github-agent-skills
python -m unittest discover -s tests -v
python scripts/validate_skills.py --strict
python scripts/check_public_files.py
```

These are the checks the Validate workflow runs, in the same order. The Codex
copy is tracked, so a fresh clone needs no synchronising step.

## Runtimes and installation

`.claude/skills/` is the only canonical source, and Claude Code reads it in
place. After changing a skill, run `python scripts/validate_skills.py --sync`;
it regenerates a byte-identical copy in `.agents/skills/` for Codex, then
validates the result. Do not edit the generated directory directly.

For a project-local installation, copy the directory for the runtime you use
into that project's corresponding discovery path. To keep a local clone as the
source of truth, update `.claude/skills/`, rerun the synchroniser, and copy the
runtime directory again. Use only the runtime you need.

## Validation

During authoring, run `python scripts/validate_skills.py`. Before sharing a
change, run the Quick start checks: the unit suite, the strict validator and
the public-file check. `GATES.md` lists the same repository gate. CI also
lints the Python files with a hash-pinned Ruff; [CONTRIBUTING.md](CONTRIBUTING.md)
shows how to run it locally.

## Recorded runs

`validation/baselines/` and `validation/forward/` hold the recorded A/B Codex
runs behind these skills: for every scenario in `tests/scenarios/`, one
no-skill control run and one named-skill run. Each file records the scenario
and rubric revision, the Codex CLI version (`codex-cli 0.151.0-alpha.7.2`), the
isolation flags, a PASS or FAIL against each rubric criterion, the SHA-256
digest of the retained raw output, and the run exit status.

Across the 5 pairs the controls fail 4 criteria in total and the
named-skill runs pass every criterion, but the release-prep pair is
non-discriminating because both of its runs pass. These files record what those
runs produced; they are not a promise that a rerun reproduces them.

## Boundary

These skills help inspect and prepare local work. They do not grant authority
to push, create pull requests or releases, alter GitHub settings, or make any
other remote mutation. Those actions require explicit confirmation at the
action boundary.

## Releases

[RELEASE_NOTES.md](RELEASE_NOTES.md) summarises each tagged release. Versions follow [Semantic Versioning](https://semver.org/).

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) and [SECURITY.md](SECURITY.md).

## Licence

Apache-2.0. See [LICENSE](LICENSE). If you redistribute this work or build on it, keep the attribution in [NOTICE](NOTICE).
