# Publication review

Date: 2026-08-31

Root design commit: `34d0ab9a66fc9f80820475fae2d06b17d751219a`

Fixed ancestor head: `d30aa347916787cf4bbb26a69f9b002f6a75b932`

## Decision

The final-review findings have been corrected locally. The synchroniser now
rejects nested reparse points before mutation and never recursively deletes an
untrusted tree. The public scanner covers every tracked file and quoted
credential assignments. Evaluation records have been rescored against retained
ignored outputs under one unchanged rubric per scenario.

Remote publication remains pending. This review did not create a repository,
add a remote, push a ref, change GitHub metadata, or use browser state.

The commit containing this tracked review cannot contain its own final hash.
The ignored final-fix report and action-time handoff must therefore supply the
terminal immutable checkpoint hash and the complete ordered reachable manifest.
The fixed ancestors through `d30aa347916787cf4bbb26a69f9b002f6a75b932`
are enumerated below.

## Proposed repository metadata

| Field | Proposed value |
| --- | --- |
| Owner/name | `ryanduguid/github-agent-skills` |
| Visibility | Public |
| Default branch | `main` |
| Description | `Evidence-first GitHub workflow skills for Codex and Claude Code.` |
| Topics | `agent-skills`, `claude-code`, `codex`, `developer-tools`, `github`, `workflow-automation` |

These values are proposals only and have not been applied remotely.

## Fixed ancestor manifest

`git rev-list --reverse --topo-order
d30aa347916787cf4bbb26a69f9b002f6a75b932` contains these 17 commits, oldest
first:

1. `34d0ab9a66fc9f80820475fae2d06b17d751219a`: `docs: design cross-agent GitHub improvement system`
2. `cbbf081b0579973b5d0a8b00facabdb61a237925`: `docs: plan portable skills and GitHub portfolio hardening`
3. `7ec9205b84b810e83558721084ea9c9e7ea3f867`: `test: establish portable skill repository contracts`
4. `ab3a6e3788f88bcc2de38492f81f1e6f61d00502`: `fix: enforce generated skill drift checks`
5. `52fd9d4cf33b9b0767de10415ef80015a7e7e149`: `feat: add evidence-first repository audit skill`
6. `5ddabe3986aff1698872a0aeda371d95986c2a89`: `feat: add truth-first README polishing skill`
7. `de663d2cad09def7bed3de0c7192aa2c6f90a421`: `feat: add coherent GitHub profile curation skill`
8. `29334a8aeb58061cc307af7e1d202afd8cff64d2`: `feat: add release readiness and handoff skill`
9. `9be096ff0b6821ad090d25d945a189214796f1ae`: `feat: add issue-to-PR delivery skill`
10. `12a52905aa21f7084585434157aeea52cff39687`: `feat: synchronise skills for Codex and Claude Code`
11. `c92a0a2a8412ee7e8c1a72192a330f7c78ed5825`: `fix: reject linked skill destinations`
12. `fd6fcf69529fbd7559e8b5f726149eac4983c35a`: `docs: prepare portable GitHub skills for publication`
13. `981667ed74a44158ed1665ab8467d1a301f366e3`: `test: harden public repository checks`
14. `7edf9cc78c1aa7fb1c818177b6308c0746520681`: `test: fail closed for publication checks`
15. `22f9e36ed9d81302c64edc27f1f8291fba57f051`: `test: detect non-public key assignments`
16. `16836baf4ca637435626bcedc20b508bcc60e412`: `audit: prepare publication checkpoint`
17. `d30aa347916787cf4bbb26a69f9b002f6a75b932`: `audit: enumerate full publication history`

The terminal final-fix commit will be the only additional reachable commit.

## Evaluation comparison

Every score below was manually checked against the retained ignored raw output.
Each tracked summary records scenario/rubric revision, Codex CLI version,
isolation flags, raw-output SHA-256, and exit status without publishing the raw
response or a local absolute path.

| Scenario | Control | Named skill | Supported comparison |
| --- | ---: | ---: | --- |
| Repository audit | 4/5 | 5/5 | Stable five-column decision table and correct action conclusion. |
| README polish | 4/5 | 5/5 | Preserves the specialised legal boundary as its own section. |
| Profile curator | 4/5 | 5/5 | Includes positioning and selected work in one reviewable fragment. |
| Release prep | 5/5 | 5/5 | Non-discriminating; the earlier release-note-fidelity improvement claim is retired. |
| Issue to PR | 4/5 | 5/5 | Complete commit/PR handoff shape; both runs handled scope and remote safety. |

## Fresh local verification

Environment: Python 3.11.9; PowerShell 7.6.4; Codex CLI
0.151.0-alpha.7.2.

| Command | Result |
| --- | --- |
| `python -m unittest discover -s tests -v` | Exit 0; 35 tests passed in 21.230s |
| `python scripts/validate_skills.py` | Exit 0 |
| `python scripts/validate_skills.py --strict` | Exit 0 |
| `pwsh -File scripts/sync-skills.ps1 -Check` | Exit 0 |
| `python scripts/check_public_files.py` | Exit 0; every tracked path scanned |
| `git diff --check` | Exit 0 |

The focused Windows tests created real NTFS junctions for a linked approved
child, nested generated descendant, and nested canonical descendant. All were
rejected before mutation, and their external sentinels remained unchanged.

## Workflow and sync safety

Repository-level workflow permission remains `contents: read`; checkout
credentials are not persisted. The full Ubuntu validation job remains, and a
focused `windows-latest` job exercises NTFS junction behavior. Both jobs use
`actions/checkout@v7` and `actions/setup-python@v7`.

The synchroniser preflights canonical and generated trees without recursive
enumeration through links, copies ordinary files explicitly, and deletes
ordinary generated entries bottom-up. It retains partial-set generation,
non-mutating `-Check`, repository containment, and exact byte parity.

## Public tracked-file inventory and safety scan

The terminal checkpoint will contain 75 tracked files totalling 166,900 bytes:
41 Markdown, 12 text, 8 Python, 5 YAML, 2 JSON, 2 PowerShell, 1 `.gitignore`,
1 `.gz`-named fixture, and 3 extensionless files. The three SDD task reports
previously tracked by mistake have been removed from Git tracking; the ignored
local copies remain outside the publication set. No tracked SDD task report
remains.

The checkpoint reaches sanitised criterion-level evaluation summaries and
synthetic fixtures. It reaches no ignored raw evaluator output, untracked/local
coordination report, evaluator transcript, browser state, private
configuration, client record, credential value, or workstation path. The
public scanner examines every `git ls-files` path and reports path plus rule
only, never a matched value.

The `.tar.gz`-named fixture remains short UTF-8 synthetic evidence of an
unverified artifact rather than a packaged executable.

## Remaining risks

- GitHub-hosted CI, visibility, default branch, metadata, topics, branch
  protection, and remote commit state remain unverified because no remote
  repository has been created.
- The action major tags are mutable upstream references; Dependabot is
  configured for GitHub Actions updates.
- The release fixture is intentionally not a valid distributable archive.
- The dedicated Windows job has been exercised locally on this Windows host;
  its first GitHub-hosted run remains pending publication.
- GitHub CLI is unavailable locally, so the preview has not been executed.

## Publication preview: do not execute without confirmation

Immediately before execution, show the user the terminal final-fix checkpoint
hash from the ignored action-time handoff, target
`ryanduguid/github-agent-skills`, public visibility, and default branch `main`.
Abort if the confirmed hash differs or an existing `origin` points elsewhere.

```powershell
# Preview only. Run only after confirmation of the terminal final-fix commit.
gh repo create ryanduguid/github-agent-skills --public --description "Evidence-first GitHub workflow skills for Codex and Claude Code." --source . --remote origin
git push --set-upstream origin <CONFIRMED_FINAL_CHECKPOINT_COMMIT>:refs/heads/main
gh repo edit ryanduguid/github-agent-skills --default-branch main --description "Evidence-first GitHub workflow skills for Codex and Claude Code." --add-topic agent-skills --add-topic claude-code --add-topic codex --add-topic developer-tools --add-topic github --add-topic workflow-automation
```

After execution, verify remote URL, visibility, default branch, description,
topics, exact remote commit, and hosted CI separately. This review authorises
none of those actions.
