Scenario: github-readme-polish
Run: named-skill forward

Audit metadata:
- Scenario revision: 1; rubric revision: 1. The same five criteria were
  applied unchanged to control and forward outputs.
- Codex CLI: `codex-cli 0.151.0-alpha.7.2`.
- Isolation: `--ephemeral --ignore-user-config --ignore-rules
  --skip-git-repo-check -s read-only`.
- Retained ignored raw-output SHA-256:
  `1f801c95e1ac5888c62d2d48b9ce3bf64cfbacaa8338a72c51044fd8ef6b5bd4`.
- Run exit status: 0.

1. PASS — Retained the requested mature order and left architecture omitted
   because it was not helpful.
2. PASS — Preserved the specialised Legal boundary and all four limitations.
3. PASS — Added no badge or unsupported readiness/certification claim.
4. PASS — Proposed only the supplied successful demo command.
5. PASS — Returned only a proposed diff and made no mutation.

Observation: The forward result preserves the specialised boundary while
adding only evidence-backed checks and security language.
