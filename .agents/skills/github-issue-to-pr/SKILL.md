---
name: github-issue-to-pr
description: Use when implementing a GitHub issue and preparing a local commit and pull-request handoff from repository code, tests, and guidance.
---

# Issue-to-PR delivery

Turn an issue into the smallest reviewable local change. An issue is not
permission to refactor adjacent code or publish a branch.

## Establish the local contract

Read repository instructions and relevant source, tests, and current diff
before deciding scope. If reads are denied, use only explicit supplied
evidence; identify the limitation and mark every other fact **unverified**.

Translate the issue into observable acceptance criteria: input or state,
required result, preserved behaviour, and any boundary that needs requester
confirmation. State a bounded assumption when wording is ambiguous; do not
silently expand it. Identify existing uncommitted changes and keep them out of
the issue change unless the issue explicitly covers them.

## Make the narrow change test-first

Before modifying source, add a focused test for each new observable behaviour
and run it to observe the expected failure. Then make the smallest source
change that passes. Preserve unrelated behaviour with existing or focused
tests. Do not combine parser rewrites, formatting sweeps, documentation
rewrites, dependency updates, or cleanup with the issue unless they are needed
to meet a stated criterion.

Run only repository-declared checks plus proportionate focused tests. Record
the exact command and result; if execution is unavailable or fails, say so
instead of inferring success. Review `git diff --check`, the final diff, and
the staged set if preparing a commit. Never revert, stage, or include another
person's unrelated work.

## Return the handoff

Use this order:

1. **Acceptance criteria and assumptions**: observable issue scope and any
   unverified interpretation.
2. **Change and evidence**: files changed, test-first evidence, commands and
   results, preserved unrelated changes, and remaining uncertainty.
3. **Commit summary**: a concise imperative subject and body matching only
   the reviewed local diff. Do not invent a hash, branch, or staged state.
4. **PR body**: `## Summary`, `## Tests`, and `## Assumptions` (when needed),
   with only observed results and a clear scope boundary.
5. **Publication checkpoint**: state the exact local commit and target branch
   that need confirmation before pushing or creating a PR.

Remote push and pull-request creation occur only after the local diff and
commit are reviewed and the requester explicitly confirms that exact commit
and target branch at the action boundary. A broad request to “push” or “open a
PR” is not that confirmation.
