# Native repository checks

`github-skills` is a local Rust candidate for this repository's skill validator,
synchroniser and public-file checker. Build it from the repository root:

```shell
cargo build --manifest-path tools/github-skills/Cargo.toml --locked --release
tools/github-skills/target/release/github-skills validate-skills --strict --root .
tools/github-skills/target/release/github-skills check-public-files --root .
```

On Windows, the executable has an `.exe` suffix. Rust 1.99 builds the candidate.
The executable needs Git for `check-public-files`; it needs no Python runtime.
The existing Python commands, skill copies and workflows remain the supported
repository checks whilst native qualification and release decisions are pending.

## Commands

`validate-skills` checks the discovered approved skill subset. Add `--strict` to
require all 5 canonical skills and exact generated-tree parity. Add `--sync` to
replace the contents of `.agents/skills` from approved canonical skill trees
before validation. Synchronisation copies complete file bytes and nested
directories. It refuses missing or invalid roots and links throughout both trees
before changing generated files.

Special approved source entries, such as a POSIX FIFO, return a processing error
before mutation. Python rejects a FIFO later during copying; the native preflight
also preserves generated files when this unsupported input is present.

`check-public-files` obtains the tracked file list from Git, applies the existing
ordered public-file rules and checks the README quick start and workflow
permissions. It reports file names and rule names without printing matched
content. It scans a symlink's stored target text without opening its target.

Both commands accept `--root PATH`; the default is the current directory. This
explicit root replaces the Python scripts' location relative to `__file__` and
lets the native executable run outside the checkout. Root selection does not
resolve away a linked repository root.

A root operand beginning with `-` needs an explicit relative or absolute path,
such as `./--strict`, so a missing operand cannot consume a check option.

Success exits 0 and contract failures exit 1. Invalid arguments and input or
processing errors exit 2. The native command reports a generic processing error
instead of a Python traceback. Help and version output use stdout; check failures
use stderr.

## Compatibility and limits

The metadata reader preserves the original line parser, including duplicate-key
handling and literal values. It does not interpret YAML quoting or block values.
Text parsing preserves Python whitespace, line separators, alphanumeric word
boundaries and the dotted and dotless Turkish letters in the fixed case-insensitive
patterns. Generated file comparisons use raw bytes. The scanner recognises BOM
UTF-16 and UTF-32 in either byte order and preserves the original text-versus-binary
classification.

Strict tree comparisons retain native path identities, including malformed UTF-8
filenames on POSIX. Diagnostic names use lossy display text, so distinct malformed
names can look alike in an error message; they cannot collapse into one comparison
entry. Recursive Windows traversal follows Python's case-insensitive path ordering.

These are local repository tools. Like the original synchroniser, the native
candidate assumes files do not change during preflight and copying. Copying is
not transactional and has no rollback guarantee. Do not use it as an isolation
boundary against a process concurrently changing the filesystem.

The fixed regular expressions have a 100,000,000-step backtracking budget.
Exhaustion returns a processing error and cannot pass a file. This is a native
resource limit; Python has no corresponding explicit budget. The qualification
corpus samples Unicode and encoding behaviour and does not prove equivalence for
every Unicode version or malformed filesystem name.

## Verification

```shell
cargo fmt --manifest-path tools/github-skills/Cargo.toml --check
cargo clippy --manifest-path tools/github-skills/Cargo.toml --locked --all-targets -- -D warnings
cargo test --manifest-path tools/github-skills/Cargo.toml --locked --all-targets
cargo build --manifest-path tools/github-skills/Cargo.toml --locked --examples
python tools/github-skills/tests/original_contract.py --root . --probe tools/github-skills/target/debug/examples/contract_probe --binary tools/github-skills/target/debug/github-skills --record native-contract-results.json
```

Add `.exe` to both executable paths on Windows. The Python replay uses the
unchanged original unittest objects and additional fabricated differential cases.
Its per-test operation counts distinguish assertions that called native code from
retained Python framework, mock and workflow assertions. The test-only
`contract_probe` example exposes library results through JSON; it is not a product
command or a release executable.

The frozen scanner corpus stores fabricated payloads and decoded text as byte
arrays. Native tests replay its original Python expected results without needing
Python or a scanner exemption. `--freeze-corpus PATH` regenerates this oracle only
during deliberate reference qualification; ordinary replays leave it unchanged.

The repository's documented Python unit, strict-validation, public-file, lint,
coverage and workflow checks remain separate obligations. The shared
`.github/ci/check_gates.py` and the Python fuzz harness remain in place. This
candidate neither publishes binaries nor changes skill installation or CI.

Apache-2.0. Retain the repository's `LICENSE` and `NOTICE` when redistributing.
