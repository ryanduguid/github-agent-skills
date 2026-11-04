// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Fabricated bytes to the public-file scanner's decoding, Repository checks
//! extraction and safety patterns.
#![no_main]

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if let Err(problem) = github_skills::fuzz::check_input(data) {
        panic!("{problem}");
    }
});
