use fancy_regex::{Regex, RegexBuilder};
use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::LazyLock,
};

pub const CLONE_URL: &str = "https://github.com/ryanduguid/github-agent-skills.git";
pub const REPOSITORY_NAME: &str = "github-agent-skills";

// These are fixed repository patterns, not user-supplied expressions. Python's
// Unicode word and whitespace classes differ from Rust's. Its case-insensitive
// ASCII i/I also matches the dotted and dotless Turkish letters.
pub fn python_regex(pattern: &str, ignore_case: bool, multiline: bool) -> Regex {
    let word = r"[\p{L}\p{N}_]";
    let mut translated = String::new();
    let mut chars = pattern.chars().peekable();
    let mut class = false;
    while let Some(c) = chars.next() {
        if c == '\\' {
            let escaped = chars.next().expect("complete fixed regex escape");
            match escaped {
                'b' if !class => {
                    translated.push_str(&format!("(?:(?<!{word})(?={word})|(?<={word})(?!{word}))"))
                }
                's' => translated.push_str(if class {
                    r"\s\u{1c}-\u{1f}"
                } else {
                    r"[\s\u{1c}-\u{1f}]"
                }),
                'S' => translated.push_str(if class { r"\S" } else { r"[^\s\u{1c}-\u{1f}]" }),
                'Z' => translated.push_str(r"\z"),
                _ => {
                    translated.push('\\');
                    translated.push(escaped);
                }
            }
        } else if c == '[' {
            class = true;
            translated.push(c);
        } else if c == ']' {
            if ignore_case {
                let start = translated.rfind('[').unwrap();
                let content = &translated[start..];
                if content.contains("A-Z") || content.contains("a-z") {
                    translated.push_str("İı");
                }
            }
            class = false;
            translated.push(c);
        } else if c == '$' && !class && !multiline {
            // Python also permits this anchor immediately before a final LF.
            translated.push_str(r"(?=\n?\z)");
        } else if ignore_case && matches!(c, 'i' | 'I') {
            translated.push_str(if class { "iİı" } else { "[iİı]" });
        } else {
            translated.push(c);
        }
    }
    let flags = match (ignore_case, multiline) {
        (true, true) => "(?im)",
        (true, false) => "(?i)",
        (false, true) => "(?m)",
        (false, false) => "",
    };
    // The default budget is too small to scan this repository's frozen byte
    // corpus. Exhaustion still propagates as an error, never a passing result.
    RegexBuilder::new(&format!("{flags}{translated}"))
        .backtrack_limit(100_000_000)
        .build()
        .expect("valid fixed Python-compatible pattern")
}

static PRIVATE_CONFIG: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"(?:^|/)(?:\.env(?:\.[^/]+)?|\.netrc|\.npmrc|credentials(?:\.[^/]+)?|\.(?:ssh|aws)(?:/|$)|(?:home|Users)/[^/]+/\.config(?:/|$))",
        true,
        false,
    )
});
static CLIENT_DATA_PATH: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"(?:^|/)(?:clients?|customers?)(?:/|$)|(?:^|/)(?:client|customer)[_-](?:data|records?|export|files?)(?:[._-]|$)",
        true,
        false,
    )
});
static TEXT_RULES: LazyLock<Vec<(&str, Regex)>> = LazyLock::new(|| {
    vec![
        (
            "transcript-path",
            python_regex(
                r"(?:^|[\s/\\])(?:transcripts?|sessions?)[/\\][^\s]+",
                true,
                false,
            ),
        ),
        (
            "raw-sdd-path",
            python_regex(
                r"\.superpowers[/\\]sdd[/\\][^/\\*\s]+[/\\]raw[/\\]",
                true,
                false,
            ),
        ),
        (
            "private-user-path",
            python_regex(
                r"(?:[A-Za-z]:[/\\](?:Users|Documents and Settings)[/\\]|\\\\[^/\\\s]+[/\\]+(?:Users|Documents and Settings)[/\\]|(?<![A-Za-z0-9])/(?:home|Users)/)[^/\\\s]+",
                true,
                false,
            ),
        ),
        (
            "credential-assignment",
            python_regex(
                r#"(?<![A-Za-z0-9_-])['"]?(?:[A-Za-z0-9_-]*(?:password|secret|token)[A-Za-z0-9_-]*|api[_ -]?key|access[_ -]?key|client[_ -]?secret|private[_ -]?key|(?!(?:public|example)[_-]?key\b)[A-Za-z0-9_-]+[_-]key)['"]?\s*[:=]\s*['"]?\S+"#,
                true,
                false,
            ),
        ),
        (
            "client-data",
            python_regex(
                r#"(?:^|[{\[,])[ \t]*['"]?(?:client|customer)[_-]?(?:data|records?|export|file|id|name)['"]?[ \t]*[:=]"#,
                true,
                true,
            ),
        ),
    ]
});
static WORKFLOW_REFERENCE: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"^[ \t]+[A-Za-z_][A-Za-z0-9_]*:[ \t]*\$\{\{[ \t]*(?:github\.token|secrets\.[A-Z][A-Z0-9_]*)[ \t]*\}\}[ \t]*\r?$",
        false,
        true,
    )
});
static PERMISSION_LINE: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        concat!(
            r"^[ \t]+id-to",
            r"ken:[ \t]*(?:read|write|none)[ \t]*(?:#.*)?\r?$"
        ),
        false,
        true,
    )
});
static QUICK_START: LazyLock<Regex> =
    LazyLock::new(|| python_regex(r"^## Quick start\s*$([\s\S]*?)(?=(?:^## )|\Z)", false, true));
static FENCED_COMMANDS: LazyLock<Regex> =
    LazyLock::new(|| python_regex(r"```(?:powershell|shell)\n([\s\S]*?)```", false, false));
static VALIDATE_JOB: LazyLock<Regex> =
    LazyLock::new(|| python_regex(r"^  validate:\n((?:^(?:    .*)?$\n)+)", false, true));
static WORKFLOW_CHECK: LazyLock<Regex> =
    LazyLock::new(|| python_regex(r"^ {8}run: (python .+)$", false, true));
static PERMISSIONS: LazyLock<Regex> =
    LazyLock::new(|| python_regex(r"^permissions:\n((?:^[ ]{2}.*\n)+)", false, true));

fn captures<'a>(
    pattern: &Regex,
    text: &'a str,
) -> io::Result<Option<fancy_regex::Captures<'a, str>>> {
    pattern.captures(text).map_err(io::Error::other)
}
fn matches(pattern: &Regex, text: &str) -> io::Result<bool> {
    pattern.is_match(text).map_err(io::Error::other)
}

pub fn permissions_block(workflow: &str) -> io::Result<Option<Vec<String>>> {
    Ok(captures(&PERMISSIONS, workflow)?.map(|m| {
        crate::splitlines(m.get(1).unwrap().as_str())
            .into_iter()
            .map(str::to_owned)
            .collect()
    }))
}
pub fn quick_start_commands(readme: &str) -> io::Result<Vec<String>> {
    let Some(section) = captures(&QUICK_START, readme)? else {
        return Ok(vec![]);
    };
    let mut commands = Vec::new();
    for capture in FENCED_COMMANDS.captures_iter(section.get(1).unwrap().as_str()) {
        let capture = capture.map_err(io::Error::other)?;
        for line in crate::splitlines(capture.get(1).unwrap().as_str()) {
            let command = line.trim_matches(crate::python_whitespace);
            if !command.is_empty() {
                commands.push(command.to_owned());
            }
        }
    }
    Ok(commands)
}
pub fn supported_quick_start(workflow: &str) -> io::Result<Vec<String>> {
    let mut result = vec![
        format!("git clone {CLONE_URL}"),
        format!("cd {REPOSITORY_NAME}"),
    ];
    if let Some(job) = captures(&VALIDATE_JOB, workflow)? {
        for capture in WORKFLOW_CHECK.captures_iter(job.get(1).unwrap().as_str()) {
            result.push(
                capture
                    .map_err(io::Error::other)?
                    .get(1)
                    .unwrap()
                    .as_str()
                    .trim_end_matches(crate::python_whitespace)
                    .to_owned(),
            );
        }
    }
    Ok(result)
}
pub fn quick_start_failures(readme: &str, workflow: &str) -> io::Result<Vec<String>> {
    Ok(
        if quick_start_commands(readme)? == supported_quick_start(workflow)? {
            vec![]
        } else {
            vec![
                "README.md: Quick start commands differ from the checks the Validate workflow runs"
                    .into(),
            ]
        },
    )
}
pub fn looks_like_text(content: &[u8]) -> bool {
    !content.is_empty()
        && content
            .iter()
            .filter(|c| matches!(c, 9 | 10 | 13 | 32..=126))
            .count()
            * 5
            >= content.len() * 4
}
pub fn decoded_text(content: &[u8]) -> (Option<String>, Option<&'static str>) {
    let decoded =
        if content.starts_with(&[0xff, 0xfe, 0, 0]) || content.starts_with(&[0, 0, 0xfe, 0xff]) {
            let little = content[0] == 0xff;
            let bytes = &content[4..];
            if !bytes.len().is_multiple_of(4) {
                None
            } else {
                bytes
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .map(|c| {
                        char::from_u32(if little {
                            u32::from_le_bytes(*c)
                        } else {
                            u32::from_be_bytes(*c)
                        })
                    })
                    .collect::<Option<String>>()
            }
        } else if content.starts_with(&[0xff, 0xfe]) || content.starts_with(&[0xfe, 0xff]) {
            let little = content[0] == 0xff;
            let bytes = &content[2..];
            if !bytes.len().is_multiple_of(2) {
                None
            } else {
                String::from_utf16(
                    &bytes
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|c| {
                            if little {
                                u16::from_le_bytes(*c)
                            } else {
                                u16::from_be_bytes(*c)
                            }
                        })
                        .collect::<Vec<_>>(),
                )
                .ok()
            }
        } else if content.contains(&0) {
            let without_nuls: Vec<_> = content.iter().copied().filter(|c| *c != 0).collect();
            return if looks_like_text(&without_nuls) {
                (None, Some("unclassifiable-text"))
            } else {
                (None, None)
            };
        } else {
            return match String::from_utf8(content.to_vec()) {
                Ok(text) => (Some(text), None),
                Err(_) if looks_like_text(content) => (None, Some("undecodable-text")),
                Err(_) => (None, None),
            };
        };
    match decoded {
        Some(text) => (Some(text), None),
        None => (None, Some("unclassifiable-text")),
    }
}

pub fn tracked_paths(root: &Path) -> io::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["ls-files", "-z"])
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        return Err(io::Error::other("git ls-files failed"));
    }
    let text = String::from_utf8(output.stdout).map_err(io::Error::other)?;
    Ok(text
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
        .collect())
}

pub fn scan_paths(root: &Path, paths: &[PathBuf]) -> io::Result<Vec<String>> {
    let mut failures = Vec::new();
    for path in paths {
        let relative = crate::posix(path);
        if matches(&PRIVATE_CONFIG, &relative)? {
            failures.push(format!("{relative}: private-config"));
            continue;
        }
        if matches(&CLIENT_DATA_PATH, &relative)? {
            failures.push(format!("{relative}: client-data"));
            continue;
        }
        let full = root.join(path);
        let (text, decoding_failure) = if full.is_symlink() {
            (Some(crate::posix(&fs::read_link(&full)?)), None)
        } else {
            match fs::read(&full) {
                Ok(content) => decoded_text(&content),
                Err(error) => {
                    let class = match error.kind() {
                        io::ErrorKind::NotFound => "FileNotFoundError",
                        io::ErrorKind::PermissionDenied => "PermissionError",
                        io::ErrorKind::IsADirectory => "IsADirectoryError",
                        io::ErrorKind::NotADirectory => "NotADirectoryError",
                        _ => "OSError",
                    };
                    failures.push(format!("{relative}: unreadable ({class})"));
                    continue;
                }
            }
        };
        if let Some(failure) = decoding_failure {
            failures.push(format!("{relative}: {failure}"));
            continue;
        }
        let Some(text) = text else {
            continue;
        };
        for (rule, pattern) in TEXT_RULES.iter() {
            let workflow = path.parent() == Some(Path::new(".github/workflows"))
                && path.extension().is_some_and(|s| s == "yml" || s == "yaml");
            let candidate = if *rule == "credential-assignment" && workflow {
                let candidate = WORKFLOW_REFERENCE
                    .try_replacen(&text, 0, "")
                    .map_err(io::Error::other)?;
                PERMISSION_LINE
                    .try_replacen(&candidate, 0, "")
                    .map_err(io::Error::other)?
                    .into_owned()
            } else {
                text.clone()
            };
            if matches(pattern, &candidate)? {
                failures.push(format!("{relative}: {rule}"));
                break;
            }
        }
    }
    Ok(failures)
}
pub fn tracked_failures(root: &Path) -> io::Result<Vec<String>> {
    scan_paths(root, &tracked_paths(root)?)
}
pub fn check(root: &Path) -> io::Result<Vec<String>> {
    let mut failures = tracked_failures(root)?;
    let workflow = crate::read_text(&root.join(".github/workflows/validate.yml"))?;
    failures.extend(quick_start_failures(
        &crate::read_text(&root.join("README.md"))?,
        &workflow,
    )?);
    if permissions_block(&workflow)? != Some(vec!["  contents: read".into()]) {
        failures.push(
            ".github/workflows/validate.yml: workflow permissions are not read-only contents"
                .into(),
        );
    }
    Ok(failures)
}
