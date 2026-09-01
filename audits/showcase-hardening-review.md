# Showcase repository hardening integrated review

Date: 2026-08-31

## Decision

The consolidated final-fix wave is complete locally. The four stale showcase
branches were backed up and rebased without conflict onto freshly fetched
remote `main` heads. Ozzit was already aligned and was not rebased. All five
review findings are fixed: MCP guidance now follows the upstream WebP proof
asset and render command; the public-file scanner detects UNC user paths;
Ozzit, corpus, and monthly guidance tests enumerate every scalar or multiline
workflow `run:` gate; and the two locally inspected account rows now carry
documentary/configuration risk wording.

The reviewed changes remain narrow guidance, policy-test, scanner-test, and
audit changes. All focused checks pass. Three complete Windows suites remain
non-green at pre-existing seams and are reported exactly below; those seams
are unchanged from the new remote bases. No hosted CI result is claimed.

## Reconciled checkpoints

| Repository | Exact fetched remote base | Final local head | Recovery ref before rebase |
| --- | --- | --- | --- |
| `Ozzit` | `a48fe571b495d98a47658ec901bca6c364729b2f` | `4219455282d5cdbf24d0a4cb6e28ec2d0b47a178` | Not applicable; already aligned, no rebase. |
| `australian-accounting-skills` | `faae4fdff7b8e4c5c1dc12d6746559d9c0839715` | `dc4af1493a12cc888847fa889ebaea3fc3b8601d` | `codex/backup-showcase-before-reconcile-20260831` -> `41cbbc5e019731813a11320ade261d4b2b704688` |
| `aus-accounting-mcp` | `7fdc82f99bd0043a8454d9a105a7cd1bfc6b4ed5` | `f9d10fd8e6ae25d44fdc6e1c29799da75f0a945e` | `codex/backup-showcase-before-reconcile-20260831` -> `ea23b9fe6240a885e9aada0f75a0b6e4435f968c` |
| `au-tax-legislation-corpus` | `e3d9c9b6e12c5f41bcae357a5d90475b4a3447b2` | `ca59dbd0bb013f53d3a080a4cf617f7d8d9a57b0` | `codex/backup-showcase-before-reconcile-20260831` -> `55de4ac49bf21f5d64e858eea80d67377bd6fccc` |
| `monthly-close-controls` | `56caebbaa3cdd406f59798170918633702ce3d29` | `4042683fb3a4d5ec45d0f0bae85c4ddff041539d` | `codex/backup-showcase-before-reconcile-20260831` -> `184e8e72dd6a8d56556791904d453779308e019b` |

The four reconciled repositories had clean worktrees and exact HTTPS fetch and
push origins under `ryanduguid` before fetch:

- `https://github.com/ryanduguid/australian-accounting-skills.git`;
- `https://github.com/ryanduguid/aus-accounting-mcp.git`;
- `https://github.com/ryanduguid/au-tax-legislation-corpus.git`; and
- `https://github.com/ryanduguid/monthly-close-controls.git`.

Fetch was read-only. No conflict occurred. No force, deletion, push, remote
rewrite, or remote-setting change occurred. For each showcase repository,
`git merge-base --is-ancestor origin/main HEAD` exits 0 and
`git diff --check origin/main...HEAD` exits 0.

## Exact local ranges and files

| Repository | Commits after the fetched base | Exact changed paths |
| --- | --- | --- |
| `Ozzit` | `0d03eca9b04712dfb12cbba008a6dcdb59405f5e`, `a87d06266250e3db2e5795698eaa7fc8893f382d`, `4219455282d5cdbf24d0a4cb6e28ec2d0b47a178` | `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `tools/tests/test_repository_policy.py` |
| `australian-accounting-skills` | `dc4af1493a12cc888847fa889ebaea3fc3b8601d` | `AGENTS.md`, `CLAUDE.md`, `tests/test_contributor_checks.py` |
| `aus-accounting-mcp` | `ee680b4bbf7c42a1dae9780da6f8d8c30fd7634c`, `0ac1be45649b29356b518fd1d2dfde8c57ecc61b`, `f9d10fd8e6ae25d44fdc6e1c29799da75f0a945e` | `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `tests/test_repository_guidance.py` |
| `au-tax-legislation-corpus` | `084b5c3b91ec8d65670a4bf25f1ebaec9df2c405`, `c6e924b53c50ca5090890aae2d03b83210743cc4`, `d4de4383e4fca895a77ef8a8b93b336c68be94e8`, `ca59dbd0bb013f53d3a080a4cf617f7d8d9a57b0` | `AGENTS.md`, `CLAUDE.md`, `tests/radar/test_repository_guidance.py` |
| `monthly-close-controls` | `3627bee4947806df322710ec8afd92007dc6b855`, `4042683fb3a4d5ec45d0f0bae85c4ddff041539d` | `AGENTS.md`, `CLAUDE.md`, `tests/test_repository_guidance.py` |

The MCP WebP asset, current release workflow, dependency/toolchain changes,
and render script belong to fetched remote base
`7fdc82f99bd0043a8454d9a105a7cd1bfc6b4ed5`; none is rewritten by the local
range. The other upstream release-history and repository-hygiene changes are
likewise preserved.

## Final-review findings and RED/GREEN proof

### MCP proof media

After rebase, the focused guidance suite initially failed 2 tests because
`AGENTS.md` and `CONTRIBUTING.md` still named deleted
`docs/quick-proof.gif`. After changing the expected path to the upstream WebP,
one test still failed because the supplementary command set omitted the
upstream README's exact render command:

```powershell
uv run --locked python scripts/render_demo_image.py docs/quick-proof.txt docs/quick-proof.webp
```

The final guidance names the WebP, lists that command, proves the derived file
exists, pins the command to upstream README evidence, and keeps
`tests/test_demo_media.py` in the load-bearing supplementary suite. Focused
GREEN is 4 tests; full GREEN is 88 tests.

### UNC private-user-path detection

The focused scanner regression failed for both a private UNC user path and a
mixed payload containing the exact allowed public GitHub API URL plus that
private path. The minimal regular-expression change adds the UNC user-root form
while retaining the exact public URL allowance.
Focused GREEN covers positive, negative, and mixed cases. The complete toolkit
suite passes 35 tests and the public-repository module passes 14 tests.

### Workflow command drift

Ozzit's old parser returned only scalar Python commands. Its new regression
failed when presented with a non-Python scalar and literal/folded multiline
gates. Corpus and monthly similarly failed because their tests enumerated
selected smoke blocks instead of all workflow gates.

All three parsers now use only the Python standard library and deterministic
semantic normalization: scalar values remain scalar, literal blocks normalize
to newline-separated commands, and folded blocks normalize to space-separated
commands. Guidance lists each actual scalar command and explains the
multiline outside-checkout smoke without duplicating a giant shell body.

Mutation checks temporarily added, separately, a non-Python scalar gate and a
multiline gate to each real workflow. Each mutation made the focused guidance
suite fail; after exact restoration, each suite passed and every workflow was
byte-clean. Final focused counts are Ozzit 11, corpus 6, and monthly 5.

### Account audit wording

Only the `ryanduguid` and `ryanduguid.github.io` rows changed in both
`audits/2026-08-31-account-repository-audit.md` and
`audits/repository-inventory.json`. Their risk now states that limited local
documentary/configuration inspection found the listed public files and
workflows, while tests, presentation readiness, code quality, security posture,
and current hosted CI remain unverified. This is not a code or security
certification.

## Repository verification

### Ozzit

- Pinned `mypy==2.3.1` installation passed; mypy passed 22 source files.
- Workbook verification passed with 134 functions and 211 parts.
- Source, signature, previous-name, index, and AFE checks passed; signature
  proof covered 121 signatures, 126 parameter tables, and 123 examples, while
  previous-name proof covered 134 functions, 130 replacements, and 4 new.
- Focused repository-policy suite: 11 passed.
- Full unittest: 176 tests, 10 failures and 1 error. The non-green results
  remain the Windows CRLF-sensitive `HelpCorrectionsTests` seam.

The parser covers all nine current scalar workflow gates, including the
unittest gate. The affected implementation, tests, `src/`, and `ozzit.xlsx`
are unchanged from fetched `origin/main`.

### Australian Accounting Skills

- Focused contributor suite: 2 passed.
- Full unittest: 63 passed, with 1 expected environment-dependent symlink
  skip.
- Validation: 17 fabricated cards and 19 skills.
- Skills CLI 1.5.22: all 19 expected skills discovered.
- Pinned PyYAML install and diff check passed.

The separate repository review records the reconciliation and test-first
rationale in detail.

### Aus Accounting MCP

- Full pytest: 88 passed.
- Ruff passed; mypy passed 11 source files.
- Focused guidance: 4 passed.
- `uv lock --check` and `uv sync --locked --extra dev` passed.
- Build produced the 0.1.6 sdist and wheel.
- Fabricated demo produced the synthetic BAS proof and
  `ERR_POLICY_DIV7A_REFUSED`.
- The current WebP render command passed and left no diff.
- Demo/media/compatibility/engine supplementary group: 10 passed.

PATH-selected `uv` was 0.12.6; `python -m uv` was the workflow-pinned 0.12.0.
That distinction is retained rather than described as hosted-CI equivalence.

### Tax legislation corpus

- `compileall` passed.
- Focused guidance: 6 passed; supported Windows radar leg: 195 passed.
- Ruff passed; mypy passed 7 source files.
- Build produced the 0.1.3 sdist and wheel.
- Corpus unittest: 274 tests, 11 errors, 8 skips.
- Full locked Windows pytest with a fresh system-temp `--basetemp`: 453 passed,
  11 failed, 8 skipped, and 238 subtests passed.
- Outside-checkout installed-wheel smoke resolved the module from the fresh
  venv's `site-packages`; `compare` returned `REVIEW_REQUIRED` with one item,
  and `validate-review` returned `DECISION_RECORDED` with one decision.

Both red suites reach the retained handle-bound live-evidence rename seam. Its
implementation and failing tests are unchanged from fetched `origin/main`.

### Monthly Close Controls

- Focused guidance: 5 passed; release-workflow contracts: 6 passed.
- Full pytest: 229 passed, 1 failed, 1 skipped. The retained failure is the
  Windows viewer wording mismatch: `not found` versus expected
  `could not be read`.
- Ruff passed; mypy passed 9 source files.
- Build produced the 0.1.2 sdist and wheel.
- Outside-checkout installed-wheel smoke resolved the package from the fresh
  venv's `site-packages`, returned required `REVIEW` exit 2, and produced
  `pack/close-review-pack.json`.

The viewer and its failing test are unchanged from fetched `origin/main`.

### Toolkit

The toolkit has no configured origin, so it has no fetched remote checkpoint.
Its exact content parent before this audit commit is
`28ccb5a02f0f7011b807a8bbe68597a44633feb9`. The final-fix implementation
commits before this audit are:

- `036ef0258d6c825c9479115077a1d5d64c941450`,
  `fix: detect private UNC user paths`; and
- `28ccb5a02f0f7011b807a8bbe68597a44633feb9`,
  `docs: align profile audit risk wording`.

Fresh toolkit verification passes:

- complete unittest suite: 35 of 35;
- public-repository module: 14 of 14;
- direct public-file scanner;
- incremental validator and strict validator;
- skill synchronization check; and
- diff/public-file checks.

A Git commit cannot contain its own object ID. This file therefore records its
exact content parent; the resulting audit commit is recorded in the ignored
Task 7 report and SDD progress ledger.

## Boundaries and remaining risks

The known defect seams were compared directly with the new `origin/main`
bases, not the obsolete pre-rebase bases, and all are unchanged:

- Ozzit: `tools/postbuild/help_corrections.py`,
  `tools/tests/postbuild/test_help_corrections.py`, `src/`, and `ozzit.xlsx`;
- corpus: `fadden/export_live_evidence_bundles.py`,
  `tests/corpus/test_live_evidence_bundle_export.py`, and
  `tests/corpus/test_publication_bundle_export.py`; and
- monthly: `closecontrol/viewer.py` and `tests/test_viewer.py`.

The five local ranges change no workflow, dependency, lockfile, action pin,
badge, release/security policy, runtime behaviour, raw Register capture,
retained response, client file, or tracked build/distribution/media artifact.
Build checks created ignored local outputs only. Worktrees are clean and all
approved-path set checks pass.

The account audit covers 39 saved public repository rows. The current private
aggregate is unknown, and the historical 42-to-39 difference is not attributed
to private repositories. No private row or identifier is published.

Remaining risks are the three explicitly non-green Windows suites, unobserved
hosted CI, action-time remote drift, and action-time account state. Before any
future push or publication, re-fetch the remotes, confirm ancestry and clean
diffs, inspect then-current hosted checks and public account evidence, and
obtain separate authority for the external write. This review itself performs
and authorises no push, publication, tag, release, repository creation, remote
configuration, GitHub setting change, capture, or client-data action.
