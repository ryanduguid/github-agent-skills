# Profile and Portfolio Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate the portable skill collection into Ryan's existing profile and portfolio without weakening their established evidence, accessibility, SEO, or identity contracts.

**Architecture:** Treat `ryanduguid` and `ryanduguid.github.io` as mature, separately tested products. Make a minimal profile change only if the new repository improves the existing concise index, and add the skill collection to the website's adoption path through its existing contract-driven HTML and metadata system.

**Tech Stack:** Markdown, static HTML/CSS, Python unittest/contract checks, Node 22, Playwright, Lighthouse.

**Spec:** `docs/superpowers/specs/2026-08-31-github-agent-system-design.md`

## Global Constraints

- Preserve the profile's 30-line ceiling, Australian English, credential wording, accounting focus, current pin order, and private account-level community-health defaults.
- Do not add `CONTRIBUTING.md` to the profile repository; `docs/MAINTAINING.md` explicitly forbids overriding the private account default.
- Preserve the website's protected legal boundary, design tokens, accessibility contracts, structured data, and canonical host `https://duguid.com.au/`.
- Do not link to `github-agent-skills` until the public repository exists or publication is part of the same confirmed push wave.

---

### Task 1: Profile audit and regression contract

**Files:**
- Modify: `ryanduguid/tools/test_repository_identity.py`
- Modify only if justified by the audit: `ryanduguid/README.md`
- Modify only if README changes: `ryanduguid/llms.txt`
- Create: `github-agent-skills/audits/profile-audit.md`

**Interfaces:**
- Produces: a documented keep/change verdict for the existing four-project profile selection.

- [ ] **Step 1: Write the audit before changing profile copy**

Compare the current four selected projects and four preserved pins to the approved narrative. Record the existing runbook constraints and whether the new generic skill repository belongs on an accounting-focused profile.

- [ ] **Step 2: Prefer a no-op when it is the stronger result**

If the new repository would dilute the concise accounting index, leave `README.md`, pins, and `llms.txt` unchanged and record that conclusion. If it materially improves discoverability, first update the identity test with the exact intended placement and line ceiling.

- [ ] **Step 3: Verify profile checks**

Run:

```powershell
python -m unittest discover -s tools -p "test_*.py" -v
python tools/banner.py --check
python tools/check_links.py
git diff --check
```

- [ ] **Step 4: Commit only evidence-backed changes**

Use `docs: align profile with portable agent skills` when copy changes; otherwise commit the audit only in `github-agent-skills`.

### Task 2: Website adoption contract

**Files:**
- Modify: `ryanduguid.github.io/scripts/site_contracts.py`
- Modify: `ryanduguid.github.io/scripts/test_contracts.py`
- Modify: `ryanduguid.github.io/index.html`
- Modify: `ryanduguid.github.io/llms.txt`
- Modify: `ryanduguid.github.io/docs/agent-tooling.md`

**Interfaces:**
- Produces: a fourth supported adoption route for `github-agent-skills` while preserving the existing MCP and accounting-skill commands.

- [ ] **Step 1: Write failing contract tests**

Add the canonical repository URL and supported install/bootstrap command to `PRIMARY_INSTALL_PATTERNS`. Add a mutation test proving the homepage fails when that command or its explanatory boundary is removed.

- [ ] **Step 2: Run the focused tests and observe failure**

Run: `python scripts/test_contracts.py`

Expected: FAIL because the new adoption route is absent from `index.html`.

- [ ] **Step 3: Add the smallest homepage change**

Extend the existing `Three supported commands` adoption band to four commands, explain that the new repository supplies GitHub maintenance workflows for Codex and Claude Code, and keep the existing fabricated-data/accounting boundary unchanged.

- [ ] **Step 4: Align agent-facing documentation**

Add one concise entry to `llms.txt` and the supported setup to `docs/agent-tooling.md`. Do not duplicate the new repository README.

- [ ] **Step 5: Run static and browser checks**

Run:

```powershell
python scripts/check_site.py
npm ci
npm run test:capture
npm run test:browser
git diff --check
```

Expected: all pass; tracked social cards and screenshots remain unchanged unless a verified visual change requires an explicit update.

- [ ] **Step 6: Commit**

Run: `git add index.html llms.txt docs/agent-tooling.md scripts/site_contracts.py scripts/test_contracts.py && git commit -m "docs: add portable agent skills adoption route"`

### Task 3: Cross-surface link and narrative review

**Files:**
- Create: `github-agent-skills/audits/profile-portfolio-integration.md`

**Interfaces:**
- Produces: exact local commits proposed for `ryanduguid`, `ryanduguid.github.io`, and `github-agent-skills` publication.

- [ ] **Step 1: Confirm every new public link resolves or is in the same pending publication set**

- [ ] **Step 2: Confirm the profile remains accounting-first and the website remains adoption-first**

- [ ] **Step 3: Review diffs for credential drift, unsupported claims, em/en dashes, broken canonical URLs, or new client-data prompts**

- [ ] **Step 4: Re-run both repositories' complete documented checks**

- [ ] **Step 5: Stop before the external publication wave**

Present exact repositories and commits for confirmation before push or browser-side public edits.

