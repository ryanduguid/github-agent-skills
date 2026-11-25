// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: Apache-2.0

//! Deterministic pre-check facts for pull request file listings.

use crate::public_files::python_regex;
use fancy_regex::Regex;
use serde_json::Value;
use std::{
    cmp::Reverse,
    ffi::OsString,
    fmt::Write as _,
    fs,
    io::Read,
    ops::ControlFlow,
    path::{Component, Path},
    sync::LazyLock,
};

pub const FILE_LIMIT: usize = 3000;
pub const MAX_INPUT: usize = 256 * 1024 * 1024;
const LARGEST: usize = 10;
const MARKER_LINES: u64 = 10;
const NOTE: &str =
    "Facts computed from the pull request's files list; every path is author-chosen data.";
const USAGE: &str =
    "pr_precheck: usage: github-skills pr-precheck <files.json in the current folder>";

const STATUSES: [&str; 7] = [
    "added",
    "removed",
    "modified",
    "renamed",
    "copied",
    "changed",
    "unchanged",
];

const LOCKFILES: [&str; 20] = [
    "Cargo.lock",
    "package-lock.json",
    "npm-shrinkwrap.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "bun.lock",
    "bun.lockb",
    "poetry.lock",
    "uv.lock",
    "Pipfile.lock",
    "pdm.lock",
    "Gemfile.lock",
    "go.sum",
    "composer.lock",
    "flake.lock",
    "packages.lock.json",
    "pubspec.lock",
    "mix.lock",
    "Package.resolved",
    "gradle.lockfile",
];

const MANIFESTS: [&str; 16] = [
    "Cargo.toml",
    "package.json",
    "pyproject.toml",
    "setup.py",
    "setup.cfg",
    "Pipfile",
    "go.mod",
    "Gemfile",
    "composer.json",
    "pom.xml",
    "build.gradle",
    "build.gradle.kts",
    "pubspec.yaml",
    "mix.exs",
    "Package.swift",
    "Directory.Packages.props",
];

const VENDORED_DIRS: [&str; 4] = ["vendor", "third_party", "third-party", "node_modules"];
const GENERATED_DIRS: [&str; 2] = ["generated", "__generated__"];
const INSTRUCTION_NAMES: [&str; 5] = [
    "AGENTS.md",
    "CLAUDE.md",
    "SKILL.md",
    ".mcp.json",
    "mcp.json",
];
const INSTRUCTION_DIRS: [&str; 2] = [".claude", "skills"];
const PERMISSION_FILES: [&str; 2] = ["CODEOWNERS", ".github/settings.yml"];
const WORKFLOW_MARKER_NAMES: [&str; 9] = [
    "pull_request_target",
    "pull_request",
    "workflow_run",
    "permissions",
    "id-token",
    "write-all",
    "write",
    "secrets",
    "GITHUB_TOKEN",
];

static MANIFEST_NAME: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"^requirements[\w.-]*\.(?:txt|in)$|\.(?:gemspec|csproj|fsproj)$",
        false,
        false,
    )
});

static GENERATED_NAME: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"\.min\.(?:js|css)$|\.map$|_pb2(?:_grpc)?\.pyi?$|\.pb\.go$|\.g\.dart$|\.generated\.\w+$",
        false,
        false,
    )
});

static GENERATED_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"@generated|DO NOT EDIT|Code generated .* DO NOT EDIT",
        true,
        false,
    )
});

static WORKFLOW_MARKERS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
    WORKFLOW_MARKER_NAMES
        .into_iter()
        .map(|name| {
            (
                name,
                python_regex(&format!(r"(?<![\w-]){name}(?![\w])"), false, false),
            )
        })
        .collect()
});

static HUNK: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"^@@ -([0-9]+)(?:,[0-9]+)? \+([0-9]+)(?:,[0-9]+)? @@",
        false,
        false,
    )
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileEntry {
    pub filename: String,
    pub status: String,
    pub additions: u64,
    pub deletions: u64,
    pub previous_filename: Option<String>,
    pub patch: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LargestFile {
    pub path: String,
    pub status: String,
    pub additions: u64,
    pub deletions: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathReason {
    pub path: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowLine {
    pub path: String,
    pub side: &'static str,
    pub line: u64,
    pub markers: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrecheckReport {
    pub files: usize,
    pub additions: u128,
    pub deletions: u128,
    pub at_file_limit: bool,
    pub largest: Vec<LargestFile>,
    pub no_patch: Vec<PathReason>,
    pub generated: Vec<PathReason>,
    pub vendored: Vec<String>,
    pub lockfiles: Vec<String>,
    pub manifests: Vec<String>,
    pub workflows: Vec<String>,
    pub permission_files: Vec<String>,
    pub workflow_lines: Vec<WorkflowLine>,
    pub instruction_files: Vec<String>,
}

struct PosixPath<'a> {
    root: &'static str,
    parts: Vec<&'a str>,
}

impl<'a> PosixPath<'a> {
    fn parse(path: &'a str) -> Self {
        let root = if path.starts_with("//") && !path.starts_with("///") {
            "//"
        } else if path.starts_with('/') {
            "/"
        } else {
            ""
        };
        let mut parts = Vec::new();
        if !root.is_empty() {
            parts.push(root);
        }
        for seg in path.trim_start_matches('/').split('/') {
            if !seg.is_empty() && seg != "." {
                parts.push(seg);
            }
        }
        Self { root, parts }
    }

    fn name(&self) -> &'a str {
        if !self.root.is_empty() && self.parts.len() == 1 {
            ""
        } else {
            self.parts.last().copied().unwrap_or("")
        }
    }

    fn dir_parts(&self) -> &[&'a str] {
        self.parts
            .len()
            .checked_sub(1)
            .map_or(&[], |n| &self.parts[..n])
    }

    fn last_part(&self) -> Option<&'a str> {
        self.parts.last().copied()
    }

    fn as_posix(&self) -> String {
        if !self.root.is_empty() {
            format!("{}{}", self.root, self.parts[1..].join("/"))
        } else if self.parts.is_empty() {
            ".".to_owned()
        } else {
            self.parts.join("/")
        }
    }

    fn parent_as_posix(&self) -> String {
        if !self.root.is_empty() {
            if self.parts.len() <= 2 {
                self.root.to_owned()
            } else {
                format!(
                    "{}{}",
                    self.root,
                    self.parts[1..self.parts.len() - 1].join("/")
                )
            }
        } else if self.parts.len() <= 1 {
            ".".to_owned()
        } else {
            self.parts[..self.parts.len() - 1].join("/")
        }
    }

    fn has_yaml_suffix(&self) -> bool {
        let name = self.name();
        let stem = name
            .strip_suffix(".yml")
            .or_else(|| name.strip_suffix(".yaml"));
        stem.is_some_and(|prefix| prefix.bytes().any(|b| b != b'.'))
    }
}

fn regex_is_match(pattern: &Regex, text: &str) -> Result<bool, String> {
    pattern
        .is_match(text)
        .map_err(|err| format!("pr_precheck: regex error: {err}"))
}

pub fn confine_filename(arg: &OsString) -> Result<&str, String> {
    let Some(name) = arg.to_str() else {
        return Err(USAGE.into());
    };
    if name.is_empty()
        || matches!(name, "." | "..")
        || name.starts_with('-')
        || name.contains(['/', '\\', '\0'])
    {
        return Err(USAGE.into());
    }
    let mut comps = Path::new(name).components();
    if !matches!(
        (comps.next(), comps.next()),
        (Some(Component::Normal(_)), None)
    ) {
        return Err(USAGE.into());
    }
    Ok(name)
}

pub fn read_bounded_utf8(reader: impl Read, limit: usize) -> Result<String, String> {
    let mut data = Vec::new();
    reader
        .take(limit as u64 + 1)
        .read_to_end(&mut data)
        .map_err(|error| format!("pr_precheck: {error}"))?;
    if data.len() > limit {
        return Err(format!("pr_precheck: input is larger than {limit} bytes"));
    }
    String::from_utf8(data).map_err(|error| format!("pr_precheck: {error}"))
}

fn pages(document: Value) -> Result<Vec<Vec<Value>>, String> {
    let Value::Array(items) = document else {
        return Err("pr_precheck: expected a JSON array of changed files".into());
    };
    if !items.is_empty() && items.iter().all(Value::is_array) {
        let mut out = Vec::with_capacity(items.len());
        for page in items {
            if let Value::Array(entries) = page {
                out.push(entries);
            }
        }
        return Ok(out);
    }
    Ok(vec![items])
}

fn parse_file_entry(item: Value, number: usize) -> Result<FileEntry, String> {
    let prefix = format!("pr_precheck: changed file {number}");
    let Value::Object(mut map) = item else {
        return Err(format!("{prefix}: not a JSON object"));
    };
    let Some(Value::String(filename)) = map.remove("filename") else {
        return Err(format!("{prefix}: 'filename' must be a non-empty string"));
    };
    if filename.is_empty() {
        return Err(format!("{prefix}: 'filename' must be a non-empty string"));
    }
    let Some(Value::String(status)) = map.remove("status") else {
        return Err(format!("{prefix}: 'status' is not a GitHub file status"));
    };
    if !STATUSES.contains(&status.as_str()) {
        return Err(format!("{prefix}: 'status' is not a GitHub file status"));
    }
    let (Some(additions), Some(deletions)) = (
        map.get("additions").and_then(Value::as_u64),
        map.get("deletions").and_then(Value::as_u64),
    ) else {
        return Err(format!(
            "{prefix}: 'additions' and 'deletions' must be non-negative integers"
        ));
    };
    let previous_filename = match map.remove("previous_filename") {
        None => None,
        Some(Value::String(prev)) if !prev.is_empty() => Some(prev),
        Some(_) => {
            return Err(format!(
                "{prefix}: 'previous_filename' must be a non-empty string"
            ));
        }
    };
    let patch = match map.remove("patch") {
        None => None,
        Some(Value::String(text)) => Some(text),
        Some(_) => return Err(format!("{prefix}: 'patch' must be a string")),
    };
    Ok(FileEntry {
        filename,
        status,
        additions,
        deletions,
        previous_filename,
        patch,
    })
}

pub fn read_pages(text: &str) -> Result<Vec<FileEntry>, String> {
    let mut rest = text.trim_matches(crate::python_whitespace);
    if rest.is_empty() {
        return Err("pr_precheck: empty input".into());
    }
    let mut files = Vec::new();
    while !rest.is_empty() {
        let mut stream = serde_json::Deserializer::from_str(rest).into_iter::<Value>();
        let document = stream
            .next()
            .ok_or_else(|| "pr_precheck: empty input".to_owned())?
            .map_err(|err| format!("pr_precheck: not JSON: {err}"))?;
        let offset = stream.byte_offset();
        for page in pages(document)? {
            for item in page {
                let number = files.len() + 1;
                files.push(parse_file_entry(item, number)?);
            }
        }
        rest = rest[offset..].trim_start_matches(crate::python_whitespace);
    }
    Ok(files)
}

fn for_each_diff_line<'a, F>(patch: &'a str, mut visit: F) -> Result<(), String>
where
    F: FnMut(&'static str, u64, &'a str) -> Result<ControlFlow<()>, String>,
{
    let mut base: u64 = 0;
    let mut head: u64 = 0;
    for text in patch.split('\n') {
        if let Some(caps) = HUNK
            .captures(text)
            .map_err(|err| format!("pr_precheck: regex error: {err}"))?
        {
            let parse = |idx: usize| -> Result<u64, String> {
                caps.get(idx)
                    .expect("capture group in HUNK")
                    .as_str()
                    .parse::<u64>()
                    .map_err(|_| "pr_precheck: hunk line number out of range".into())
            };
            base = parse(1)?;
            head = parse(2)?;
        } else if let Some(added) = text.strip_prefix('+') {
            if visit("head", head, added)?.is_break() {
                break;
            }
            head = head.saturating_add(1);
        } else if let Some(removed) = text.strip_prefix('-') {
            if visit("base", base, removed)?.is_break() {
                break;
            }
            base = base.saturating_add(1);
        } else if !text.starts_with('\\') {
            base = base.saturating_add(1);
            head = head.saturating_add(1);
        }
    }
    Ok(())
}

fn no_patch_reason(entry: &FileEntry) -> &'static str {
    if entry.additions > 0 || entry.deletions > 0 {
        "diff too large"
    } else if entry.status == "renamed" {
        "rename without changes"
    } else {
        "binary"
    }
}

fn generated_reason(path: &str, patch: &str) -> Result<Option<String>, String> {
    let pure = PosixPath::parse(path);
    let name_match = match pure.last_part() {
        Some(last) => regex_is_match(&GENERATED_NAME, last)?,
        None => false,
    };
    if name_match
        || pure
            .dir_parts()
            .iter()
            .any(|part| GENERATED_DIRS.contains(part))
    {
        return Ok(Some("generated file name".into()));
    }
    let mut found = None;
    for_each_diff_line(patch, |side, line, text| {
        if side == "head" && line <= MARKER_LINES && regex_is_match(&GENERATED_MARKER, text)? {
            found = Some(format!("generated marker at line {line}"));
            return Ok(ControlFlow::Break(()));
        }
        Ok(ControlFlow::Continue(()))
    })?;
    Ok(found)
}

fn is_workflow(raw_path: &str) -> bool {
    let pure = PosixPath::parse(raw_path);
    let in_workflows = pure.parent_as_posix() == ".github/workflows" && pure.has_yaml_suffix();
    in_workflows
        || matches!(pure.name(), "action.yml" | "action.yaml")
        || raw_path == ".github/dependabot.yml"
}

fn is_instruction(pure: &PosixPath<'_>) -> bool {
    pure.last_part()
        .is_some_and(|last| INSTRUCTION_NAMES.contains(&last))
        || pure
            .dir_parts()
            .iter()
            .any(|part| INSTRUCTION_DIRS.contains(part))
}

fn collect_workflow_lines(
    path: &str,
    patch: &str,
    out: &mut Vec<WorkflowLine>,
) -> Result<(), String> {
    for_each_diff_line(patch, |side, line, text| {
        let mut markers = Vec::new();
        for (name, pattern) in WORKFLOW_MARKERS.iter() {
            if regex_is_match(pattern, text)? {
                markers.push(*name);
            }
        }
        if !markers.is_empty() {
            out.push(WorkflowLine {
                path: path.to_owned(),
                side,
                line,
                markers,
            });
        }
        Ok(ControlFlow::Continue(()))
    })
}

fn classify_names(entry: &FileEntry, out: &mut PrecheckReport) -> Result<(), String> {
    let names = std::iter::once(entry.filename.as_str()).chain(entry.previous_filename.as_deref());
    for name in names {
        let pure = PosixPath::parse(name);
        let file_name = pure.name();
        let posix = pure.as_posix();
        if LOCKFILES.contains(&file_name) {
            out.lockfiles.push(name.to_owned());
        }
        if MANIFESTS.contains(&file_name) || regex_is_match(&MANIFEST_NAME, file_name)? {
            out.manifests.push(name.to_owned());
        }
        if pure
            .dir_parts()
            .iter()
            .any(|part| VENDORED_DIRS.contains(part))
        {
            out.vendored.push(name.to_owned());
        }
        if is_workflow(&posix) {
            out.workflows.push(name.to_owned());
        }
        if PERMISSION_FILES.contains(&file_name) || PERMISSION_FILES.contains(&posix.as_str()) {
            out.permission_files.push(name.to_owned());
        }
        if is_instruction(&pure) {
            out.instruction_files.push(name.to_owned());
        }
    }
    Ok(())
}

fn classify(entry: &FileEntry, out: &mut PrecheckReport) -> Result<(), String> {
    let patch = match &entry.patch {
        Some(text) => text.as_str(),
        None => {
            out.no_patch.push(PathReason {
                path: entry.filename.clone(),
                reason: no_patch_reason(entry).into(),
            });
            ""
        }
    };
    classify_names(entry, out)?;
    if let Some(reason) = generated_reason(&entry.filename, patch)? {
        out.generated.push(PathReason {
            path: entry.filename.clone(),
            reason,
        });
    }
    if is_workflow(&entry.filename) {
        collect_workflow_lines(&entry.filename, patch, &mut out.workflow_lines)?;
    }
    Ok(())
}

pub fn precheck(files: &[FileEntry]) -> Result<PrecheckReport, String> {
    let additions = files.iter().map(|item| u128::from(item.additions)).sum();
    let deletions = files.iter().map(|item| u128::from(item.deletions)).sum();
    let mut ranked: Vec<usize> = (0..files.len()).collect();
    ranked.sort_by_key(|&idx| {
        Reverse(u128::from(files[idx].additions) + u128::from(files[idx].deletions))
    });
    let largest = ranked
        .into_iter()
        .take(LARGEST)
        .map(|idx| {
            let item = &files[idx];
            LargestFile {
                path: item.filename.clone(),
                status: item.status.clone(),
                additions: item.additions,
                deletions: item.deletions,
            }
        })
        .collect();
    let mut report = PrecheckReport {
        files: files.len(),
        additions,
        deletions,
        at_file_limit: files.len() >= FILE_LIMIT,
        largest,
        no_patch: Vec::new(),
        generated: Vec::new(),
        vendored: Vec::new(),
        lockfiles: Vec::new(),
        manifests: Vec::new(),
        workflows: Vec::new(),
        permission_files: Vec::new(),
        workflow_lines: Vec::new(),
        instruction_files: Vec::new(),
    };
    for item in files {
        classify(item, &mut report)?;
    }
    Ok(report)
}

fn push_json_str_ascii(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (' '..='~').contains(&c) => out.push(c),
            c => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
        }
    }
    out.push('"');
}

fn push_str_list(out: &mut String, indent: usize, items: &[&str]) {
    if items.is_empty() {
        out.push_str("[]");
        return;
    }
    out.push_str("[\n");
    let pad = " ".repeat(indent + 2);
    for (idx, item) in items.iter().enumerate() {
        out.push_str(&pad);
        push_json_str_ascii(out, item);
        if idx + 1 < items.len() {
            out.push(',');
        }
        out.push('\n');
    }
    let _ = write!(out, "{}]", " ".repeat(indent));
}

fn push_string_field(out: &mut String, name: &str, items: &[String]) {
    let refs: Vec<&str> = items.iter().map(String::as_str).collect();
    let _ = write!(out, "  \"{name}\": ");
    push_str_list(out, 2, &refs);
}

fn push_obj_list<T, F>(out: &mut String, name: &str, items: &[T], mut render: F)
where
    F: FnMut(&mut String, &T),
{
    let _ = write!(out, "  \"{name}\": ");
    if items.is_empty() {
        out.push_str("[]");
        return;
    }
    out.push_str("[\n");
    for (idx, item) in items.iter().enumerate() {
        out.push_str("    {\n");
        render(out, item);
        out.push_str("    }");
        if idx + 1 < items.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  ]");
}

fn push_kv_str(out: &mut String, indent: usize, key: &str, val: &str, comma: bool) {
    let _ = write!(out, "{}\"{key}\": ", " ".repeat(indent));
    push_json_str_ascii(out, val);
    if comma {
        out.push(',');
    }
    out.push('\n');
}

pub fn format_report(report: &PrecheckReport) -> String {
    let mut out = String::from("{\n");
    push_kv_str(&mut out, 2, "note", NOTE, true);
    let _ = writeln!(out, "  \"files\": {},", report.files);
    let _ = writeln!(out, "  \"additions\": {},", report.additions);
    let _ = writeln!(out, "  \"deletions\": {},", report.deletions);
    let _ = writeln!(out, "  \"at_file_limit\": {},", report.at_file_limit);

    push_obj_list(&mut out, "largest", &report.largest, |buf, item| {
        push_kv_str(buf, 6, "path", &item.path, true);
        push_kv_str(buf, 6, "status", &item.status, true);
        let _ = writeln!(buf, "      \"additions\": {},", item.additions);
        let _ = writeln!(buf, "      \"deletions\": {}", item.deletions);
    });
    out.push_str(",\n");

    for (name, items) in [
        ("no_patch", &report.no_patch),
        ("generated", &report.generated),
    ] {
        push_obj_list(&mut out, name, items, |buf, item| {
            push_kv_str(buf, 6, "path", &item.path, true);
            push_kv_str(buf, 6, "reason", &item.reason, false);
        });
        out.push_str(",\n");
    }

    for (name, items) in [
        ("vendored", &report.vendored),
        ("lockfiles", &report.lockfiles),
        ("manifests", &report.manifests),
        ("workflows", &report.workflows),
        ("permission_files", &report.permission_files),
    ] {
        push_string_field(&mut out, name, items);
        out.push_str(",\n");
    }

    push_obj_list(
        &mut out,
        "workflow_lines",
        &report.workflow_lines,
        |buf, item| {
            push_kv_str(buf, 6, "path", &item.path, true);
            push_kv_str(buf, 6, "side", item.side, true);
            let _ = writeln!(buf, "      \"line\": {},", item.line);
            buf.push_str("      \"markers\": ");
            push_str_list(buf, 6, &item.markers);
            buf.push('\n');
        },
    );
    out.push_str(",\n");

    push_string_field(&mut out, "instruction_files", &report.instruction_files);
    out.push_str("\n}");
    out
}

pub fn run_from_str(text: &str) -> Result<String, String> {
    let files = read_pages(text)?;
    let report = precheck(&files)?;
    Ok(format_report(&report))
}

pub fn run_cli(args: &[OsString]) -> Result<String, String> {
    let [arg] = args else {
        return Err(USAGE.into());
    };
    let name = confine_filename(arg)?;
    let file = fs::File::open(Path::new(name)).map_err(|error| format!("pr_precheck: {error}"))?;
    let text = read_bounded_utf8(file, MAX_INPUT)?;
    run_from_str(&text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn read_bounded_rejects_oversize_input_and_invalid_utf8() {
        let err = read_bounded_utf8(Cursor::new(b"abcd"), 3).unwrap_err();
        assert_eq!(err, "pr_precheck: input is larger than 3 bytes");

        let ok = read_bounded_utf8(Cursor::new(b"abc"), 3).unwrap();
        assert_eq!(ok, "abc");

        let utf8_err = read_bounded_utf8(Cursor::new([0xff, 0xfe]), 10).unwrap_err();
        assert!(utf8_err.starts_with("pr_precheck: "));
    }

    #[test]
    fn confine_filename_rejects_separators_dot_entries_flags_and_non_utf8() {
        for bad in ["", ".", "..", "-f", "--help", "a/b", "a\\b", "a\0b"] {
            assert!(confine_filename(&OsString::from(bad)).is_err(), "{bad:?}");
        }
        assert_eq!(
            confine_filename(&OsString::from("files.json")).unwrap(),
            "files.json"
        );
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            let raw = OsString::from_vec(vec![0xff]);
            assert!(confine_filename(&raw).is_err());
        }
    }

    #[test]
    fn posix_path_matches_pure_posix_path_edge_cases() {
        let dot = PosixPath::parse(".");
        assert_eq!(dot.name(), "");
        assert_eq!(dot.as_posix(), ".");
        assert_eq!(dot.parent_as_posix(), ".");
        assert_eq!(dot.last_part(), None);
        assert!(dot.dir_parts().is_empty());

        let root = PosixPath::parse("///");
        assert_eq!(root.name(), "");
        assert_eq!(root.as_posix(), "/");
        assert_eq!(root.parent_as_posix(), "/");

        let root_child = PosixPath::parse("/a");
        assert_eq!(root_child.name(), "a");
        assert_eq!(root_child.as_posix(), "/a");
        assert_eq!(root_child.parent_as_posix(), "/");

        let unc = PosixPath::parse("//srv/a/b");
        assert_eq!(unc.as_posix(), "//srv/a/b");
        assert_eq!(unc.parent_as_posix(), "//srv/a");
        assert_eq!(unc.name(), "b");

        let dot_yml = PosixPath::parse("..yml");
        assert!(!dot_yml.has_yaml_suffix());
        let hidden_yml = PosixPath::parse(".a.yml");
        assert!(hidden_yml.has_yaml_suffix());
    }

    #[test]
    fn hunk_overflow_and_edge_cases_do_not_panic() {
        let huge = format!("@@ -{} +1 @@\n+x", "9".repeat(30));
        let err = generated_reason("src/a.rs", &huge).unwrap_err();
        assert_eq!(err, "pr_precheck: hunk line number out of range");

        assert_eq!(generated_reason(".", "").unwrap(), None);

        let mut seen = 0;
        for_each_diff_line("-a\n-b\n", |_, _, _| {
            seen += 1;
            Ok(ControlFlow::Break(()))
        })
        .unwrap();
        assert_eq!(seen, 1);
    }
}
