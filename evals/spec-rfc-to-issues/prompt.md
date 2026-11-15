---
name: spec-rfc-to-issues
tags: [activation]
max_turns: 4
timeout_seconds: 240
allowed_tools: [Skill, Read, Glob, Grep]
---

Break our RFC in docs/rfcs/offline-sync.md into GitHub issues in the order we should build them, and make sure nothing the RFC asks for is missed. Don't create them yet.
