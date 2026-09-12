---
name: github-release-prep
description: Use when assessing release readiness, reconciling version drift, preparing release notes, or handing off a GitHub tag, release, or package publication.
---

# Release preparation

Prepare a release decision from repository evidence. Preparation establishes
readiness; it never makes publication authority implicit.

## Read the repository contract first

Read repository-native release, contribution, package, and CI guidance before
choosing a version or a command. Preserve its declared Semantic Versioning
policy and release conventions. If guidance or a required surface cannot be
read, use only explicit supplied evidence and mark the result **unverified**;
do not infer policy, commands, tags, CI state, checksums, signatures, or remote
release state.

## Reconcile the release surfaces

Determine the proposed version from the declared policy, not from a successful
build. Search the relevant repository surfaces and report each one:

| Surface | Required comparison |
| --- | --- |
| Version declarations | Package/manifest, build metadata, release configuration, generated metadata, and any other declared version file. |
| Change record | Matching changelog or release-note entry, retaining its headings and bullets in proposed notes. |
| Git and CI evidence | Relevant local tags, retained build/test results, CI configuration, and their limits. |
| Artefacts | Filename/version, checksum or digest, signature where used, and provenance/attestation where required. |

Classify each surface as **matched**, **drift**, **missing**, or **unverified**.
Call out every mismatch with both path and value. A retained successful build
is historical evidence only: it neither verifies current CI nor proves artefact
integrity, provenance, release state, or authority to publish.

An artefact is not verified merely because it exists. Where the repository
requires a checksum, signature, SBOM, provenance, or attestation, identify the
specific absent or unreadable evidence as a blocker. Do not fabricate it.

## Return a decision-ready handoff

Return these sections in order:

1. **Readiness verdict**: ready only when every required surface matches and
   required verification evidence is present; otherwise state **not ready**.
2. **Release-surface reconciliation**: the status table above, including
   unverified limits.
3. **Exact blockers**: path, observed value/evidence, and the condition that
   clears each blocker.
4. **Proposed version and notes**: the declared SemVer result and the matching
   changelog entry without invented release claims.
5. **Verification commands**: only commands declared by repository guidance;
   record their results separately from the intended remote action.
6. **Publication checkpoint**: after readiness is rechecked, list the remote
   tag, GitHub Release, artefact upload, and registry publication actions that
   still need explicit user authority.

## Authority boundary

Do not create, move, or push a tag; create or edit a GitHub Release; upload an
artefact; publish to a registry; or contact a remote service during release
preparation. Even a request to publish requires a separate explicit checkpoint
after the readiness verdict is ready. Present the proposed remote actions for
approval instead of performing them.
