# GitHub Agent System Design

Date: 2026-08-31
Owner: Ryan Duguid

## Objective

Build a portable Codex and Claude Code workflow that improves the public quality, maintainability, and presentation of Ryan's GitHub portfolio without applying unsafe blanket changes across all repositories.

## Scope

### New public repository

Create `github-agent-skills` as the canonical, portable source for five focused Agent Skills:

1. `github-repository-audit`
2. `github-readme-polish`
3. `github-profile-curator`
4. `github-release-prep`
5. `github-issue-to-pr`

Each skill will follow the open Agent Skills format and be usable from both Codex and Claude Code. The repository will include a Windows-friendly synchronization/bootstrap mechanism rather than requiring symlinks.

### Portfolio surfaces

Improve these two presentation repositories:

- `ryanduguid/ryanduguid` (GitHub profile README)
- `ryanduguid/ryanduguid.github.io` (portfolio site)

### Showcase repositories

Audit all public first-party repositories, then make tailored changes only to these five showcase projects:

- `ryanduguid/Ozzit` — Excel LAMBDA and modelling utility
- `ryanduguid/australian-accounting-skills` — domain Agent Skills collection
- `ryanduguid/aus-accounting-mcp` — MCP and AI integration
- `ryanduguid/au-tax-legislation-corpus` — provenance-rich legal data
- `ryanduguid/monthly-close-controls` — deterministic accounting controls

Forks, archives, and private repositories are excluded from mutation. Other first-party public repositories receive an audit report and prioritised recommendations only.

## Shared agent configuration

Each selected repository will use:

- `AGENTS.md` as the concise shared source of repository instructions.
- `CLAUDE.md` importing `@AGENTS.md`, followed only by genuinely Claude-specific guidance.
- Repository-scoped skills only where the workflow is unique to that repository.
- Deterministic CI and security checks as enforcement; agent instructions will not replace merge gates.

Instructions will state verified setup, test, lint, build, and release commands. They will not invent commands or duplicate large documentation sections.

## Repository quality baseline

For each showcase repository, preserve working configuration and add only missing, applicable elements:

- Clear README: problem, audience, proof/demo, installation, usage, limitations, architecture, testing, security, licence.
- CI for the repository's real formatting, lint, test, and build commands.
- Dependabot for actual package ecosystems and GitHub Actions.
- CodeQL default or workflow setup when the languages and repository shape support it.
- `SECURITY.md`, contribution guidance, pull-request template, and issue forms when missing.
- Release notes and versioning guidance where the project publishes artefacts.
- Least-privilege GitHub Actions permissions and pinned or maintained action versions.

No badge will be added unless its target workflow or service exists and is healthy.

## Profile strategy

Position Ryan around a single narrative: open-source Australian computational accounting, combining deterministic controls, evidence-rich tax data, Excel/Power BI tooling, MCP, and reusable agent workflows.

The profile and site will highlight five showcase repositories, explain who each serves, and link to runnable demonstrations or concrete usage examples. Decorative activity widgets and vanity metrics are out of scope unless they improve navigation or trust.

## Skill design and validation

Skills will be created and deployed one at a time. Each will have:

- A precise trigger-only description.
- A concise `SKILL.md` with progressive disclosure.
- Supporting scripts or references only where deterministic behaviour or substantial conditional guidance requires them.
- Baseline scenarios demonstrating the failure or gap without the skill.
- Forward scenarios showing correct application with the skill.
- Structural validation with the bundled skill validator.

The same canonical skill content will be synchronized into Codex's `.agents/skills` and Claude Code's `.claude/skills` discovery paths by a checked-in bootstrap script.

## Delivery sequence

1. Build and validate the portable skills repository.
2. Clone and audit the portfolio, profile, and showcase repositories.
3. Produce an account-wide repository audit and prioritisation matrix.
4. Apply repository-specific agent instructions and quality improvements.
5. Run each repository's existing and added checks locally.
6. Review all diffs for correctness, secrets, misleading claims, and unrelated changes.
7. Commit changes separately per repository with reviewable messages.
8. Create or publish the new skills repository and push selected repository updates only after the final external-write checkpoint.

## Acceptance criteria

- The portable skill repository passes structural and scenario validation.
- Both Codex and Claude Code can discover the shared skills after bootstrap.
- Every selected repository has verified commands documented for agents.
- Added workflows parse successfully and run the repository's real checks.
- No existing user changes are overwritten.
- No secret, credential, transcript, or private configuration enters a public repository.
- The profile and portfolio tell one coherent story and link only to live destinations.
- Every GitHub mutation is represented by a local, reviewable commit before publication.

## External-write boundary

Repository creation, public README/site edits, workflow publication, pushes, and GitHub setting changes are external mutations. Prepare and verify all changes locally first. Immediately before publication, present the exact repositories and commits to be pushed and obtain the action-time confirmation required for browser-side public changes.
