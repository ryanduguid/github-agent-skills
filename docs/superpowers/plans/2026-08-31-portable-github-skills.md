# Portable GitHub Skills Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build and validate a public repository containing five portable GitHub workflow skills for Codex and Claude Code.

**Architecture:** Keep one canonical copy of each skill under `skills/`, then generate byte-identical runtime copies under `.agents/skills/` and `.claude/skills/` with a Windows-friendly PowerShell synchroniser. Validate structure, copy parity, trigger descriptions, and realistic fresh-agent scenarios before publication.

**Tech Stack:** Agent Skills Markdown, Python 3.11 standard library, PowerShell 7, Codex CLI, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-31-github-agent-system-design.md`

## Global Constraints

- Create exactly five focused skills: `github-repository-audit`, `github-readme-polish`, `github-profile-curator`, `github-release-prep`, and `github-issue-to-pr`.
- Preserve automatic discovery; publication or remote mutation still requires explicit authority at the action boundary.
- Do not require symlinks; Windows clones must work through the checked-in synchroniser.
- Skills must not invent repository facts, badges, commands, releases, or security claims.
- Create and validate one skill completely before starting the next skill.
- No public file may contain credentials, browser state, transcripts, private configuration, or client data.

---

### Task 1: Repository contracts and skill validator

**Files:**
- Create: `GATES.md`
- Create: `AGENTS.md`
- Create: `CLAUDE.md`
- Create: `scripts/validate_skills.py`
- Create: `tests/test_validate_skills.py`
- Create: `tests/fixtures/invalid-skill/SKILL.md`
- Create: `.gitignore`

**Interfaces:**
- Produces: `validate(root: pathlib.Path) -> list[str]`, returning deterministic failure messages.
- Produces: `python scripts/validate_skills.py`, exiting `0` only when all canonical and generated skills are valid and identical.

- [ ] **Step 1: Write validator tests before implementation**

Create tests covering: missing YAML delimiters, missing `name`, missing `description`, directory/name mismatch, descriptions not starting with `Use when`, forbidden placeholders, unexpected skill names, and mismatched generated copies.

- [ ] **Step 2: Verify the tests fail for the missing validator**

Run: `python -m unittest tests.test_validate_skills -v`

Expected: FAIL because `scripts.validate_skills` does not exist.

- [ ] **Step 3: Implement the minimal validator**

The validator must parse only the small frontmatter subset needed here, compare the discovered canonical set to the fixed five-name set, scan for `TBD`, `TODO`, `FIXME`, and scaffold markers, and compare generated files with `filecmp.cmp(..., shallow=False)`.

- [ ] **Step 4: Add cross-runtime instructions**

`AGENTS.md` must identify `skills/` as canonical, require `scripts/sync-skills.ps1` after every skill edit, and list the verification command. `CLAUDE.md` must contain `@AGENTS.md` followed only by the Claude-specific instruction to use `.claude/skills/` after synchronisation.

- [ ] **Step 5: Run the validator tests**

Run: `python -m unittest tests.test_validate_skills -v`

Expected: all tests pass.

- [ ] **Step 6: Commit**

Run:

```powershell
git add GATES.md AGENTS.md CLAUDE.md .gitignore scripts/validate_skills.py tests
git commit -m "test: establish portable skill repository contracts"
```

### Task 2: Repository audit skill

**Files:**
- Create: `tests/scenarios/github-repository-audit.md`
- Create: `validation/baselines/github-repository-audit.txt`
- Create: `validation/forward/github-repository-audit.txt`
- Create: `skills/github-repository-audit/SKILL.md`

**Interfaces:**
- Consumes: a local repository path or explicitly supplied repository evidence.
- Produces: an evidence table with `Area`, `Observed evidence`, `Risk`, `Recommendation`, and `Priority`, followed by a no-change conclusion when no justified edit exists.

- [ ] **Step 1: Write the baseline scenario**

The scenario supplies a fixture repository with a README, passing CI, a licence, and no contributor guide. It asks for an audit without authorising edits. Acceptance requires evidence-backed findings, no invented workflow status, no file writes, and no recommendation to add files already present.

- [ ] **Step 2: Run a fresh control without the skill**

Run:

```powershell
codex exec --ephemeral --ignore-user-config --ignore-rules --skip-git-repo-check -s read-only -C tests/fixtures/sample-repository -o validation/baselines/github-repository-audit.txt (Get-Content -Raw tests/scenarios/github-repository-audit.md)
```

Expected: capture the unassisted result and record at least one observable omission or stop authoring this skill if the control already satisfies every acceptance criterion.

- [ ] **Step 3: Create the minimal skill**

Frontmatter name must be `github-repository-audit`; description must start `Use when assessing whether a GitHub repository is trustworthy, maintainable, presentable, or ready to feature.` The body must require inspection before judgement, separate deterministic enforcement from agent guidance, classify evidence as present/missing/unverified, and prohibit mutation during an audit-only request.

- [ ] **Step 4: Validate and run the forward scenario**

Run `python scripts/validate_skills.py`, then rerun the scenario from the repository root with `$github-repository-audit` explicitly named and save the result under `validation/forward/`.

Expected: every acceptance criterion passes and the result is materially more reliable than the control.

- [ ] **Step 5: Commit**

Run: `git add skills tests/scenarios validation && git commit -m "feat: add evidence-first repository audit skill"`

### Task 3: README polishing skill

**Files:**
- Create: `tests/scenarios/github-readme-polish.md`
- Create: `validation/baselines/github-readme-polish.txt`
- Create: `validation/forward/github-readme-polish.txt`
- Create: `skills/github-readme-polish/SKILL.md`

**Interfaces:**
- Consumes: repository source, verified commands, existing README, and existing release/workflow state.
- Produces: a truthful README diff preserving provenance and professional boundaries.

- [ ] **Step 1: Create and run the no-skill scenario**

Use a fixture where the tempting response would add a nonexistent CI badge, claim production readiness, and replace a specialised legal disclaimer. Save the fresh-agent control output.

- [ ] **Step 2: Create the skill after observing failure**

The fixed output contract is: short answer, proof/demo, requirements, verified quick start, scope and limitations, architecture only when helpful, checks, security, licence. The skill must preserve specialised disclaimers, use only verified commands, add only live badges, and keep mature README structure when it is already clearer than the default recipe.

- [ ] **Step 3: Validate and forward-test**

Expected: no nonexistent badge, no inflated maturity claim, no lost disclaimer, and all proposed commands trace to repository evidence.

- [ ] **Step 4: Commit**

Run: `git add skills tests/scenarios validation && git commit -m "feat: add truth-first README polishing skill"`

### Task 4: Profile curation skill

**Files:**
- Create: `tests/scenarios/github-profile-curator.md`
- Create: `validation/baselines/github-profile-curator.txt`
- Create: `validation/forward/github-profile-curator.txt`
- Create: `skills/github-profile-curator/SKILL.md`

**Interfaces:**
- Consumes: profile README, public repository inventory, pin state when supplied, and portfolio links.
- Produces: one positioning statement, a 4-6 project selection, evidence for each choice, and proposed profile copy without changing pins unless requested.

- [ ] **Step 1: Run a baseline against a mixed original/fork/archive inventory**

Acceptance requires forks, archives, and private repositories to be excluded from showcase recommendations unless the user explicitly chooses them; recommendations must optimise narrative coherence rather than star counts.

- [ ] **Step 2: Write the minimal skill**

Require a single coherent professional narrative, verified link targets, concise copy, and respect for an existing maintained profile runbook. Vanity widgets and decorative activity graphs are excluded unless they improve a stated user goal.

- [ ] **Step 3: Validate, forward-test, and commit**

Run the validator and scenario, manually compare control and forward output, then commit with `feat: add coherent GitHub profile curation skill`.

### Task 5: Release preparation skill

**Files:**
- Create: `tests/scenarios/github-release-prep.md`
- Create: `validation/baselines/github-release-prep.txt`
- Create: `validation/forward/github-release-prep.txt`
- Create: `skills/github-release-prep/SKILL.md`

**Interfaces:**
- Consumes: version files, changelog/release notes, build metadata, tags, CI configuration, and repository release instructions.
- Produces: readiness verdict, exact blockers, proposed version and notes, verification commands, and a separate publication checkpoint.

- [ ] **Step 1: Run the control on a fixture with version drift and an unverified artefact**

Acceptance requires refusal to tag or publish, exact identification of drift, and separation of local preparation from remote release creation.

- [ ] **Step 2: Write the skill**

The skill must inspect repository-native release guidance first, preserve semantic-version policy, verify artefacts and provenance where present, and never treat a successful build as authority to publish.

- [ ] **Step 3: Validate, forward-test, and commit**

Commit with `feat: add release readiness and handoff skill` only after the forward result passes all criteria.

### Task 6: Issue-to-PR skill

**Files:**
- Create: `tests/scenarios/github-issue-to-pr.md`
- Create: `validation/baselines/github-issue-to-pr.txt`
- Create: `validation/forward/github-issue-to-pr.txt`
- Create: `skills/github-issue-to-pr/SKILL.md`

**Interfaces:**
- Consumes: issue text, repository instructions, current code and tests.
- Produces: acceptance criteria, scoped implementation, evidence, commit summary, and PR body; remote PR creation occurs only when requested.

- [ ] **Step 1: Run a control with an ambiguous issue that tempts unrelated refactoring**

Acceptance requires a narrow scope, explicit assumptions, test-first behaviour, preservation of unrelated changes, and no automatic push.

- [ ] **Step 2: Write the skill**

Require the agent to translate the issue into observable acceptance criteria, inspect repository guidance, execute the smallest defensible change, run proportionate verification, review the diff, and report remaining uncertainty honestly.

- [ ] **Step 3: Validate, forward-test, and commit**

Commit with `feat: add issue-to-PR delivery skill`.

### Task 7: Cross-runtime synchronisation

**Files:**
- Create: `scripts/sync-skills.ps1`
- Create: `tests/test_sync_skills.py`
- Generate: `.agents/skills/*`
- Generate: `.claude/skills/*`

**Interfaces:**
- Produces: `pwsh -File scripts/sync-skills.ps1 -Check` for drift detection.
- Produces: `pwsh -File scripts/sync-skills.ps1` for deterministic regeneration.

- [ ] **Step 1: Write failing parity tests**

Test that both runtime trees contain exactly the canonical five directories and every file is byte-identical to `skills/`.

- [ ] **Step 2: Implement synchronisation**

The script must resolve paths from `$PSScriptRoot`, reject an unexpected destination outside the repository, remove only known generated skill directories, copy canonical directories, and support `-Check` without mutation.

- [ ] **Step 3: Generate, verify, and commit**

Run:

```powershell
pwsh -File scripts/sync-skills.ps1
pwsh -File scripts/sync-skills.ps1 -Check
python -m unittest discover -s tests -v
python scripts/validate_skills.py
git add scripts tests .agents .claude
git commit -m "feat: synchronise skills for Codex and Claude Code"
```

### Task 8: Public repository quality and CI

**Files:**
- Create: `README.md`
- Create: `CONTRIBUTING.md`
- Create: `SECURITY.md`
- Create: `LICENSE`
- Create: `.github/workflows/validate.yml`
- Create: `.github/dependabot.yml`
- Create: `.github/pull_request_template.md`
- Create: `.github/ISSUE_TEMPLATE/bug_report.yml`
- Create: `.github/ISSUE_TEMPLATE/feature_request.yml`

**Interfaces:**
- Produces: a clone-and-run path using Python and PowerShell already present on Windows.

- [ ] **Step 1: Add tests for README commands and public-file safety**

Verify that every fenced command in the quick start exists in the repository and that no tracked text file contains known credential-key patterns or transcript paths.

- [ ] **Step 2: Write repository documentation and templates**

The README must explain the five skills, supported runtimes, sync model, validation, installation choices, and the no-remote-mutation boundary. Issue forms must request reproduction evidence without soliciting secrets or private repository contents.

- [ ] **Step 3: Add least-privilege CI**

The workflow must use read-only contents permission, Python 3.11, PowerShell, `python -m unittest discover -s tests -v`, `python scripts/validate_skills.py`, and `pwsh -File scripts/sync-skills.ps1 -Check`.

- [ ] **Step 4: Run the complete local gate**

Run all three commands plus `git diff --check`.

Expected: all pass with no warnings attributable to repository content.

- [ ] **Step 5: Commit**

Run: `git add . && git commit -m "docs: prepare portable GitHub skills for publication"`

### Task 9: Publication-ready review

**Files:**
- Modify: `GATES.md`
- Create: `audits/publication-review.md`

**Interfaces:**
- Produces: exact commit list and push instructions for the external-write checkpoint.

- [ ] **Step 1: Run the full test and validation suite again**

- [ ] **Step 2: Run `codex exec review --uncommitted` or review the branch diff against the initial design commit**

- [ ] **Step 3: Scan tracked files for secrets and private paths**

- [ ] **Step 4: Record repository name, visibility (`public`), description, topics, default branch, and exact commits proposed for publication**

- [ ] **Step 5: Stop at the publication checkpoint**

Do not create the GitHub repository or push until the user sees the exact publication set and confirms the external write.

