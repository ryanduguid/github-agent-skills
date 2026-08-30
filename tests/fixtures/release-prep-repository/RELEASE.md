# Example Tool release guide

This repository follows Semantic Versioning. A feature release after `v2.3.0`
is `2.4.0`; use the same release version in `package.json`,
`build-metadata.json`, the changelog heading, generated artefact name, and tag
`v<version>`.

Before any release, run the declared checks:

```text
python -m unittest discover -s tests -v
python -m build
```

Record the command results, verify the generated archive checksum and its
in-toto provenance statement, and compare them with the release version.
Prepare release notes from the matching changelog entry. Creating a remote tag,
GitHub Release, or publication requires explicit authority after readiness is
verified; a successful build is not publication authority.
