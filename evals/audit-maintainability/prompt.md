---
name: audit-maintainability
tags: [activation]
max_turns: 4
timeout_seconds: 240
allowed_tools: [Skill, Read, Glob, Grep]
---

Assess whether this GitHub repository is maintainable: a Python CLI, 2,300 lines in one file, 3 open issues older than a year, a single contributor, CI that runs only flake8, and no tagged releases. I only have this summary, not the code.
