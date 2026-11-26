# Porting notes: `pr_precheck.py` to `github-skills pr-precheck`

`skills/github-pr-review/scripts/pr_precheck.py` (269 lines of Python) has a native counterpart, `github-skills pr-precheck`, in `tools/github-skills/src/pr_precheck.rs` and its `pr_precheck/` modules.

## What stays in Python

- **`skills/github-pr-review/scripts/pr_precheck.py`** stays. `skills/github-pr-review/SKILL.md` runs the script installed with the skill folder, with no build step, on machines that may not have Rust; the native command is additive until prebuilt `github-skills` binaries ship. Changing the script also changes the skill's digest, which ties it to its recorded evaluation runs.
- **`.github/ci/check_gates.py`** stays: `validate.yml` runs it directly as a single standard-library check.
- **`tests/`** keeps the Python suite for the retained scripts, plugin packaging, workflow pins and skill authority boundaries.

`scripts/validate_skills.py` and `scripts/check_public_files.py` were ported earlier (`tools/github-skills/README.md`).

## Parity contract

One invocation, `pr-precheck <files.json>`, run in the folder that holds `<files.json>`. These must match the Python script:

- the exit code (`0`, or `2` for every refusal);
- stdout, byte for byte after Windows line endings are normalised: `json.dumps(..., indent=2, ensure_ascii=True)`, the same field order, `[]` for empty lists, and `\u` escapes for control, DEL and non-ASCII characters (UTF-16 surrogate pairs above the BMP);
- the `pr_precheck:` prefix on every error message.

Path confinement must never be bypassed, and hostile input must never panic or exhaust memory.

## How parity is checked

`tools/github-skills/tests/pr_precheck_parity.rs` runs both the Python script and the native command on the same inputs and compares exit codes and stdout, and the stderr prefix on refusals. It covers page layouts (single, concatenated, `--slurp`ed, pretty-printed), every refusal path, one file of each classification, the `PurePosixPath` quirks below, author-chosen control and non-ASCII characters, ties in `largest`, the 3,000-file limit and the usage errors. It runs under `cargo test` on Linux and Windows and needs Python 3.8 or later on `PATH` (or named by `PYTHON`).

`tools/github-skills/tests/pr_precheck.rs` and the unit tests pin the behaviour the Python script cannot show: the divergences below, the `u128` totals and the confinement rules.

## Quirks kept

- Unicode whitespace (as Python's `str.isspace`) is stripped around and between concatenated JSON documents.
- `--slurp`'s array of arrays is unwrapped one level, only when the top-level array is non-empty and every element is an array.
- `PurePosixPath` normalisation of `.` segments and repeated slashes; a hidden name such as `.yml` has no suffix; `is_workflow` tests the raw path for `.github/dependabot.yml` but the normalised path for the `workflows` list.
- `largest` is a stable descending sort on `additions + deletions`, first 10.
- `no_patch` reasons in order: `diff too large`, `rename without changes`, `binary`. `generated` reasons: a generated file name first, then a marker in the first 10 head lines.
- Diff line numbering: `base` and `head` start at `0`; `\ No newline` lines leave both unchanged.
- Python's `$` matches before a final newline, so `a.map\n` counts as generated; each of the 9 workflow markers keeps its lookaround boundaries.

## Intended divergences

1. **Help and usage.** `pr-precheck --help` and `-h` print the `github-skills` usage and exit `0`; the Python script treats `--help` as a missing file. The usage error names `github-skills pr-precheck`.
2. **Confinement before opening.** `.`, `..`, names starting with `-`, NUL bytes and both `/` and `\` are refused on every platform before the file is opened. Exit code, empty stdout and the stderr prefix match.
3. **JSON limits.** `serde_json` follows RFC 8259: it refuses `NaN`, `Infinity`, unpaired surrogates, nesting deeper than 128 and counts outside 64-bit integers, where Python accepts some of them. Totals sum in `u128`.
4. **Error text.** I/O and UTF-8 errors carry Rust's messages after the `pr_precheck:` prefix, not CPython's.
5. **Hunk headers.** Only ASCII digits start a hunk, and a line number beyond 64 bits is refused (exit `2`). Python's `\d` also accepts other Unicode digits and has no size limit; GitHub writes only ASCII hunk headers.
6. **A file named `.`.** Python raises `IndexError` (exit `1`, a traceback) on a path with no parts, such as `.` or `./`; the native command reports no classification for it. GitHub never lists such a path.

## Dependencies

`serde_json` moved from `[dev-dependencies]` to `[dependencies]` in `tools/github-skills/Cargo.toml`. `Cargo.lock` gains no crate and `deny.toml` is unchanged; `fuzz/Cargo.lock` records the same move.

## Checks

The `native` job in `.github/workflows/validate.yml` runs formatting, Clippy, the tests (the parity test included) and the release build on Linux and Windows; the coverage job holds the crate's line-coverage floor.
