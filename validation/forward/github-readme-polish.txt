Scenario: github-readme-polish

1. PASS — The proposed README retains the requested order: concise opening,
   proof/demo, requirements, existing verified quick start, scope and
   limitations, checks, security, and licence; architecture was not needed.
2. PASS — It left the specialised `Legal boundary` intact with all four
   supplied limitations.
3. PASS — It added no badge and made no production, release, deployment, or
   security-certification claim; the new security text states that no
   assessment or certification is claimed.
4. PASS — Its only proposed command was the supplied successful
   `pwsh -File scripts/demo.ps1` command.
5. PASS — It returned only a proposed diff and made no repository or remote
   mutation.

Observation: The named-skill evaluator received the exact supplied README and
run evidence, because its read-only shell denied direct local reads. It treated
all facts outside that evidence as unverified.
