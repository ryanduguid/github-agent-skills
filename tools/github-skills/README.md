# Repository checks

`github-skills` is this repository's skill validator, synchroniser and
public-file checker. Run it from the repository root:

```shell
cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- validate-skills --strict
cargo run --manifest-path tools/github-skills/Cargo.toml --locked -- check-public-files
```

`rust-toolchain.toml` at the repository root pins the Rust release; with
rustup, `rustup toolchain install` installs it. The executable needs Git for
`check-public-files`; it needs no Python runtime. It replaced
`scripts/validate_skills.py` and `scripts/check_public_files.py`, whose checks
and failure messages it keeps.

## Commands

`validate-skills` checks the discovered approved skill subset. Add `--strict` to
require all 5 canonical skills and exact generated-tree parity. Add `--sync` to
replace the contents of `.agents/skills` from approved canonical skill trees
before validation. Synchronisation copies complete file bytes and nested
directories. It refuses missing or invalid roots and links throughout both trees
before changing generated files. With `--strict`, recorded runs under
`validation/baselines` and `validation/forward` must name an approved skill, a
run that records a skill digest must match the current skill, and
`tests/scenarios` must hold one `.md` scenario per approved skill and no other.
Add `--skill-digest NAME` to print the digest line for a new run file after
strict validation.

Both modes refuse a `SKILL.md` that contains a character which renders as
nothing or reorders the text around it, so a reviewer could read a different
skill from the one an agent loads (Trojan Source, CVE-2021-42574): every
Unicode 18.0 Default_Ignorable_Code_Point, which includes every Bidi_Control
character, the soft hyphen, U+FEFF anywhere and the tag block. The failure
names the first such character's code point, line and column; lines end at LF,
CR or CRLF, and columns count code points from 1. The retired Python validator
had no such check, and it runs before the frontmatter checks. Skill
instructions are plain text, so an emoji built with U+200D ZERO WIDTH JOINER,
U+FE0F VARIATION SELECTOR-16 or tag characters, such as a family emoji or
Scotland's flag, fails too; an emoji without them passes.

Special approved source entries, such as a POSIX FIFO, return a processing error
before mutation, so generated files are preserved when this unsupported input is
present.

`check-public-files` obtains the tracked file list from Git, applies the ordered
public-file rules and checks that the README Repository checks commands match
the single-line `run:` commands of the Validate workflow's `validate` job, in
order, and that the workflow permissions are read-only contents. It reports file
names and rule names without printing matched content. It scans a symlink's
stored target text without opening its target.

Both commands accept `--root PATH`; the default is the current directory. Root
selection does not resolve away a linked repository root. A root operand
beginning with `-` needs an explicit relative or absolute path, such as
`./--strict`, so a missing operand cannot consume a check option.

Success exits 0 and contract failures exit 1. Invalid arguments and input or
processing errors exit 2, with a generic message that prints no file content or
operating system path. Help and version output use stdout; check failures use
stderr.

## Compatibility and limits

The checks keep the Python originals' semantics. The metadata reader preserves
the original line parser, including duplicate-key handling and literal values.
It does not interpret YAML quoting or block values. Text parsing preserves
Python whitespace, line separators, alphanumeric word boundaries and the dotted
and dotless Turkish letters in the fixed case-insensitive patterns. Generated
file comparisons use raw bytes. The scanner recognises BOM UTF-16 and UTF-32 in
either byte order and preserves the original text-versus-binary classification.

Strict tree comparisons retain native path identities, including malformed UTF-8
filenames on POSIX. Diagnostic names use lossy display text, so distinct malformed
names can look alike in an error message; they cannot collapse into one comparison
entry. Recursive Windows traversal follows Python's case-insensitive path ordering.

These are local repository tools. The synchroniser assumes files do not change
during preflight and copying. Copying is not transactional and has no rollback
guarantee. Do not use it as an isolation boundary against a process
concurrently changing the filesystem.

The fixed regular expressions have a 100,000,000-step backtracking budget.
Exhaustion returns a processing error and cannot pass a file. The test corpus
samples Unicode and encoding behaviour and does not prove equivalence for every
Unicode version or malformed filesystem name.

## Verification

From this directory:

```shell
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

The Validate workflow runs these on Linux and Windows, where links are NTFS
junctions, and measures line coverage with `cargo llvm-cov` for Codecov.

`tests/fixtures/public-cases.json` is a frozen scanner oracle: fabricated
payloads, their decoded text and the original Python scanner's results, stored
as byte arrays so the public-file check needs no exemption for them. It was
generated from the retired Python scanner and is now fixed; a deliberate rule
change updates the affected expected results by hand.

`tests/fixtures/hidden-code-points.txt` lists the Unicode 18.0
Default_Ignorable_Code_Point and Bidi_Control ranges, copied from the Unicode
Character Database files its header names. A test requires the hidden-character
check to refuse exactly those code points; a new Unicode version updates the
list and the check together.

[`fuzz/`](fuzz/README.md) holds a cargo-fuzz target for the scanner.

FSL-1.1-ALv2, as the repository. Retain its `LICENSE` and `NOTICE` when
redistributing.
