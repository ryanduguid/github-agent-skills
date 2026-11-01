---
name: audit-before-adopting
tags: [activation]
max_turns: 4
timeout_seconds: 240
allowed_tools: [Skill, Read, Glob, Grep]
---

We might depend on an open source library for parsing bank statements. Its repository has 1,200 stars, signed releases, Dependabot, a SECURITY.md and 70% test coverage, but the maintainer has not replied to issues for 5 months. How far can we trust it?
