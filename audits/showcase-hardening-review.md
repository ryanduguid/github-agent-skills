# Showcase repository hardening integrated review

Date: 2026-08-31

## Decision

The five approved showcase ranges are narrow, reviewable documentation and
policy-test changes. Every focused or newly added test passes. All complete
documented gates were run from clean worktrees at the exact reviewed heads.

Three full Windows suites remain red and are not publication-green claims:
Ozzit retains its CRLF-sensitive postbuild failures, the tax corpus retains its
handle-bound live-evidence promotion failures, and Monthly Close Controls
retains its viewer wording mismatch. The affected runtime and baseline-test
files are byte-unchanged across their reviewed ranges, the failure counts match
the earlier task evidence, and the new guidance tests pass.

The scanner false positive and the plan's fixed-total assumption have now been
corrected in two separately scoped toolkit commits. The complete integrated
verification was then replayed at the same five showcase heads. The set is
locally publication-ready, subject to action-time confirmation of remote
ancestry, hosted checks and then-current public account evidence. This review
authorises no push, publication, tag, release, repository creation, remote
configuration or GitHub setting change.

## Exact reviewed checkpoints

| Repository | Base | Reviewed head | Commits in range | Exact changed paths |
| --- | --- | --- | --- | --- |
| `Ozzit` | `a48fe571b495d98a47658ec901bca6c364729b2f` | `a87d06266250e3db2e5795698eaa7fc8893f382d` | `0d03eca9b04712dfb12cbba008a6dcdb59405f5e`, `a87d06266250e3db2e5795698eaa7fc8893f382d` | `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `tools/tests/test_repository_policy.py` |
| `australian-accounting-skills` | `9955e59e783cdd0967e11d09e2e25cfb55809483` | `41cbbc5e019731813a11320ade261d4b2b704688` | `41cbbc5e019731813a11320ade261d4b2b704688` | `AGENTS.md`, `CLAUDE.md`, `tests/test_contributor_checks.py` |
| `aus-accounting-mcp` | `c140f78ef4b2c4cad7d3631f2ad090efc426e7cd` | `ea23b9fe6240a885e9aada0f75a0b6e4435f968c` | `911d8d730b6538527a600ff849c1c4991d60e97c`, `ea23b9fe6240a885e9aada0f75a0b6e4435f968c` | `AGENTS.md`, `CLAUDE.md`, `CONTRIBUTING.md`, `tests/test_repository_guidance.py` |
| `au-tax-legislation-corpus` | `1844b865dab04267645d74a8ad1c1fb8f3a4a7db` | `55de4ac49bf21f5d64e858eea80d67377bd6fccc` | `53ad362bf71e3d8c2c4b7667a18a49221e18432f`, `1594a1086ca3a30b4f694aadc0a568cc723133f3`, `55de4ac49bf21f5d64e858eea80d67377bd6fccc` | `AGENTS.md`, `CLAUDE.md`, `tests/radar/test_repository_guidance.py` |
| `monthly-close-controls` | `d14d39741219387a929f68a4bbba8a4e844a7c13` | `184e8e72dd6a8d56556791904d453779308e019b` | `184e8e72dd6a8d56556791904d453779308e019b` | `AGENTS.md`, `CLAUDE.md`, `tests/test_repository_guidance.py` |
| `github-agent-skills` review content | `4fac42cdb70fb9c6e4bad4e1fa1dec876bf0bb8f` | `387a720c54c6f463ba1bec8965d329c9ad62a6c3` | `8d5e3836d2e7caf6ea8891b82090f89aa30bb7a0`, `b71afdce82915802f389e6a2fb0ea7e28cff2503`, `387a720c54c6f463ba1bec8965d329c9ad62a6c3` | `audits/showcase-hardening-review.md`, `docs/superpowers/plans/2026-08-31-showcase-repository-hardening.md`, `scripts/check_public_files.py`, `tests/test_public_repository.py` |

For every repository range, `git merge-base --is-ancestor <base> <head>`
exited 0. Before testing and again after testing, each supplied head was checked
out and `git status --short --untracked-files=all`, `git diff --exit-code` and
`git diff --cached --exit-code` were clean.

### Toolkit content parent and audit-commit self-reference

`4fac42cdb70fb9c6e4bad4e1fa1dec876bf0bb8f` was the exact toolkit content
parent before the initial audit-only commit
`8d5e3836d2e7caf6ea8891b82090f89aa30bb7a0`. The scanner fix and plan
amendment then produced content parent
`387a720c54c6f463ba1bec8965d329c9ad62a6c3` for this closing audit amendment.
A Git commit cannot contain its own final object ID because inserting that ID
changes the object being named. Therefore this file records the exact parent
and audit-only scope; the resulting closing-audit commit ID is recorded in the
ignored Task 7 report and must be included in the action-time handoff.

## Independent diff and boundary review

The complete diffs were reviewed from each stated base to head. A set-equality
check compared `git diff --name-only <base> <head>` with the approved paths in
the table above; all five comparisons passed. `git diff --check <base> <head>`
and `git show --check --format= <head>` also exited 0 for every repository.

No five-showcase-repository range changes:

- a GitHub Actions workflow, action pin or permissions block;
- a dependency, lockfile, package manifest or supported runtime;
- a badge, README claim, public product claim or repository metadata;
- `SECURITY.md`, a release workflow, `RELEASING.md` or release policy;
- application/runtime behaviour, skill content, validation data or schemas;
- a raw Register capture, retained response, evidence bundle, client file or
  generated build/distribution output.

The build checks created only ignored local build outputs. No generated output
was added to or changed in a reviewed commit, and every tracked worktree
remained clean after the checks.

The authorised toolkit follow-up changes only the scanner, its regression
coverage, this audit and the Task 1 plan interface. It changes no workflow,
dependency, lockfile, badge, release or security policy, showcase runtime,
public product claim, raw capture, client data or tracked generated artifact.

The three known Windows defect seams were checked directly with
`git diff --quiet <base> <head> -- <affected paths>` and all exited 0:

- Ozzit: `tools/postbuild/help_corrections.py`,
  `tools/tests/postbuild/test_help_corrections.py`, `src/`, and `ozzit.xlsx`;
- tax corpus: `fadden/export_live_evidence_bundles.py`,
  `tests/corpus/test_live_evidence_bundle_export.py`, and
  `tests/corpus/test_publication_bundle_export.py`;
- Monthly Close Controls: `closecontrol/viewer.py` and
  `tests/test_viewer.py`.

## Ozzit verification

These are the exact scalar commands in `.github/workflows/verify.yml`:

```powershell
python -m pip install "mypy==2.3.1"
python -m mypy --config-file mypy.ini
python tools/verify_workbook.py ozzit.xlsx
python tools/verify_sources.py ozzit.xlsx src
python tools/verify_signatures.py src
python tools/verify_previous_names.py functions.csv
python tools/verify_index.py ozzit.xlsx src functions.csv
python tools/verify_afe.py ozzit.xlsx src
python -m unittest discover -s tools/tests -v
```

The install and seven non-test gates exited 0: mypy found no issues in 22
source files; workbook verification found 134 functions and 211 parts; source,
signature, previous-name, index and AFE checks all completed successfully.

The full unittest command exited 1 after 175 tests with 10 failures and 1
error. All are the retained Windows `HelpCorrectionsTests` behaviour: the
fixture rewrites LF-controlled multiline source anchors through Windows text
translation and the existing postbuild pass then finds zero anchors. This is
not reported as a passing workflow.

The focused new-policy command passed:

```powershell
python -m unittest tools.tests.test_repository_policy -v
```

Result: 10 passed. The new test derives the current workflow commands and
checks both contributor guides. The failing postbuild implementation, fixture,
workbook and source paths are unchanged from the base, so the guidance range
does not worsen the known defect. The local scalar commands used Python 3.11.9
while hosted CI selects Python 3.12; that environment difference remains a
residual limitation.

## Australian Accounting Skills verification and change rationale

This repository was not a no-op. At the untouched base, all existing gates
passed, but `AGENTS.md` delegated the substantive guide to `CLAUDE.md` and
`CLAUDE.md` did not import `@AGENTS.md`. That directly contradicted the approved
cross-runtime shared-entry contract. A focused test first proved the drift, and
the final change moved the existing substantive guidance into `AGENTS.md`,
made `CLAUDE.md` the one-line import, and changed no skill or runtime content.

The fresh focused command was:

```powershell
python -m unittest tests.test_contributor_checks -v
```

Result: 2 passed. The complete requested sequence was:

```powershell
python -m pip install --disable-pip-version-check --no-deps --requirement requirements-test.txt
python -m unittest discover -s tests -v
python scripts/validate_validation.py
python tests/verify_skills_cli.py
git diff --check
```

Results: the pinned dependency was already satisfied; 63 tests passed with 1
environment-dependent symlink skip; validation confirmed 17 fabricated cards
and 19 skills; Skills CLI 1.5.22 discovered all 19 expected skills; the diff
check exited 0. All gates therefore pass at the reviewed head. The exact range
contains only the two entry guides and their contributor-contract test.

## Aus Accounting MCP verification

The CI gates are exactly:

```powershell
python -m pip install "uv==0.12.0"
uv run --locked --extra dev pytest -q
uv run --locked --extra dev ruff check aus_accounting_mcp tests
uv run --locked --extra dev mypy aus_accounting_mcp
```

Results: the pinned Python package was already satisfied; 72 tests passed;
Ruff passed; mypy found no issues in 10 source files. The focused new-guidance
suite also passed 4 tests:

```powershell
uv run --locked --extra dev pytest -q tests/test_repository_guidance.py
```

The following commands are supplementary local or release-readiness checks,
not CI gates:

```powershell
uv lock --check
uv sync --locked --extra dev
uv run --locked --extra dev python -m build
uv run --locked aus-accounting-mcp-demo
uv run --locked --extra dev pytest -q tests/test_demo.py tests/test_demo_media.py tests/test_compatibility.py tests/test_engine_versions.py
```

Results: lock and sync checks exited 0; the build produced the 0.1.6 sdist and
wheel; the fabricated demo emitted both the synthetic BAS proof and
`ERR_POLICY_DIV7A_REFUSED`; the focused demo/media/compatibility group passed
10 tests. No outside-checkout wheel command exists in this repository's
documented CI or guidance, so one is not added or represented as a gate here.

On this host, PATH-selected `uv` was 0.12.6 while `python -m uv` was the
workflow-pinned 0.12.0. The exact documented `uv` command forms used the PATH
executable; the version distinction is retained rather than described as an
exact hosted-CI reproduction.

## Tax legislation corpus verification

The workflow-defined compile, test, build, lint and type-check commands are:

```powershell
python -m compileall -q .
python -m unittest discover -s tests/corpus -t . -v
python -m pip install "uv==0.12.0"
uv run --locked --extra dev pytest tests
uv run --locked --extra dev pytest tests/radar
uv run --locked --extra dev --python 3.12 python -m build
uv run --locked --extra dev ruff check tax_radar_au tests
uv run --locked --extra dev mypy tax_radar_au
```

CI scope matters: compile and corpus unittest run on Ubuntu; locked full pytest
runs on Ubuntu for Python 3.10 through 3.13; Windows 3.12 runs only
`tests/radar`; package, Ruff and mypy jobs run on Ubuntu. The Task 7 Windows
checks used `UV_PYTHON=3.12` where the task report or workflow required it.

Fresh results:

- compileall exited 0;
- the focused guidance suite passed 5 tests;
- the supported Windows radar leg passed 193 tests;
- Ruff passed and mypy found no issues in 7 source files;
- the locked build produced the 0.1.3 sdist and wheel;
- corpus unittest exited 1 after 274 tests with 11 errors and 8 skips;
- the required local full locked suite exited 1 with 451 passed, 11 failed,
  8 skipped and 238 subtests passed.

The full Windows suite used the task-report-safe temporary base:

```powershell
$env:UV_PYTHON='3.12'
$task7FixPytest = Join-Path ([System.IO.Path]::GetTempPath()) ("tax-radar-task7-fix-" + [guid]::NewGuid().ToString("N"))
uv run --locked --extra dev pytest tests --basetemp $task7FixPytest
```

Both red commands reach the unchanged live-evidence promotion seam and surface
the existing `handle-bound rename failed` condition. They are not described as
green. The guidance test itself is included in the passing radar leg.

The Windows-equivalent installed-wheel smoke from `AGENTS.md` was also run from
a fresh system temporary directory outside the checkout. It installed the
locally built wheel with `--no-index`, resolved packaged samples from the
temporary venv, returned `REVIEW_REQUIRED` with one item from `compare`, and
returned `DECISION_RECORDED` with one decision from `validate-review`; both
commands exited 0. No capture or publication workflow was run.

## Monthly Close Controls verification

The exact scalar CI gates are:

```powershell
python -m pip install "uv==0.12.0"
uv run --locked --extra dev pytest -q
uv run --locked --extra dev --python 3.12 python -m build
uv run --locked --extra dev ruff check closecontrol tests
uv run --locked --extra dev mypy closecontrol
```

The focused guidance and release-workflow contract commands were also run:

```powershell
uv run --locked --extra dev pytest -q tests/test_repository_guidance.py
uv run --locked --extra dev pytest -q tests/test_release_workflow_hardening.py
```

Results: 4 guidance tests passed; 6 release-workflow tests passed; Ruff passed;
mypy found no issues in 9 source files; the build produced the 0.1.2 sdist and
wheel. The full suite collected 230 tests and exited 1 with 228 passed, 1
skipped and 1 failed. The retained failure is
`test_pack_dir_pointing_at_a_file_fails_closed`: Windows produces the existing
`not found` message where the test expects `could not be read`. The viewer and
its test are unchanged from the base, and the new guidance suite passes.

The installed-wheel smoke ran from a fresh system temporary directory outside
the checkout. The installed module resolved from that venv's `site-packages`,
the fabricated demo returned the required `REVIEW` exit 2, and
`pack/close-review-pack.json` existed. This is a package check, not a
publication action.

## Toolkit verification

The false positive was reproduced before the production edit. This focused RED
command ran one test and failed because the exact public GitHub REST
user-repositories URL was reported as `private-user-path`:

```powershell
python -m unittest tests.test_public_repository.PublicFileScannerTests.test_allows_public_github_user_url_but_rejects_rooted_private_paths -v
```

Commit `b71afdce82915802f389e6a2fb0ea7e28cff2503` applies the minimal scanner
change: the POSIX user-root alternative now requires a non-alphanumeric root
boundary. It does not exempt HTTP content globally. Its regression matrix
allows only the exact public API URL while continuing to reject Windows user
roots with either separator, POSIX home and root-level user paths, file URIs,
local paths embedded in URL query/path values, and a mixed payload containing
both the public URL and a real local path. The focused GREEN command passed:

```powershell
python -m unittest tests.test_public_repository.PublicFileScannerTests.test_allows_public_github_url_and_rejects_unsafe_content_without_echoing_it -v
```

Commit `387a720c54c6f463ba1bec8965d329c9ad62a6c3` separately amends the Task 1
plan interface. It requires the complete currently observable public inventory
and an authenticated private aggregate only when available; it forbids assuming
a fixed total and forbids publishing private rows. The existing account audit
and JSON retain their historical 42-to-39 reconciliation unchanged.

The public repository's complete local verification set was run after both
commits and again after this audit amendment was staged so the scanner included
it:

```powershell
python -m unittest discover -s tests -v
python -m unittest tests.test_public_repository -v
python scripts/validate_skills.py
python scripts/validate_skills.py --strict
pwsh -File scripts/sync-skills.ps1 -Check
python scripts/check_public_files.py
git diff --cached --check
```

Final results: the complete unit suite passed 35 tests; the public-repository
module passed 14 tests; incremental and strict validation, sync-drift, direct
public-file scanning and staged diff checks all exited 0. The prior false
positive was introduced by Task 1 commit
`f61b9c61f7a91cf74578afc4e2297a014607e2ba` relative to showcase base
`11fc9e4ef26d37595f42574f4d96643e309cacc4`, not inherited from the
pre-showcase baseline. The focused regression and full public-file gate now
prove it is fixed without weakening rejection of private paths.

## Public account inventory discrepancy

The preserved Task 1 evidence records a historical expectation of 42 followed
by 39 unique repositories returned by the unauthenticated public endpoint. No
authenticated private aggregate was available. The difference of three is
unresolved and is not attributed to private repositories. No private repository
name, URL, path, metadata or configuration is present in the public inventory
or this review.

The amended plan no longer treats 42 as a required input or assumes any fixed
account total. Its publication unit is the complete public inventory observable
at collection time; a private count is an aggregate action input only when
authenticated evidence is available and private rows are never public. The
retained 39-public/unknown-private boundary is a residual limitation on any
account-wide completeness claim, but does not change the exact five-repository
hardening set.

## Residual risks and stop-before-push boundary

- Ozzit's complete Windows unittest command remains red with 10 failures and 1
  error; only its focused policy and seven non-test workflow gates are green.
- The corpus complete Windows suites remain red at the counts above; its
  supported Windows radar CI surface and all focused/new checks are green.
- Monthly Close Controls' complete Windows suite remains red by one wording
  assertion; its focused, release, build, lint, type and wheel checks are green.
- Local Windows checks do not prove current hosted CI state on Ubuntu or the
  current remote default-branch ancestry.
- The PATH `uv` 0.12.6 versus installed Python-module `uv` 0.12.0 distinction
  prevents a claim that every local `uv` process used the hosted pin.
- The saved public account inventory contains 39 observed records. The current
  private aggregate is unknown, and public/private account completeness must be
  confirmed from then-current evidence at action time; no fixed total may be
  assumed.

No push, publication, release, tag, remote change, repository creation or
GitHub setting mutation has been performed. The locally verified publication
set stops here. A later action-time handoff must present the five exact reviewed
heads in this file, confirm then-current remote ancestry and hosted state, use
then-current observable account evidence, and include the final audit-only
toolkit commit recorded in the Task 7 report before any external write.
