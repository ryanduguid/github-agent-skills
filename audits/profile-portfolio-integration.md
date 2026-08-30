# Profile and portfolio integration audit

Date: 31 August 2026

## Verdict

The local publication set is coherent and ready only for an action-time
confirmation. No external action was taken. The profile remains
accounting-first and unchanged; the site remains adoption-first and adds a
bounded fourth adoption route for the portable skills. The new repository URL
is not currently public: a read-only request to
`https://github.com/ryanduguid/github-agent-skills` returned HTTP 404. It is
therefore valid only as part of the same pending publication wave as the local
toolkit content, not as an independently resolving public link today.

## Fixed reviewed checkpoints

| Surface | Local branch | Reviewed checkpoint | Result |
| --- | --- | --- | --- |
| Profile `ryanduguid` | `codex/profile-agent-skills` | `b45ca45aa753e4297b73f6d9528f3c2f1f5f600e` | Unchanged no-op. The profile has no new toolkit link or repository commit. |
| Website `ryanduguid.github.io` | `codex/site-agent-skills` | `93d51f77b34e27e985cfc3785d3f7e15f1b7fe64` | Adds the fourth adoption route and completes its browser contract with current desktop/mobile baselines, a passing evidence-based mobile height guard and a fixed visual-diff allowance. |
| Toolkit `github-agent-skills` content | `codex/github-agent-skills` | `fbb28060c0ededb00a6206e1fc8c507e17c27cdb` | The locally verified content parent included in the pending wave. |

The toolkit checkpoint above deliberately names the content parent, not this
amendment's resulting commit. A Git commit object cannot contain its own final
object ID because the file contents determine that ID. The prior audit commits
are `713907317e3edacbae009ddf7c93e60ed875d20b` and
`63187599c9c35abaf5c1a364127a762cae762e2d`; this amendment's final commit is
recorded in the SDD task report. The external candidate is that ordered audit
chain on top of `fbb28060c0ededb00a6206e1fc8c507e17c27cdb`.

The former website checkpoint
`fcc38c6aca97a774d6702f135cb1c643043653e0` remains reviewed history for the
adoption-route terminology correction, but it is superseded for publication:
its 9,538px mobile guard could not accept the final 9,726px render, and its
desktop baseline still showed the former heading.

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
  `4a29bdcfc469a73ff8cd9cec449fe8b5ea5f8c3d..93d51f77b34e27e985cfc3785d3f7e15f1b7fe64`
  changes only the adoption copy, contracts, protected `llms.txt` digest, and
  justified homepage browser contracts and baselines.
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
| `npm ci` | Passed: 331 packages added and 332 audited; lockfile unchanged. npm repeated seven existing audit findings: one moderate and six high. It also reported four deprecated transitive packages. |
| `npm run test:capture` | Passed: 2 tests. |
| `npm run test:browser` | Passed from a clear port: 70 tests, 4 skipped. The final homepage renders were 1,440x6,710 desktop and 390x9,726 mobile; both current baselines were visually inspected. The 9,960px mobile cap is the 9,278px pre-route render plus the complete 448px adoption-route delta and the original 234px guard margin. |
| `npm run test:lighthouse` | Passed with final exit code 0. Pretest passed 5 of 5 tests; Lighthouse completed 3 runs for each of 4 URLs, processed 12 results and wrote 12 local reports. |
| `git diff --check` and `git diff --exit-code` | Passed: clean worktree. |

Before the focused browser diagnosis, `127.0.0.1:4173` had no listener. The
repository-supported `python -u scripts/serve_site.py` command was started in a
dedicated execution session; its sole listener was identified as owned PID
14024 from that exact command line and parent session. After the focused tests,
the identity was checked again, only PID 14024 was stopped, and the port was
confirmed clear. The port was also confirmed clear before the final Playwright
browser suite and Lighthouse collector, so their configured servers could not
reuse a transient process. The earlier browser result at the superseded
checkpoint did not establish that ownership and is not used as final evidence.

The representative Lighthouse results from the final 12-run median set were:

| Route | Performance | Accessibility | Best practices | SEO | LCP | TBT | CLS |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| `/` | 98 | 100 | 100 | 100 | 1,953 ms | 0 ms | 0 |
| `/tools/` | 98 | 100 | 100 | 100 | 1,802 ms | 0 ms | 0 |
| `/evidence/` | 97 | 100 | 100 | 100 | 2,102 ms | 0 ms | 0 |
| `/tools/coal-lsl-levy/` | 98 | 100 | 100 | 100 | 1,953 ms | 0 ms | 0 |

### Toolkit publication set

The toolkit audit is itself part of the candidate publication set, so the
README Quick start checks and the tracked public-file scan were rerun against
this exact audit-only amendment immediately before commit.

| Command | Result |
| --- | --- |
| `pwsh -File scripts/sync-skills.ps1` | Passed with no generated-copy changes. |
| `python -m unittest discover -s tests -v` | Passed: 35 tests. |
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
3. Confirm the complete toolkit candidate: content parent
   `fbb28060c0ededb00a6206e1fc8c507e17c27cdb`, prior audit commits
   `713907317e3edacbae009ddf7c93e60ed875d20b` and
   `63187599c9c35abaf5c1a364127a762cae762e2d`, and this amendment's final
   commit recorded in the SDD task report. Publish that final local candidate
   from `codex/github-agent-skills` to public `main`.
4. Verify the public repository URL and its public README resolve after the
   toolkit publication. Do not treat the current HTTP 404 as a valid public
   destination.
5. Only after step 4 succeeds, publish or deploy
   `ryanduguid.github.io`, branch `codex/site-agent-skills`, checkpoint
   `93d51f77b34e27e985cfc3785d3f7e15f1b7fe64`.

No push, repository creation, deployment, metadata edit, pin edit, OAuth
consent or browser-side public action occurred during this audit.
