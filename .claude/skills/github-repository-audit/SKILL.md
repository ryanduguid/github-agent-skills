---
name: github-repository-audit
description: Use when assessing whether a GitHub repository is trustworthy, maintainable, presentable, or ready to feature.
---

# Evidence-first repository audit

Assess the supplied local repository or explicit repository evidence without
changing it. An audit is a decision aid, not permission to improve the
repository.

## Inspect, then classify

List relevant files and read the evidence before making a claim. Check the
README, licence, CI configuration and any supplied CI result; search for
contributor guidance before calling it absent. Record the source path or
supplied artifact for every observation.

| Status | Use only when |
| --- | --- |
| **present** | Direct inspection or explicit supplied evidence confirms the stated fact. |
| **missing** | The relevant search completed and found no evidence. |
| **unverified** | Evidence was not supplied, could not be read, is ambiguous, or would require an unobserved remote check. |

A bare path mention is not evidence. Attribute a present claim based on
explicit supplied evidence, but do not infer file contents. A retained CI
result can confirm that result, but never the current remote workflow state.

## Separate controls from judgement

Use deterministic checks for facts they can enforce (file existence, search
results, command exit status). They do not establish presentation quality,
maintainability, or remote GitHub state. Make those judgements only from the
evidence inspected; otherwise mark them **unverified**.

## Report

Return this evidence table, keeping recommendations limited to demonstrated
gaps:

| Area | Observed evidence | Risk | Recommendation | Priority |
| --- | --- | --- | --- | --- |

Do not recommend adding a file already observed as present. If no evidence
supports an edit, end: “No repository change is justified by the inspected
evidence.” When at least one recommendation is evidence-backed, end with the
highest-priority next action instead.

## Audit-only boundary

For an audit-only request, do not create, edit, delete, stage, commit, push,
open a pull request, alter remote settings, or contact public services.
Report possible changes as recommendations; make them only with separate
authorisation.
