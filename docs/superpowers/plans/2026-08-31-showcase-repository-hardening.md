# Showcase Repository Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give five showcase repositories accurate cross-runtime instructions and close only demonstrated contribution or documentation gaps while preserving their mature CI and security controls.

**Architecture:** Apply one evidence-first audit contract across the account, then make isolated repository-specific commits. Each repository receives concise `AGENTS.md` and `CLAUDE.md` only when absent; CI, Dependabot, CodeQL, release workflows, and community files remain unchanged when already adequate.

**Tech Stack:** Markdown, Python 3.10-3.12, unittest/pytest, uv, existing GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-31-github-agent-system-design.md`

## Global Constraints

- Deep-change targets are exactly `Ozzit`, `australian-accounting-skills`, `aus-accounting-mcp`, `au-tax-legislation-corpus`, and `monthly-close-controls`.
- Forks, archives, and private repositories receive no mutations.
- Existing repository commands and runbooks are authoritative; do not normalise unlike projects into one generic template.
- Existing private account-level community-health defaults remain in place; add a local file only when project-specific guidance is demonstrably needed.
- Never broaden tax, legal, financial, lodgement, client-data, or human-review boundaries.

---

### Task 1: Account-wide audit matrix

**Files:**
- Create: `github-agent-skills/audits/2026-08-31-account-repository-audit.md`
- Create: `github-agent-skills/audits/repository-inventory.json`

**Interfaces:**
- Consumes: the 42-repository GitHub inventory plus local evidence from the seven cloned repositories.
- Produces: one row per repository with visibility, original/fork/archive state, purpose, language, maintenance recency, depth of inspection, presentation value, risk, and action.

- [ ] **Step 1: Record the complete inventory without secrets or private contents**

Private repositories may be named with public metadata already visible to the owner, but their contents, URLs that expose private paths, and configuration details must not enter the public audit.

- [ ] **Step 2: Classify each repository**

Use one action from `showcase`, `maintain`, `archive candidate`, `upstream fork`, `private/no public action`, or `needs deeper review`.

- [ ] **Step 3: Prove the mutation set**

The five showcase repositories must be justified by breadth of portfolio signal: Excel modelling, Agent Skills, MCP, provenance-rich data, and deterministic controls.

- [ ] **Step 4: Commit the audit**

Run: `git add audits && git commit -m "docs: audit GitHub portfolio and prioritise showcase repositories"`

### Task 2: Ozzit contributor and agent guidance

**Files:**
- Create: `Ozzit/AGENTS.md`
- Create: `Ozzit/CLAUDE.md`
- Create: `Ozzit/CONTRIBUTING.md`
- Modify: `Ozzit/tools/tests/test_repository_policy.py`

**Interfaces:**
- Produces: verified agent commands for workbook, source, signature, index, AFE, mypy, and unit-test gates.

- [ ] **Step 1: Add failing policy tests**

Require `AGENTS.md` to name the committed workbook as authority, forbid fabricating Excel recalculation evidence, list every command from `.github/workflows/verify.yml`, preserve native-Excel/no-macro scope, and point release work to `RELEASING.md`.

- [ ] **Step 2: Run the focused test and observe failure**

Run: `python -m unittest tools.tests.test_repository_policy -v`

- [ ] **Step 3: Add concise cross-runtime instructions**

`CLAUDE.md` must import `@AGENTS.md`; `CONTRIBUTING.md` must describe source/workbook authority, the verification sequence, and the requirement for Excel-backed cache refresh when cached formula results change.

- [ ] **Step 4: Run Ozzit's complete verification workflow locally**

Run the exact mypy, workbook, source, signature, previous-name, index, AFE, and unittest commands from `.github/workflows/verify.yml`.

- [ ] **Step 5: Commit**

Run: `git add AGENTS.md CLAUDE.md CONTRIBUTING.md tools/tests/test_repository_policy.py && git commit -m "docs: add cross-runtime Ozzit contributor guidance"`

### Task 3: Australian Accounting Skills no-op verification

**Files:**
- Modify only if a failing check proves drift: `australian-accounting-skills/AGENTS.md`
- Modify only if a failing check proves drift: `australian-accounting-skills/CLAUDE.md`
- Modify only if consolidation is required: `australian-accounting-skills/tests/test_contributor_checks.py`
- Create: `github-agent-skills/audits/australian-accounting-skills-review.md`

**Interfaces:**
- Produces: a verified keep/change decision for the repository that already has both instruction files and mature contribution/security/release controls.

- [ ] **Step 1: Run all repository gates without editing**

Run:

```powershell
python -m pip install --disable-pip-version-check --no-deps --requirement requirements-test.txt
python -m unittest discover -s tests -v
python scripts/validate_validation.py
python tests/verify_skills_cli.py
git diff --check
```

- [ ] **Step 2: Record whether Codex and Claude instructions disagree**

If no contradiction or missing command exists, make no repository change. Do not add generic templates already supplied by account defaults.

- [ ] **Step 3: Commit only the audit in `github-agent-skills`**

### Task 4: Aus Accounting MCP agent guidance

**Files:**
- Create: `aus-accounting-mcp/AGENTS.md`
- Create: `aus-accounting-mcp/CLAUDE.md`
- Create: `aus-accounting-mcp/CONTRIBUTING.md`
- Create: `aus-accounting-mcp/tests/test_repository_guidance.py`

**Interfaces:**
- Produces: verified commands for uv sync, tests, ruff, mypy, build and demo checks, with publication routed through existing release workflows.

- [ ] **Step 1: Write failing guidance tests**

Require the MCP facade/delegated-engine boundary, refusal of Division 7A, synthetic-only fixtures, money handling through `aus_accounting_mcp.money`, no invented current rates, and exact CI commands extracted from `.github/workflows/ci.yml`.

- [ ] **Step 2: Add the guidance and contributor file**

`CLAUDE.md` imports `@AGENTS.md`. `CONTRIBUTING.md` describes the adapter boundary, compatibility metadata, demo evidence, and release handoff.

- [ ] **Step 3: Run complete checks**

Run the repository's uv-locked pytest, ruff, mypy, package build, and demo media tests exactly as defined in CI.

- [ ] **Step 4: Commit**

Run: `git add AGENTS.md CLAUDE.md CONTRIBUTING.md tests/test_repository_guidance.py && git commit -m "docs: add cross-runtime MCP contributor guidance"`

### Task 5: Tax legislation corpus agent guidance

**Files:**
- Create: `au-tax-legislation-corpus/AGENTS.md`
- Create: `au-tax-legislation-corpus/CLAUDE.md`
- Create: `au-tax-legislation-corpus/tests/radar/test_repository_guidance.py`

**Interfaces:**
- Produces: a concise map of corpus, radar, live capture, evidence publication, and human-authorisation boundaries.

- [ ] **Step 1: Write failing guidance tests**

Require the instructions to distinguish synthetic monitor contracts from live Register capture, prohibit committing raw live captures, preserve immutable output and provenance rules, state the Windows-only official live-export boundary, and list the documented compile/unittest/pytest gates.

- [ ] **Step 2: Add cross-runtime guidance**

Keep details in existing `BUILD.md`, `RADAR.md`, `RELEASING.md`, and approved design plans rather than duplicating them. `CLAUDE.md` imports `@AGENTS.md`.

- [ ] **Step 3: Run checks and commit**

Run compileall, corpus unittests, full locked pytest, ruff, and mypy as CI defines. Commit with `docs: add cross-runtime corpus guidance`.

### Task 6: Monthly Close Controls agent guidance

**Files:**
- Create: `monthly-close-controls/AGENTS.md`
- Create: `monthly-close-controls/CLAUDE.md`
- Create: `monthly-close-controls/tests/test_repository_guidance.py`

**Interfaces:**
- Produces: exact local commands and hard boundaries for review packs, exit codes, fabricated fixtures, and human acknowledgement.

- [ ] **Step 1: Write failing guidance tests**

Require the instructions to preserve `PASS`, `REVIEW`, and `BLOCKED`; never turn acknowledgement into approval; keep client files outside the checkout; use exact Decimal arithmetic; and route release work through `RELEASING.md` and existing workflows.

- [ ] **Step 2: Add concise instructions**

`CLAUDE.md` imports `@AGENTS.md`; no duplicate contribution file is needed because `CONTRIBUTING.md` already exists.

- [ ] **Step 3: Run checks and commit**

Run the locked pytest, ruff, mypy, build, and release-workflow tests from CI. Commit with `docs: add cross-runtime monthly-close guidance`.

### Task 7: Integrated review and publication set

**Files:**
- Create: `github-agent-skills/audits/showcase-hardening-review.md`

**Interfaces:**
- Produces: exact per-repository commit hashes, tests run, no-op decisions, and remaining risks.

- [ ] **Step 1: Review each repository diff independently**

- [ ] **Step 2: Re-run each repository's complete documented checks from a clean working tree**

- [ ] **Step 3: Confirm no workflow, dependency, badge, release, security policy, or public claim changed without evidence**

- [ ] **Step 4: Confirm the Australian Accounting Skills repository remains a no-op unless its own gates exposed drift**

- [ ] **Step 5: Stop before pushes**

Present the exact commit set for external-write confirmation.

