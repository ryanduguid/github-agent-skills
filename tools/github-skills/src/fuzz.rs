//! The fuzz target's checks, shared by the cargo-fuzz target and the test
//! that replays its seed corpus.
//!
//! Fabricated bytes go to the public-file scanner's text decoding, Repository
//! checks command extraction and safety patterns. Decoding must return text or
//! a known failure, never both; extracted commands must be stripped and
//! non-empty; every pattern must finish within its backtracking budget. The
//! error says which invariant failed.

use crate::public_files::{self, CLIENT_DATA_PATH, PRIVATE_CONFIG, TEXT_RULES};
use std::io;

/// Decoded text, or the failure that classifies undecodable text.
pub type Decoded = (Option<String>, Option<&'static str>);

/// The scanner functions under test, replaceable so the tests can prove each
/// check bites.
pub struct Scanner {
    pub decode: fn(&[u8]) -> Decoded,
    pub commands: fn(&str) -> io::Result<Vec<String>>,
}

impl Default for Scanner {
    fn default() -> Self {
        Scanner {
            decode: public_files::decoded_text,
            commands: public_files::quick_start_commands,
        }
    }
}

/// Check one input against the real scanner.
pub fn check_input(data: &[u8]) -> Result<(), String> {
    check_with(data, &Scanner::default())
}

pub fn check_with(data: &[u8], scanner: &Scanner) -> Result<(), String> {
    let (text, failure) = (scanner.decode)(data);
    if text.is_some() && failure.is_some() {
        return Err(format!("decoded text also carried failure {failure:?}"));
    }
    if !matches!(
        failure,
        None | Some("unclassifiable-text" | "undecodable-text")
    ) {
        return Err(format!("unexpected failure {failure:?}"));
    }

    let readme = String::from_utf8_lossy(data);
    let commands = (scanner.commands)(&readme)
        .map_err(|error| format!("Repository checks extraction failed: {error}"))?;
    if commands.iter().any(|command| {
        command.is_empty() || command != command.trim_matches(crate::python_whitespace)
    }) {
        return Err(format!(
            "blank or unstripped Repository checks command in {commands:?}"
        ));
    }

    // A pattern that backtracks without bound exhausts its budget here.
    let patterns = TEXT_RULES
        .iter()
        .map(|(rule, pattern)| (*rule, pattern))
        .chain([
            ("private-config", &*PRIVATE_CONFIG),
            ("client-data path", &*CLIENT_DATA_PATH),
        ]);
    for (rule, pattern) in patterns {
        pattern
            .is_match(readme.as_ref())
            .map_err(|error| format!("{rule} pattern did not finish: {error}"))?;
    }
    Ok(())
}
