---
name: github-readme-polish
description: Use when improving a GitHub README's clarity, quick start, credibility, badges, or project presentation without overstating repository evidence.
---

# Truth-first README polishing

Polish the README into a useful decision page. Improve clarity; do not invent
project maturity.

## Establish the evidence

Read the current README, source, command definitions, workflow files, release
state, and supplied run evidence before proposing claims. If reads are denied,
use only explicit supplied facts and label everything else **unverified**.
Supplied README text can support a review-only diff; a path alone cannot.

| Claim | Required evidence |
| --- | --- |
| Command | The exact script, task definition, or successful supplied run. |
| Badge | A live, relevant endpoint and the workflow or release it represents. |
| Readiness/security | Concrete assessed evidence; otherwise omit it. |

A retained local CI result is not a current GitHub badge, release, deployment,
or production-readiness claim. Never add a badge merely because one would look
professional. Never propose install, build, test, deployment, release, or
security commands not traced to repository evidence.

## Shape the README

Keep an already mature structure when it is clearer. Otherwise use this order:

1. Short answer
2. Proof or demo
3. Requirements
4. Verified quick start
5. Scope and limitations
6. Architecture, only when it helps a reader understand the project
7. Checks
8. Security
9. Licence

Keep specialised legal, safety, regulatory, privacy, or domain disclaimers in
their own boundary and preserve their operative limitations. Do not relabel or
compress one into generic security or marketing copy. Add a plain limitation
when evidence does not justify a maturity claim.

## Review boundary

For an audit or review request, return a concise rationale and a proposed diff
only; do not edit, stage, commit, publish, or contact GitHub. State which
facts are supplied, inspected, or unverified. Make edits only when separately
authorised.
