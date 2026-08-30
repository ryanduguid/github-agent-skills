# Publication review

Date: 2026-08-31

Root design commit: `34d0ab9a66fc9f80820475fae2d06b17d751219a`

Reviewed implementation head: `22f9e36ed9d81302c64edc27f1f8291fba57f051`

Parent checkpoint: `16836baf4ca637435626bcedc20b508bcc60e412`

## Decision

The reviewed local implementation has no actionable correctness, safety, or
publication-content finding. Local gates passed. Remote publication remains
pending: this review did not create a repository, add a remote, push a ref,
change GitHub metadata, or use browser state.

The full immutable publication history through the parent checkpoint is
enumerated below. The commit that fixes this audit is necessarily omitted from
its own tracked contents: embedding its eventual hash would change that hash.
The action-time handoff must therefore supply the audit-fix commit's full hash
and the exact ordered `git rev-list --reverse --topo-order <hash>` output. The
tracked 16-commit ancestor manifest plus that terminal immutable hash is the
exact publication manifest and must be confirmed immediately before any
command in the publication preview is run.

## Proposed repository metadata

| Field | Proposed value |
| --- | --- |
| Owner/name | `ryanduguid/github-agent-skills` |
| Visibility | Public |
| Default branch | `main` |
| Description | `Evidence-first GitHub workflow skills for Codex and Claude Code.` |
| Topics | `agent-skills`, `claude-code`, `codex`, `developer-tools`, `github`, `workflow-automation` |

These values are proposals only and have not been applied remotely.

## Exact publication history through the parent checkpoint

`git rev-list --reverse --topo-order
16836baf4ca637435626bcedc20b508bcc60e412` returned these 16 commits, oldest
first. Subjects were compared with `git log --reverse --topo-order
--format='%H%x09%s' 16836baf4ca637435626bcedc20b508bcc60e412`.

1. **Design/planning:** `34d0ab9a66fc9f80820475fae2d06b17d751219a` — `docs: design cross-agent GitHub improvement system`
2. **Design/planning:** `cbbf081b0579973b5d0a8b00facabdb61a237925` — `docs: plan portable skills and GitHub portfolio hardening`
3. **Implementation:** `7ec9205b84b810e83558721084ea9c9e7ea3f867` — `test: establish portable skill repository contracts`
4. **Implementation:** `ab3a6e3788f88bcc2de38492f81f1e6f61d00502` — `fix: enforce generated skill drift checks`
5. **Implementation:** `52fd9d4cf33b9b0767de10415ef80015a7e7e149` — `feat: add evidence-first repository audit skill`
6. **Implementation:** `5ddabe3986aff1698872a0aeda371d95986c2a89` — `feat: add truth-first README polishing skill`
7. **Implementation:** `de663d2cad09def7bed3de0c7192aa2c6f90a421` — `feat: add coherent GitHub profile curation skill`
8. **Implementation:** `29334a8aeb58061cc307af7e1d202afd8cff64d2` — `feat: add release readiness and handoff skill`
9. **Implementation:** `9be096ff0b6821ad090d25d945a189214796f1ae` — `feat: add issue-to-PR delivery skill`
10. **Implementation:** `12a52905aa21f7084585434157aeea52cff39687` — `feat: synchronise skills for Codex and Claude Code`
11. **Implementation:** `c92a0a2a8412ee7e8c1a72192a330f7c78ed5825` — `fix: reject linked skill destinations`
12. **Implementation:** `fd6fcf69529fbd7559e8b5f726149eac4983c35a` — `docs: prepare portable GitHub skills for publication`
13. **Implementation:** `981667ed74a44158ed1665ab8467d1a301f366e3` — `test: harden public repository checks`
14. **Implementation:** `7edf9cc78c1aa7fb1c818177b6308c0746520681` — `test: fail closed for publication checks`
15. **Implementation:** `22f9e36ed9d81302c64edc27f1f8291fba57f051` — `test: detect non-public key assignments`
16. **Checkpoint:** `16836baf4ca637435626bcedc20b508bcc60e412` — `audit: prepare publication checkpoint`

The future explicit checkpoint push publishes exactly this reachable history
plus the terminal audit-fix checkpoint identified by full hash in the
action-time handoff. No local coordination report is reachable from that
checkpoint.

`codex exec review --help` documents only `--base <BRANCH>` and
`--commit <SHA>`; it does not document an exact two-SHA range review. No Codex
review session was started. The non-mutating fallback was a structured manual
review using:

```powershell
git log --format='%H%x09%s' --reverse cbbf081b0579973b5d0a8b00facabdb61a237925..HEAD
git diff --stat cbbf081b0579973b5d0a8b00facabdb61a237925..HEAD
git diff --name-status cbbf081b0579973b5d0a8b00facabdb61a237925..HEAD
git diff --check cbbf081b0579973b5d0a8b00facabdb61a237925..HEAD
```

The review inspected repository guidance, public documentation, five canonical
skills, generated copies, workflow and community files, synchronisation and
validation scripts, public-file scanner, unit tests, scenarios, fixtures, and
validation records. Generated-copy parity was independently checked. No
actionable finding was identified.

## Fresh local verification

Environment: Python 3.11.9; PowerShell 7.6.4.

| Command | Result |
| --- | --- |
| `python -m unittest discover -s tests -v` | Exit 0; 29 tests passed in 5.796s |
| `python scripts/validate_skills.py` | Exit 0 |
| `python scripts/validate_skills.py --strict` | Exit 0 |
| `pwsh -File scripts/sync-skills.ps1 -Check` | Exit 0 |
| `python scripts/check_public_files.py` | Exit 0 |
| `git diff --check cbbf081b0579973b5d0a8b00facabdb61a237925..HEAD` | Exit 0 |

## Workflow actions

The workflow has repository-level `contents: read`, disables persisted checkout
credentials, and uses only these external actions:

| Workflow reference | Official source | Tag resolved 2026-08-31 |
| --- | --- | --- |
| `actions/checkout@v7` | <https://github.com/actions/checkout> | `3d3c42e5aac5ba805825da76410c181273ba90b1` |
| `actions/setup-python@v7` | <https://github.com/actions/setup-python> | `5fda3b95a4ea91299a34e894583c3862153e4b97` |

The tag resolutions were measured with `git ls-remote --tags` against the two
official action repositories. The workflow intentionally uses maintained major
tags rather than immutable commit pins; Dependabot is configured weekly for
GitHub Actions.

## Public tracked-file inventory and safety scan

`git ls-files` returned 78 files totalling 167,424 bytes: 44 Markdown, 12 text,
8 Python, 5 YAML, 2 JSON, 2 PowerShell, 1 `.gitignore`, 1 `.gz`-named fixture,
and 3 extensionless files. No tracked file exceeded 1 MiB and no tracked file
contained a NUL byte. The `.tar.gz` fixture is deliberately short UTF-8
synthetic evidence of an unverified release artifact; it is not a packaged or
executable binary.

The repository scanner passed without printing sensitive values. A second
filename-only `git grep` scan covered credential assignments, local/private
paths, transcript references, browser state, private configuration, client
data, and scaffold markers across every tracked file. It found:

- no credential assignment and no local/private user path;
- transcript and browser terms only in policy, scanner tests, plans, and sanitised
  task reports; the reports refer to ignored repository-relative raw outputs,
  not retained transcripts or browser state;
- private-config and client-data terms only in rejection rules/tests or policy;
- scaffold terms only in validation rules/tests, plans, and the deliberate
  synthetic release-artifact fixture.

No secret value, workstation path, browser profile, private configuration,
client record, raw evaluator output, or unexpected executable binary was found.

## Links

All Markdown links used by public-facing repository documentation are local and
their targets exist. `https://duguid.com.au/`, which occurs in synthetic profile
fixture evidence, returned HTTP 200. `.test` destinations occur only in
synthetic evaluation fixtures and are deliberately non-production examples.

The prospective clone URL
`https://github.com/ryanduguid/github-agent-skills.git` is the only intentionally
unresolved public-facing repository link: the corresponding GitHub page
returned HTTP 404 before publication. No attempt was made to resolve it through
credentials or browser state.

## Remaining risks

- GitHub-hosted CI and default-branch protection are unverified because the
  remote repository does not yet exist.
- The action major tags are mutable upstream refs; their exact resolutions are
  recorded above, and Dependabot can propose later version updates.
- The `.tar.gz` release fixture is intentionally plain synthetic evidence, not
  a valid archive; consumers must not treat it as a distributable artifact.
- GitHub CLI is not installed in this environment, so the preview syntax was
  checked against the official `gh repo create` and `gh repo edit` manuals but
  was not executed locally.

## Publication preview — do not execute without confirmation

Immediately before execution, show the user the exact final audit-fix checkpoint commit hash,
the target `ryanduguid/github-agent-skills`, public visibility, and `main`, then
obtain action-time confirmation for these external writes. Abort if the
confirmed hash differs from the final audit-fix checkpoint or if an `origin` remote
already exists with a different URL.

```powershell
# Preview only. Run only after action-time confirmation of the exact final checkpoint commit.
gh repo create ryanduguid/github-agent-skills --public --description "Evidence-first GitHub workflow skills for Codex and Claude Code." --source . --remote origin
git push --set-upstream origin <CONFIRMED_FINAL_CHECKPOINT_COMMIT>:refs/heads/main
gh repo edit ryanduguid/github-agent-skills --default-branch main --description "Evidence-first GitHub workflow skills for Codex and Claude Code." --add-topic agent-skills --add-topic claude-code --add-topic codex --add-topic developer-tools --add-topic github --add-topic workflow-automation
```

After execution, verify the remote URL, public visibility, default branch,
description, topics, exact remote commit, and hosted CI separately. This review
does not authorise any of those actions.
