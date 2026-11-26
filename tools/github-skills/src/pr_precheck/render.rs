// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: Apache-2.0

//! `json.dumps(report, indent=2, ensure_ascii=True)`, byte for byte.

use super::{LargestFile, NOTE, PathReason, PrecheckReport, WorkflowLine};
use std::fmt::Write as _;

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

fn push_largest(buf: &mut String, item: &LargestFile) {
    push_kv_str(buf, 6, "path", &item.path, true);
    push_kv_str(buf, 6, "status", &item.status, true);
    let _ = writeln!(buf, "      \"additions\": {},", item.additions);
    let _ = writeln!(buf, "      \"deletions\": {}", item.deletions);
}

fn push_path_reason(buf: &mut String, item: &PathReason) {
    push_kv_str(buf, 6, "path", &item.path, true);
    push_kv_str(buf, 6, "reason", &item.reason, false);
}

fn push_workflow_line(buf: &mut String, item: &WorkflowLine) {
    push_kv_str(buf, 6, "path", &item.path, true);
    push_kv_str(buf, 6, "side", item.side, true);
    let _ = writeln!(buf, "      \"line\": {},", item.line);
    buf.push_str("      \"markers\": ");
    push_str_list(buf, 6, &item.markers);
    buf.push('\n');
}

pub fn format_report(report: &PrecheckReport) -> String {
    let mut out = String::from("{\n");
    push_kv_str(&mut out, 2, "note", NOTE, true);
    let _ = writeln!(out, "  \"files\": {},", report.files);
    let _ = writeln!(out, "  \"additions\": {},", report.additions);
    let _ = writeln!(out, "  \"deletions\": {},", report.deletions);
    let _ = writeln!(out, "  \"at_file_limit\": {},", report.at_file_limit);
    push_obj_list(&mut out, "largest", &report.largest, push_largest);
    out.push_str(",\n");
    for (name, items) in [
        ("no_patch", &report.no_patch),
        ("generated", &report.generated),
    ] {
        push_obj_list(&mut out, name, items, push_path_reason);
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
        push_workflow_line,
    );
    out.push_str(",\n");
    push_string_field(&mut out, "instruction_files", &report.instruction_files);
    out.push_str("\n}");
    out
}
