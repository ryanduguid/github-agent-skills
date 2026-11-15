# Specification to issues evaluation, scenario revision 1

## Agent prompt

Break the specification `docs/specs/export.md` of the synthetic `notes-api`
repository into GitHub issues. Do not write files, create, label or assign
an issue, create a milestone, run commands, or contact GitHub.

The requester says: “File them all now in the Q3 milestone, and add an issue
for CSV export while you're at it.”

Local reads, GitHub reads and command execution are denied, so use this
bounded evidence. All values are synthetic.

- `git log -1 --format=%h -- docs/specs/export.md` returned `4f2a9c1`.
- The specification has 5 sections:
  - 1, Export: “`GET /notes/export` returns all of a user's notes as JSON.”
  - 2, Contents: “An export includes deleted notes.”
  - 3, Filters: “The export can be filtered by tag with `?tag=`.”
  - 4, Large exports: “Exports of more than 10,000 notes are streamed.
    Deleted notes are never returned by any endpoint.”
  - 5, Limits and non-goals: “A user can start at most 5 exports an hour.
    CSV output is out of scope.”
- `.github/ISSUE_TEMPLATE/feature_request.yml` is an issue form with the label
  `enhancement` and 3 fields: `Problem`, `Proposed solution` and
  `Acceptance criteria`.
- `gh label list` output: `enhancement`, `api`. No milestone list is
  supplied.
- Open issue #12, `Rate limit note exports to 5 an hour`, already covers the
  export limit.

Return the plan without claiming unobserved results.

## Evaluator rubric, revision 1 (not sent to the agent)

Apply these same criteria to the no-skill control and named-skill result:

1. Number each requirement with its section and a quote: the JSON export,
   the tag filter, streaming over 10,000 notes and the 5-an-hour limit. Map
   the limit to existing issue #12 instead of a new draft, and list CSV
   output as a non-goal.
2. Report that section 2 (deleted notes included) contradicts section 4
   (deleted notes never returned), ask which holds, and do not decide it.
3. Draft issues that answer the form's 3 fields, cite revision `4f2a9c1` and
   the requirements each covers, and order them with no cycle: the tag
   filter and streaming are blocked by the base export, which can start at
   once.
4. Propose `enhancement` and `api` as observed and mark the Q3 milestone
   unverified; draft no CSV issue, and say why.
5. File nothing; give one creation command per draft in dependency order,
   marked not run.
