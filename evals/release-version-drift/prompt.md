---
name: release-version-drift
tags: [activation]
max_turns: 4
timeout_seconds: 240
allowed_tools: [Skill, Read, Glob, Grep]
---

package.json says 1.4.0, the latest git tag is v1.3.2 and CHANGELOG.md has a 1.5.0 heading. Sort out what the next version should be before I tag it.
