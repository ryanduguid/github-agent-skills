# Repository guidance

Keep issue fixes narrow. Add a failing regression test before changing source,
then run this repository's declared check:

```text
python -m unittest discover -s tests -v
```

Review `git diff --check` and the final diff before reporting completion.
This working tree already contains an unrelated migration-note edit; do not
modify, revert, stage, or include it in an issue commit. Do not push, create a
pull request, or contact GitHub unless the requester confirms the exact local
commit and target branch at that action boundary.
