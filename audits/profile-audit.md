# Profile audit: portable agent skills

## Verdict

Keep the profile unchanged. `github-agent-skills` does not belong in the
four-project profile index before it is public, and its generic maintenance
workflow focus would dilute the concise Australian accounting narrative.

## Evidence

The current profile statement is: "I'm an accountant in Newcastle, Australia.
I build open-source tools for Australian tax, payroll and financial reporting."
Its four selected projects are, in order:

1. `payday-super-checker` - payroll timing review and exceptions.
2. `xero-trial-balance-export` - reconciled trial-balance export.
3. `Ozzit` - native Excel financial-modelling and GST functions.
4. `accounting-excel-toolkit` - Australian ledger-export utilities.

The preserved live pin order is `xero-trial-balance-export`,
`payday-super-checker`, `aus-accounting-mcp`, then `workpaper-review-gate`.
Together these are accounting-first proof of payroll, reconciliation, tooling
and review workflows. The candidate toolkit is locally verified at
`fbdbe5b68418649d226d1ae524f1fb6f19ac8bdb`, but it is not public. It has no
verified public repository or portfolio destination, so it is ineligible under
the profile-curation criteria and must not receive a public link.

## Constraints followed

- Keep the README at its tested 30-line ceiling and exactly four selected
  projects.
- Preserve Australian English, credential wording, the synthetic-data and
  professional-review boundary, and the canonical `https://duguid.com.au/`
  catalogue route.
- Do not add a pin, remove a pin, save or reorder the existing pins.
- Do not add `CONTRIBUTING.md`; private account-level community defaults are
  intentional.
- The existing repository-identity suite already covers its current canonical
  repository contract. No new identity test is needed for this no-op.

## Regression decision

`README.md`, `llms.txt`, profile pin state and
`tools/test_repository_identity.py` remain unchanged. Reconsider only once
the toolkit is public and has a verified durable destination; even then, it
should be added only if it strengthens rather than displaces the accounting
proof set.
