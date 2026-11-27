// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: Apache-2.0

//! Differential parity: `github-skills pr-precheck` against the skill's
//! `pr_precheck.py` on the same inputs. Needs Python 3.8 or later on PATH
//! (or named by `PYTHON`), as the repository's unit tests do.

use serde_json::{Value, json};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const WORKFLOW_PATCH: &str = "@@ -1,3 +1,4 @@\n on:\n-  pull_request:\n+  pull_request_target:\n+permissions: write-all\n \\ No newline at end of file\n@@ -10 +11,3 @@\n+  grants id-token write\n+  run: echo ${{ secrets.GITHUB_TOKEN }}\n+  my-write: writer\n-  contents: write";

/// Raw inputs: page layouts, separators and every refusal path.
const RAW_CASES: [(&str, &[u8]); 22] = [
    ("empty array", b"[]"),
    ("empty file", b""),
    ("whitespace only", b" \n\t\x1c "),
    ("concatenated pages", b"[{\"filename\":\"a\",\"status\":\"added\",\"additions\":1,\"deletions\":0}]\n\x1c [{\"filename\":\"b\",\"status\":\"removed\",\"additions\":0,\"deletions\":2}]\n"),
    ("slurped pages", b"[[{\"filename\":\"a\",\"status\":\"added\",\"additions\":1,\"deletions\":0}],[]]"),
    ("array of empty arrays", b"[[],[]]"),
    ("mixed array", b"[[], {\"filename\":\"a\",\"status\":\"added\",\"additions\":1,\"deletions\":0}]"),
    ("duplicate keys keep the last", b"[{\"filename\":\"a\",\"filename\":\"Cargo.lock\",\"status\":\"added\",\"additions\":1,\"deletions\":0}]"),
    ("not json", b"[{"),
    ("trailing garbage", b"[] x"),
    ("object", b"{}"),
    ("number item", b"[1]"),
    ("missing filename", b"[{\"status\":\"added\",\"additions\":1,\"deletions\":0}]"),
    ("empty filename", b"[{\"filename\":\"\",\"status\":\"added\",\"additions\":1,\"deletions\":0}]"),
    ("bad status", b"[{\"filename\":\"a\",\"status\":\"bogus\",\"additions\":1,\"deletions\":0}]"),
    ("negative count", b"[{\"filename\":\"a\",\"status\":\"added\",\"additions\":-1,\"deletions\":0}]"),
    ("boolean count", b"[{\"filename\":\"a\",\"status\":\"added\",\"additions\":true,\"deletions\":0}]"),
    ("float count", b"[{\"filename\":\"a\",\"status\":\"added\",\"additions\":1.5,\"deletions\":0}]"),
    ("NaN count", b"[{\"filename\":\"a\",\"status\":\"added\",\"additions\":NaN,\"deletions\":0}]"),
    ("empty previous_filename", b"[{\"filename\":\"a\",\"status\":\"renamed\",\"additions\":0,\"deletions\":0,\"previous_filename\":\"\"}]"),
    ("non-string patch", b"[{\"filename\":\"a\",\"status\":\"added\",\"additions\":1,\"deletions\":0,\"patch\":5}]"),
    ("invalid UTF-8", b"[{\"filename\":\"\xff\",\"status\":\"added\",\"additions\":1,\"deletions\":0}]"),
];

fn entry(path: &str, status: &str, counts: (u64, u64), patch: Option<&str>) -> Value {
    let mut item = json!({
        "filename": path,
        "status": status,
        "additions": counts.0,
        "deletions": counts.1,
    });
    if let Some(text) = patch {
        item["patch"] = json!(text);
    }
    item
}

const PLUS: Option<&str> = Some("@@ -1 +1 @@\n+x");

/// One file of each classification, the path quirks `PurePosixPath` keeps,
/// author-chosen control and non-ASCII characters, and ties in `largest`.
/// Path, status, (additions, deletions) and patch.
type Case = (&'static str, &'static str, (u64, u64), Option<&'static str>);

const CLASSIFIED: [Case; 35] = [
    (
        ".github/workflows/ci.yml",
        "modified",
        (5, 2),
        Some(WORKFLOW_PATCH),
    ),
    (
        "./.github/workflows/a.yml",
        "added",
        (1, 0),
        Some("@@ -0,0 +1 @@\n+on: pull_request"),
    ),
    (".github//workflows/b.yaml", "added", (1, 0), PLUS),
    (".github/workflows/.yml", "added", (1, 0), PLUS),
    ("//.github/workflows/c.yml", "added", (1, 0), PLUS),
    (
        "./.github/dependabot.yml",
        "modified",
        (1, 0),
        Some("@@ -1 +1 @@\n+permissions: write"),
    ),
    (".github/dependabot.yml", "modified", (1, 0), PLUS),
    ("sub/action.yml", "added", (1, 0), PLUS),
    ("action.yaml", "added", (1, 0), PLUS),
    ("/abs/CODEOWNERS", "added", (1, 0), PLUS),
    (".github/settings.yml", "modified", (1, 0), PLUS),
    ("x/.github/settings.yml", "modified", (1, 0), PLUS),
    ("Cargo.lock", "modified", (40, 40), None),
    ("new/Cargo.toml", "renamed", (0, 0), None),
    ("requirements-dev.txt", "added", (1, 0), PLUS),
    ("app/App.csproj", "added", (1, 0), PLUS),
    ("vendor/lib.c", "added", (3, 0), PLUS),
    ("a/node_modules/b.js", "added", (1, 0), PLUS),
    ("skills/x/README.md", "modified", (1, 1), PLUS),
    (".claude/settings.json", "added", (1, 0), PLUS),
    ("deep/AGENTS.md", "modified", (1, 0), PLUS),
    ("pkg/GEMINI.md", "modified", (1, 0), PLUS),
    (".github/copilot-instructions.md", "added", (1, 0), PLUS),
    (".codex/config.toml", "modified", (1, 0), PLUS),
    ("web/app.min.js", "added", (9, 0), PLUS),
    ("src/__generated__/x.ts", "added", (1, 0), PLUS),
    ("proto/a_pb2_grpc.pyi", "added", (1, 0), PLUS),
    ("a.map\n", "added", (1, 0), PLUS),
    (
        "src/gen.go",
        "added",
        (3, 0),
        Some("@@ -0,0 +1,3 @@\n+package x\n+// Code generated by protoc. DO NOT EDIT.\n+x"),
    ),
    (
        "src/late.rs",
        "added",
        (2, 0),
        Some("@@ -0,0 +11,2 @@\n+// @generated\n+x"),
    ),
    ("img/logo.png", "added", (0, 0), None),
    (
        "dir/\u{e9}\u{1F600}\u{202e}\u{7f}\u{7}\"\\.txt",
        "added",
        (2, 0),
        PLUS,
    ),
    ("big.bin", "modified", (u64::MAX, 1), None),
    (".", "added", (1, 0), PLUS),
    ("./", "added", (1, 0), PLUS),
];

fn classified_files() -> Value {
    let mut files: Vec<Value> = CLASSIFIED
        .iter()
        .map(|&(path, status, counts, patch)| entry(path, status, counts, patch))
        .collect();
    let mut renamed = entry("SKILL.md", "renamed", (1, 0), PLUS);
    renamed["previous_filename"] = json!("old/Cargo.lock");
    files.push(renamed);
    Value::Array(files)
}

fn python() -> String {
    if let Ok(name) = env::var("PYTHON") {
        return name;
    }
    let candidates = if cfg!(windows) {
        ["python", "python3"]
    } else {
        ["python3", "python"]
    };
    candidates
        .into_iter()
        .find(|name| {
            Command::new(name)
                .args(["-c", "import sys; sys.exit(sys.version_info < (3, 8))"])
                .output()
                .is_ok_and(|out| out.status.success())
        })
        .expect("Python 3.8 or later on PATH, or named by PYTHON")
        .to_owned()
}

fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../skills/github-pr-review/scripts/pr_precheck.py")
        .canonicalize()
        .unwrap()
}

fn run_both(dir: &Path, args: &[&str]) -> (Output, Output) {
    let python = Command::new(python())
        .arg("-I")
        .arg(script())
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    let native = Command::new(env!("CARGO_BIN_EXE_github-skills"))
        .arg("pr-precheck")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    (python, native)
}

fn assert_same(case: &str, python: &Output, native: &Output) {
    let py_stderr = String::from_utf8_lossy(&python.stderr);
    assert_eq!(
        python.status.code(),
        native.status.code(),
        "{case}: exit codes differ; Python stderr: {py_stderr}"
    );
    if python.status.success() {
        let py_stdout = String::from_utf8(python.stdout.clone())
            .unwrap()
            .replace("\r\n", "\n");
        assert_eq!(
            py_stdout,
            String::from_utf8(native.stdout.clone()).unwrap(),
            "{case}: stdout differs"
        );
        assert!(native.stderr.is_empty(), "{case}");
    } else {
        assert_eq!(native.status.code(), Some(2), "{case}: {py_stderr}");
        assert!(
            python.stdout.is_empty() && native.stdout.is_empty(),
            "{case}"
        );
        for stderr in [&python.stderr, &native.stderr] {
            assert!(stderr.starts_with(b"pr_precheck:"), "{case}");
        }
    }
}

fn check_input(case: &str, bytes: &[u8]) {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("files.json"), bytes).unwrap();
    let (python, native) = run_both(dir.path(), &["files.json"]);
    assert_same(case, &python, &native);
}

#[test]
fn raw_inputs_match_the_python_script() {
    for (case, bytes) in RAW_CASES {
        check_input(case, bytes);
    }
}

#[test]
fn classifications_match_the_python_script() {
    let files = classified_files();
    check_input("classified", files.to_string().as_bytes());
    let pretty = serde_json::to_string_pretty(&json!([files, [], files])).unwrap();
    check_input("classified, slurped and pretty", pretty.as_bytes());
}

#[test]
fn ties_and_the_file_limit_match_the_python_script() {
    let many: Vec<Value> = (0..3000)
        .map(|n| entry(&format!("f{n}.rs"), "modified", (n % 4, 0), None))
        .collect();
    check_input("3,000 files", Value::Array(many).to_string().as_bytes());
}

#[test]
fn usage_errors_match_the_python_script() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("files.json"), "[]").unwrap();
    let cases: [&[&str]; 6] = [
        &[],
        &["files.json", "files.json"],
        &["../files.json"],
        &["./files.json"],
        &["missing.json"],
        &["."],
    ];
    for args in cases {
        let (python, native) = run_both(dir.path(), args);
        assert_same(&format!("{args:?}"), &python, &native);
    }
}
