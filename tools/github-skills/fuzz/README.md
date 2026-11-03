`fuzz/` is a separately locked cargo-fuzz crate with one libFuzzer target,
`public_files`, which feeds fabricated bytes to the public-file scanner through
`github_skills::fuzz::check_input`. It checks BOM-marked and NUL-bearing text
decoding, Repository checks command extraction and every safety pattern:
decoding returns text or a failure, never both; extracted commands are
stripped and non-empty; and each pattern finishes within its backtracking
budget. Panics and failed invariants fail the run.

On Linux, with the `nightly-2026-10-15` toolchain and cargo-fuzz 0.13.2 that
CI pins, run from `tools/github-skills`:

```sh
cargo +nightly-2026-10-15 fuzz run --target x86_64-unknown-linux-gnu public_files fuzz/corpus -- -max_total_time=60 -max_len=4096 -timeout=2 -rss_limit_mb=1024
```

The cargo-fuzz release binary is built for musl and defaults to that target,
where the address sanitiser cannot build, so the command names the glibc
target. CI runs the target weekly and on demand, with read-only repository
access and a 20-minute timeout, and lints it with Clippy on every pull request.
`tests/fuzz_harness.rs` replays the checked-in corpus with `cargo test` on
Linux and Windows, and proves that a failed
invariant or a scanner panic reaches the caller. A successful run is bounded
evidence, not proof that every input is handled.
