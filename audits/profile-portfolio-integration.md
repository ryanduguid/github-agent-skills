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
| Website `ryanduguid.github.io` | `codex/site-agent-skills` | `fcc38c6aca97a774d6702f135cb1c643043653e0` | Adds the fourth adoption route, URL, bootstrap sequence and protected boundary statement. |
| Toolkit `github-agent-skills` content | `codex/github-agent-skills` | `fbb28060c0ededb00a6206e1fc8c507e17c27cdb` | The locally verified content parent included in the pending wave. |

The toolkit checkpoint above deliberately names the content parent, not this
amendment's resulting commit. A Git commit object cannot contain its own final
object ID because the file contents determine that ID. The prior audit commit
is `713907317e3edacbae009ddf7c93e60ed875d20b`; this amendment's final commit
is recorded in the SDD task report. The external candidate is that ordered
audit pair on top of `fbb28060c0ededb00a6206e1fc8c507e17c27cdb`.

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
  `4a29bdcfc469a73ff8cd9cec449fe8b5ea5f8c3d..fcc38c6aca97a774d6702f135cb1c643043653e0`
  changes only the adoption copy, contracts, protected `llms.txt` digest, and
  justified homepage browser baselines.
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
| `npm run test:browser` | Passed: 70 tests, 4 skipped. The Windows local server logged two client-aborted connections, but Playwright completed successfully. |
| `npm run test:lighthouse` | Passed with final exit code 0. Pretest passed 5 of 5 tests; Lighthouse completed 3 runs for each of 4 URLs, processed 12 results and wrote 12 local reports. |
| `git diff --check` and `git diff --exit-code` | Passed: clean worktree. |

### Toolkit publication set

The toolkit audit is itself part of the candidate publication set, so its
documented Quick start checks were rerun at the content parent before this
audit was added.

| Command | Result |
| --- | --- |
| `pwsh -File scripts/sync-skills.ps1` | Passed with no generated-copy changes. |
| `python -m unittest discover -s tests -v` | Passed: 35 tests. |
| `python scripts/validate_skills.py --strict` | Passed with exit 0. |
| `pwsh -File scripts/sync-skills.ps1 -Check` | Passed with exit 0. |
| `git diff --check` and `git diff --exit-code` | Passed before adding this audit. |

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
   `fbb28060c0ededb00a6206e1fc8c507e17c27cdb`, prior audit
   `713907317e3edacbae009ddf7c93e60ed875d20b`, and this amendment's final
   commit recorded in the SDD task report. Publish that final local candidate
   from `codex/github-agent-skills` to public `main`.
4. Verify the public repository URL and its public README resolve after the
   toolkit publication. Do not treat the current HTTP 404 as a valid public
   destination.
5. Only after step 4 succeeds, publish or deploy
   `ryanduguid.github.io`, branch `codex/site-agent-skills`, checkpoint
   `fcc38c6aca97a774d6702f135cb1c643043653e0`.

No push, repository creation, deployment, metadata edit, pin edit, OAuth
consent or browser-side public action occurred during this audit.
