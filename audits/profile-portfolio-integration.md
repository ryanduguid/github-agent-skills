# Profile and portfolio integration audit

Date: 31 August 2026

## Verdict

The reconciled local publication set is coherent and ready only for an
action-time confirmation. No external action was taken. The profile remains
accounting-first and unchanged; the site remains adoption-first and carries a
bounded fourth adoption route for the portable skills. The site is now rebased
onto exact remote `main` checkpoint
`a76c096cdae71fe9e35aabeb67f8b5dcb62bef43` and ends at local checkpoint
`40496693ed6761e8e8b853d28e48234541d916cf`. The old local site head is retained
at recovery ref `codex/backup-site-before-reconcile-20260831`. The new
repository URL is not currently public: a read-only request to
`https://github.com/ryanduguid/github-agent-skills` returned HTTP 404. It is
therefore valid only as part of the same pending publication wave as the local
toolkit content, not as an independently resolving public link today.

## Fixed reviewed checkpoints

| Surface | Local branch | Reviewed checkpoint | Result |
| --- | --- | --- | --- |
| Profile `ryanduguid` | `codex/profile-agent-skills` | `b45ca45aa753e4297b73f6d9528f3c2f1f5f600e` | Unchanged no-op. The profile has no new toolkit link or repository commit. |
| Website `ryanduguid.github.io` | `codex/site-agent-skills` | `40496693ed6761e8e8b853d28e48234541d916cf` | Rebased onto exact remote base `a76c096cdae71fe9e35aabeb67f8b5dcb62bef43`; preserves all upstream identity, contact, dependency and register changes while retaining the bounded fourth adoption route and refreshed strict desktop/mobile visual contracts. |
| Toolkit `github-agent-skills` content | `codex/github-agent-skills` | `9714c70af4d2dba06f0e0cc9d0a223aa23fddd82` | The final locally reviewed toolkit content state immediately before this audit-only reconciliation amendment. |

The toolkit checkpoint above deliberately names the final content state
reviewed immediately before this audit-only amendment, not the amendment's
resulting commit. A Git commit object cannot contain its own final object ID
because the file contents determine that ID. This amendment's final commit is
recorded in the SDD task report. The complete external candidate is that audit
commit on top of `9714c70af4d2dba06f0e0cc9d0a223aa23fddd82`.

The former website checkpoints
`fcc38c6aca97a774d6702f135cb1c643043653e0` and
`93d51f77b34e27e985cfc3785d3f7e15f1b7fe64` remain reviewed local history. The
latter is the exact old head preserved by the recovery ref, but both are
superseded for publication by the rebased site checkpoint above.

## Cross-surface review

- The profile remains a 30-line, four-project Australian accounting index with
  the selected work and pin order unchanged: `xero-trial-balance-export`,
  `payday-super-checker`, `aus-accounting-mcp`, and `workpaper-review-gate`.
  Its canonical catalogue route remains `https://duguid.com.au/`.
- The website remains adoption-first. Its fourth route explains a local clone
  and `sync-skills.ps1` workflow, while its existing calculator and evaluation
  routes remain available for firms that do not run Codex or Claude Code.
- The exact website claim is bounded to GitHub maintenance workflows for Codex
  and Claude Code and explicitly preserves fabricated-data and human-review
  boundaries. It makes no credential, employment, certification, regulatory,
  or client-data claim.
- The changed profile range is empty. The website range
  `a76c096cdae71fe9e35aabeb67f8b5dcb62bef43..40496693ed6761e8e8b853d28e48234541d916cf`
  changes only the adoption copy and documentation, contracts, protected
  `llms.txt` digest, and justified homepage browser contracts and baselines.
  `origin/main...HEAD` contains exactly those nine intended local site files;
  upstream-only identity, contact, package and register paths have no local
  delta.
- Review of the textual website delta found no em dash or en dash. No
  credential wording changed, no canonical `https://duguid.com.au/` route was
  replaced, and no form, client-data prompt, OAuth path, fabricated-data
  boundary or human-review boundary was added or weakened.

## Verification rerun

Commands were run from the stated worktree. A blank `git diff --check` output
means success.

### Profile

| Command | Result |
| --- | --- |
| `python -m unittest discover -s tools -p "test_*.py" -v` | Passed: 58 tests. |
| `python tools/banner.py --check` | Passed: 23 blocks checked, 0 failures. |
| `python tools/check_links.py` | Passed: 28 links across 4 files; the exact hibernated LinkedIn identity URL received the permitted HTTP 999 automation denial. |
| `git diff --check` and `git diff --exit-code` | Passed: clean worktree. |

### Website

| Command | Result |
| --- | --- |
| `python scripts/test_contracts.py` | Passed: 26 design mutations and 49 public-contract mutations. |
| `python scripts/check_site.py` | Passed with exit 0, including contract, local server MIME, design, link-policy, Search Console self-test, SEO and link checks. |
| `npm ci` | Passed: 331 packages added and 332 audited; lockfile unchanged. npm reported six high-severity audit findings and three deprecated transitive packages. |
| `npm run test:capture` | Passed: 2 tests. |
| `npm run test:browser` | Passed from a clear port: 70 tests, 4 skipped. The final homepage renders are 1,440x6,823 desktop and 390x9,882 mobile; both regenerated baselines were visually inspected. The existing strict 9,960px mobile height cap passes with 78px headroom, and the fixed visual-diff allowance remains in force. |
| `npm run test:lighthouse` | Passed with final exit code 0. Pretest passed 5 of 5 tests; Lighthouse completed 3 runs for each of 4 URLs, processed 12 results and wrote 12 local reports. |
| `git diff --check` and `git diff --exit-code` | Passed: clean worktree. |

`127.0.0.1:4173` was confirmed clear before capture, the initial browser run,
the focused snapshot regeneration, the final browser suite and Lighthouse.
Playwright ran with `CI=1`, forcing its configured repository server not to
reuse any existing listener, and the port was confirmed clear after every run.
Lighthouse started the same repository-supported
`python -u scripts/serve_site.py` command and also left the port clear. The
initial combined browser run passed all non-visual tests but failed only the
two deliberately retained upstream homepage baselines: expected mobile
390x9,434 versus actual 390x9,882, and expected desktop 1,440x6,630 versus
actual 1,440x6,823. That evidence justified regenerating exactly those two
baselines; a fresh full browser suite then passed.

The representative Lighthouse results from the final 12-run median set were:

| Route | Performance | Accessibility | Best practices | SEO | LCP | TBT | CLS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `/` | 98 | 100 | 100 | 100 | 1,952 ms | 0 ms | 0 |
| `/tools/` | 98 | 100 | 100 | 100 | 1,953 ms | 0 ms | 0 |
| `/evidence/` | 97 | 100 | 100 | 100 | 2,103 ms | 0 ms | 0 |
| `/tools/coal-lsl-levy/` | 98 | 100 | 100 | 100 | 1,953 ms | 0 ms | 0 |

### Toolkit publication set

The toolkit audit is itself part of the candidate publication set, so the
README Quick start checks and the tracked public-file scan were rerun against
this exact audit-only amendment immediately before commit.

| Command | Result |
| --- | --- |
| `pwsh -File scripts/sync-skills.ps1` | Passed with no generated-copy changes. |
| `python -m unittest discover -s tests -v` | Passed: 35 tests. |
| `python scripts/validate_skills.py` | Passed with exit 0. |
| `python scripts/validate_skills.py --strict` | Passed with exit 0. |
| `pwsh -File scripts/sync-skills.ps1 -Check` | Passed with exit 0. |
| `python scripts/check_public_files.py` | Passed with exit 0; every tracked path was scanned. |
| `git diff --check` and post-commit `git diff --exit-code` | Passed: the amendment was the only toolkit change and the committed worktree was clean. |

## Action-time confirmation required

Stop here. Before any repository creation, push, hosted deployment, GitHub
metadata change, pin change, or browser-side public edit, confirm the exact
local state again:

1. `ryanduguid`, branch `codex/profile-agent-skills`, checkpoint
   `b45ca45aa753e4297b73f6d9528f3c2f1f5f600e`: no profile push, content
   mutation or pin mutation is proposed.
2. Confirm authority to create, or confirm existing ownership of, the public
   `ryanduguid/github-agent-skills` repository. This local toolkit has no
   configured remote today. Only after that confirmation, configure `origin`
   as `https://github.com/ryanduguid/github-agent-skills.git`.
3. Confirm the complete toolkit candidate: final content state
   `9714c70af4d2dba06f0e0cc9d0a223aa23fddd82` and this audit-only
   amendment's final commit recorded in the SDD task report. Publish that
   complete local candidate from `codex/github-agent-skills` to public `main`.
4. Verify the public repository URL and its public README resolve after the
   toolkit publication. Do not treat the current HTTP 404 as a valid public
   destination.
5. Only after step 4 succeeds, publish or deploy
   `ryanduguid.github.io`, branch `codex/site-agent-skills`, checkpoint
   `40496693ed6761e8e8b853d28e48234541d916cf`, which is based on exact fetched
   remote checkpoint `a76c096cdae71fe9e35aabeb67f8b5dcb62bef43`. Keep recovery ref
   `codex/backup-site-before-reconcile-20260831` until the publication decision
   is complete.

The publication boundary remains toolkit first, public toolkit URL and README
verification next, and site second. No profile mutation is part of that wave.

No push, repository creation, deployment, metadata edit, pin edit, OAuth
consent or browser-side public action occurred during this audit.
