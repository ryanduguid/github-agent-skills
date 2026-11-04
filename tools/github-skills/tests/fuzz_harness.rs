// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The fuzz target's invariants on its seed corpus, and proof that a failed
//! invariant or an unexpected scanner error reaches the caller. Ported from
//! the retired tests/test_fuzz_harness.py.

use github_skills::fuzz::{Scanner, check_input, check_with};
use std::path::Path;

#[test]
fn seed_corpus() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus");
    let mut seeds: Vec<_> = std::fs::read_dir(&corpus)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    seeds.sort();
    assert!(!seeds.is_empty());
    for seed in seeds {
        check_input(&std::fs::read(&seed).unwrap())
            .unwrap_or_else(|error| panic!("{}: {error}", seed.display()));
    }
}

#[test]
fn a_decoder_returning_text_and_failure_is_caught() {
    let scanner = Scanner {
        decode: |_| (Some("text".into()), Some("undecodable-text")),
        ..Scanner::default()
    };
    let error = check_with(b"x", &scanner).unwrap_err();
    assert!(error.contains("also carried failure"), "{error}");
}

#[test]
fn unexpected_scanner_errors_escape() {
    // A panicking scanner is not caught: the fuzzer reports the crash.
    let scanner = Scanner {
        commands: |_| panic!("bug"),
        ..Scanner::default()
    };
    assert!(std::panic::catch_unwind(|| check_with(b"x", &scanner)).is_err());
}
