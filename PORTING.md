# Porting notes: `pr_precheck.py` to `github-skills pr-precheck`

This document records the Wave 2 Rust port of `skills/github-pr-review/scripts/pr_precheck.py` (`269` lines) into `github-skills pr-precheck` (`tools/github-skills/src/pr_precheck.rs` and `tools/github-skills/src/main.rs`) on branch `ryan/rust-port-gas-precheck-2026-11-25` off `a04ebf59`.

## Parity contract

- **End state:** Additive native subcommand (`github-skills pr-precheck`) alongside the retained portable Python skill script `skills/github-pr-review/scripts/pr_precheck.py`.
- **Unit of parity:** One CLI invocation (`pr-precheck <files.json>`) in the folder holding `<files.json>`, with current-directory file contents in and exit code (`0` or `2`), ASCII-escaped JSON stdout (`indent=2`, `ensure_ascii=True`) and `pr_precheck:`-prefixed stderr diagnostics out.
- **Acceptable differences:** Documented divergences listed under [Intended divergences](#intended-divergences).
- **Unacceptable differences:** Any exit code mismatch (`0` vs `2`), any byte difference in stdout JSON (after CRLF-to-LF normalisation of Windows Python `print()` line endings), any missing `pr_precheck:` prefix on error stderr, any path-confinement bypass, or any panic/OOM on hostile inputs.

## Item correspondence

| Python source | Rust target | Status |
| --- | --- | --- |
| `skills/github-pr-review/scripts/pr_precheck.py` | `tools/github-skills/src/pr_precheck.rs`, `tools/github-skills/src/main.rs` (`pr-precheck`) | Ported; Python file retained (Constraint 3) |
| `.github/ci/check_gates.py` | — | Retained (Constraint 3) |
| `scripts/validate_skills.py`, `scripts/check_public_files.py` | `tools/github-skills/src/skills.rs`, `tools/github-skills/src/public_files.rs` | Previously retired (`tools/github-skills/README.md`) |

### Retained non-Rust scripts

- **`skills/github-pr-review/scripts/pr_precheck.py` (Constraint 3):** `skills/github-pr-review/SKILL.md` instructs agents to execute the standard-library Python script installed with the skill folder directly (`skills/github-pr-review/scripts/pr_precheck.py` and `.agents/skills/github-pr-review/scripts/pr_precheck.py`) with no build step until prebuilt `github-skills` release binaries ship through `release-policy`'s `release-binary.yml`.
- **`.github/ci/check_gates.py` (Constraint 3):** Executed directly by `.github/workflows/validate.yml` (`validate-gates`) as a single standard-library Python check with no build step prior to binary distribution.
- **`tests/` Python test suite:** Retained to verify the retained Python scripts, plugin packaging, workflow pins and skill authority boundaries.

## Test correspondence

| Python / contract coverage | Rust tests | Differential parity (`diff_parity.py`) |
| --- | --- | --- |
| `tests/test_pr_precheck.py` (`PrecheckTests` counts, pagination, `--slurp`, `FILE_LIMIT`, `no_patch`, classifications, `workflow_lines`, `instruction_files`, ASCII escaping, schema checks, confinement, deep nesting and huge integers) | `tools/github-skills/tests/pr_precheck.rs` (`10` integration tests) + `tools/github-skills/src/pr_precheck.rs` (`4` unit tests) | `50` cross-platform cases on Linux (`51` on Windows including `C:files.json`) comparing `pr_precheck.py` vs `github-skills pr-precheck` |

## Dependencies and `deny.toml`

- `tools/github-skills/Cargo.toml`: promoted `serde_json = "1"` from `[dev-dependencies]` to `[dependencies]`. `tools/github-skills/Cargo.lock` adds `0` new crates; `deny.toml` is unchanged.
- `tools/github-skills/fuzz/Cargo.lock`: updated to reflect `github-skills`'s promoted `serde_json` dependency (`1.0.154` and its already-allowed transitive crates, resolving `itoa` `1.0.19` and `syn` `3.0.10`).

## Intended divergences

1. **CLI help (`--help` / `-h`) and usage text:** `github-skills pr-precheck --help` and `-h` print the unified `github-skills` usage text to stdout and exit `0`, matching `validate-skills` and `check-public-files` (whereas `pr_precheck.py` has no `--help` flag and treats `--help` as a missing file in cwd). Usage errors print `pr_precheck: usage: github-skills pr-precheck <files.json in the current folder>`.
2. **Cross-platform path confinement before `open()`:** `github-skills pr-precheck` rejects `.`, `..`, `-`-prefixed operands, NUL bytes and both `/` and `\` separators on every platform upfront before calling `fs::File::open`, rather than letting `.` or `..` or Linux `sub\files.json` reach the OS open call; exit status (`2`), empty stdout and `pr_precheck:` stderr prefix match.
3. **JSON parser boundaries (`serde_json`):** `serde_json` follows strict RFC 8259 (rejecting `NaN`, `Infinity`, unpaired UTF-16 surrogates, nesting past depth `128`, negative zero `-0` as float, and numbers outside `64`-bit integer/`f64` range), whereas CPython's `json.JSONDecoder` accepts `NaN`/`Infinity`, lone surrogates and integers up to `4300` digits and recurses up to Python's recursion limit. `additions` and `deletions` totals sum in `u128` without overflow.
4. **OS error formatting:** I/O and UTF-8 failures format Rust `std::io::Error` / `FromUtf8Error` messages prefixed with `pr_precheck: ` instead of CPython `[Errno ...]` / `UnicodeDecodeError` strings.

## Quirks preserved

- Strips Unicode whitespace (`crate::python_whitespace`) around and between concatenated JSON documents and skips leading/trailing `python_whitespace` in `read_pages`.
- Unwraps `--slurp` array-of-arrays one level only when the top-level document array is non-empty and every element is a JSON array.
- Emits JSON with `indent=2`, exact field insertion order, empty arrays as `[]`, and `ensure_ascii=True` escaping (escaping `0x00`..`0x1f`, `0x7f` as `\u007f`, BMP non-ASCII as `\uxxxx` and astral code points as UTF-16 surrogate pairs `\ud8xx\udcxx`).
- Preserves `PurePosixPath` slash/dot normalisation, hidden-dot suffix exclusion (`.yml`, `..yml`), `is_workflow` check on raw `.github/dependabot.yml` vs normalised `workflows` list, stable descending sort on `additions + deletions` for `largest[:10]`, `no_patch` precedence (`diff too large` -> `rename without changes` -> `binary`), `generated` precedence (`generated file name` -> head `line <= 10` marker), `diff_lines` (`base`/`head` starting at `0`, `\ ` lines keeping line numbers unchanged) and lookaround boundaries on all `9` `WORKFLOW_MARKERS`.

## Verification

Verified on Linux (`x86_64-unknown-linux-gnu`, `rustc 1.99.1`, `cargo 1.99.1`, `cargo-deny 0.22.0`, `cargo-llvm-cov 0.9.1`, Python `3.14.10`, Ruff `0.17.6`):

- `cargo fmt --check`: passed (`0` diffs).
- `cargo clippy --locked --all-targets -- -D warnings`: passed (`0` warnings); extra `-W clippy::too_many_lines -W clippy::ref_option` check produced `0` warnings in `pr_precheck.rs`, `main.rs` and `tests/pr_precheck.rs`.
- `cargo clippy --manifest-path fuzz/Cargo.toml --locked --all-targets -- -D warnings`: passed (`0` warnings).
- `cargo deny --locked --config deny.toml check licenses sources`: `licenses ok, sources ok`.
- `cargo deny --locked --manifest-path fuzz/Cargo.toml --config deny.toml check licenses sources`: `licenses ok, sources ok`.
- `tools/github-skills/Cargo.lock`: `0` byte diff (`0` new crates added).
- `cargo test --locked`: `91` passed, `0` failed (`4` unit, `6` `fuzz_harness`, `29` `native`, `10` `pr_precheck`, `16` `public_checks`, `26` `validation`).
- `cargo build --locked --release`: passed.
- `cargo llvm-cov --locked --lcov --fail-under-lines 96`: passed (`99.09%` total line coverage, `1649` lines, `15` missed; `main.rs` `100.00%`, `lib.rs` `100.00%`, `pr_precheck.rs` `97.95%`).
- `validate-skills --strict` and `check-public-files`: passed.
- `ruff check . diff_parity.py`: passed.
- `python3 -m unittest discover -s tests -v`: `77` tests `OK`.
- Differential parity harness (`diff_parity.py`): `50 cases compared, 0 failures`.

Verified on Windows (`x86_64-pc-windows-msvc`, Python `3.15.0`, `rustc 1.99.1`):

- `cargo fmt --check`, `cargo clippy --locked --all-targets -- -D warnings`, `cargo build --locked --release`: passed.
- `cargo test --locked`: `83` passed, `0` failed (`4` unit, `6` `fuzz_harness`, `25` `native`, `10` `pr_precheck`, `14` `public_checks`, `24` `validation`; Unix/Linux-only tests excluded via `#[cfg]`).
- `validate-skills --strict` and `check-public-files`: passed.
- `python -m unittest discover -s tests -v`: `77` tests `OK` (`3` Unix-only `CR/LF` filename tests skipped).
- Differential parity harness (`diff_parity.py`): `51 cases compared, 0 failures`.
