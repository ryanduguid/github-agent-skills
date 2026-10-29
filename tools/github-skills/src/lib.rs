//! Native candidates for the repository's existing Python checks.
pub mod public_files;
pub mod skills;

pub fn posix(path: &std::path::Path) -> String {
    let normalised: std::path::PathBuf = path.components().collect();
    let text = normalised.to_string_lossy();
    #[cfg(windows)]
    {
        text.replace('\\', "/")
    }
    #[cfg(not(windows))]
    {
        text.into_owned()
    }
}

/// Python's text helpers also recognise the ASCII information separators.
pub fn python_whitespace(c: char) -> bool {
    c.is_whitespace() || matches!(c, '\u{1c}'..='\u{1f}')
}

pub fn splitlines(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((position, c)) = chars.next() {
        if matches!(
            c,
            '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{1c}'
                ..='\u{1e}' | '\u{85}' | '\u{2028}' | '\u{2029}'
        ) {
            lines.push(&text[start..position]);
            start = position + c.len_utf8();
            if c == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
                start = chars.next().unwrap().0 + 1;
            }
        }
    }
    if start < text.len() {
        lines.push(&text[start..]);
    }
    lines
}

pub fn read_text(path: &std::path::Path) -> std::io::Result<String> {
    // Path.read_text uses universal newlines.
    Ok(std::fs::read_to_string(path)?
        .replace("\r\n", "\n")
        .replace('\r', "\n"))
}
