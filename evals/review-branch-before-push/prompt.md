---
name: review-branch-before-push
tags: [activation]
max_turns: 4
timeout_seconds: 240
allowed_tools: [Skill, Read, Glob, Grep]
---

I've finished the retry back-off change on this branch. Before I push it, look over everything it changes against main and tell me whether anything should hold it back. Don't change any files.
