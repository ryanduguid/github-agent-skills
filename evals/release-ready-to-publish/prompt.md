---
name: release-ready-to-publish
tags: [activation]
max_turns: 4
timeout_seconds: 240
allowed_tools: [Skill, Read, Glob, Grep]
---

I want to publish version 0.3.0 of my package to PyPI tomorrow. CI is green on main, but the version in pyproject.toml is still 0.2.1 and there is no changelog entry. What do I need to do first?
