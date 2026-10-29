use github_skills::{public_files as p, skills as s};
use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn write(root: &Path, name: &str, bytes: &[u8]) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn skill(name: &str) -> Vec<u8> {
    format!("---\nname: {name}\ndescription: Use when testing.\n---\n# Skill\n").into_bytes()
}

#[test]
fn frozen_original_scanner_and_decoder_oracles() {
    let cases: Value = serde_json::from_str(include_str!("fixtures/public-cases.json")).unwrap();
    let root = tempfile::tempdir().unwrap();
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let name = case["path"].as_str().unwrap();
        let bytes: Vec<_> = case["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as u8)
            .collect();
        if !case["path_only"].as_bool().unwrap_or(false) {
            write(root.path(), name, &bytes);
        }
        let failures = p::scan_paths(root.path(), &[name.into()]).unwrap();
        assert_eq!(
            serde_json::json!(failures),
            case["expected"],
            "scanner case {index}"
        );
        let (text, failure) = p::decoded_text(&bytes);
        assert_eq!(
            serde_json::json!(text.map(String::into_bytes)),
            case["decoded_bytes"],
            "decoded bytes case {index}"
        );
        assert_eq!(
            serde_json::json!(failure),
            case["decoding_failure"],
            "decoding failure case {index}"
        );
    }
}

#[test]
fn scanner_can_check_the_full_encoded_corpus_without_an_exemption() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "golden.json",
        include_bytes!("fixtures/public-cases.json"),
    );
    assert!(
        p::scan_paths(root.path(), &["golden.json".into()])
            .unwrap()
            .is_empty()
    );
}

#[test]
fn strict_tree_parity_and_incremental_subset() {
    let root = tempfile::tempdir().unwrap();
    assert!(s::validate(root.path(), false).unwrap().is_empty());
    assert_eq!(s::validate(root.path(), true).unwrap().len(), 5);
    for name in s::APPROVED_NAMES {
        write(
            root.path(),
            &format!("{}/{name}/SKILL.md", s::CANONICAL),
            &skill(name),
        );
        write(
            root.path(),
            &format!("{}/{name}/references/data.bin", s::CANONICAL),
            b"\0\xff\x10",
        );
    }
    assert!(s::validate(root.path(), false).unwrap().is_empty());
    assert!(s::sync(root.path()).unwrap().is_empty());
    assert!(s::validate(root.path(), true).unwrap().is_empty());
    let name = s::APPROVED_NAMES[0];
    fs::remove_file(
        root.path()
            .join(format!("{}/{name}/references/data.bin", s::RUNTIME)),
    )
    .unwrap();
    write(
        root.path(),
        &format!("{}/{name}/references/extra.txt", s::RUNTIME),
        b"safe",
    );
    assert_eq!(
        s::validate(root.path(), true).unwrap(),
        vec![
            format!(
                "{}/{name}/references/data.bin: missing generated copy",
                s::RUNTIME
            ),
            format!(
                "{}/{name}/references/extra.txt: unexpected generated file",
                s::RUNTIME
            )
        ]
    );
    assert!(s::sync(root.path()).unwrap().is_empty());
    assert!(s::validate(root.path(), true).unwrap().is_empty());
}

#[test]
fn sync_refusal_preserves_generated_files() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), ".agents/skills/sentinel.txt", b"before");
    assert_eq!(
        s::sync(root.path()).unwrap(),
        vec![".claude/skills: is missing"]
    );
    assert_eq!(
        fs::read(root.path().join(".agents/skills/sentinel.txt")).unwrap(),
        b"before"
    );
    write(root.path(), ".claude/skills", b"file");
    assert_eq!(
        s::sync(root.path()).unwrap(),
        vec![".claude/skills: is not an ordinary directory"]
    );
    assert_eq!(
        fs::read(root.path().join(".agents/skills/sentinel.txt")).unwrap(),
        b"before"
    );
}

#[test]
fn metadata_is_literal_and_uses_python_line_separators() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("SKILL.md");
    fs::write(&path,"---\u{2028}name: first\u{2028} name: ignored\u{2028}name: last\u{2028}description:\u{1f}Use when testing.\u{1c}---").unwrap();
    let metadata = s::frontmatter(&path).unwrap().unwrap();
    assert_eq!(metadata["name"], "last");
    assert_eq!(metadata["description"], "Use when testing.");
    fs::write(&path, b"\xff\xfe").unwrap();
    assert_eq!(
        s::frontmatter(&path).unwrap(),
        Err("not valid UTF-8 text".into())
    );
}

#[test]
fn placeholder_boundaries_use_python_alphanumeric_rules() {
    let root = tempfile::tempdir().unwrap();
    let name = s::APPROVED_NAMES[0];
    let relative = format!("{}/{name}/SKILL.md", s::CANONICAL);
    for (suffix, forbidden) in [
        ("\u{301}TODO\u{301}", true),
        ("éTODOé", false),
        ("_TODO_", false),
        ("scaffoldİng", true),
        ("scaffoldıng", true),
    ] {
        let mut content = skill(name);
        content.extend(suffix.as_bytes());
        write(root.path(), &relative, &content);
        assert_eq!(
            !s::validate(root.path(), false).unwrap().is_empty(),
            forbidden,
            "{suffix}"
        );
    }
}

#[cfg(unix)]
#[test]
fn links_are_refused_before_sync_and_scanned_as_text() {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    write(
        root.path(),
        ".claude/skills/github-repository-audit/SKILL.md",
        &skill("github-repository-audit"),
    );
    write(root.path(), ".agents/skills/sentinel.txt", b"before");
    write(outside.path(), "sentinel.txt", b"outside");
    symlink(
        outside.path(),
        root.path()
            .join(".claude/skills/github-repository-audit/references"),
    )
    .unwrap();
    assert_eq!(
        s::sync(root.path()).unwrap(),
        vec![
            ".claude/skills/github-repository-audit/references: is a link, not an ordinary file or directory"
        ]
    );
    assert_eq!(
        fs::read(root.path().join(".agents/skills/sentinel.txt")).unwrap(),
        b"before"
    );
    assert_eq!(
        fs::read(outside.path().join("sentinel.txt")).unwrap(),
        b"outside"
    );
    symlink(
        ["/home", "/fabricated/note"].concat(),
        root.path().join("unsafe.link"),
    )
    .unwrap();
    assert_eq!(
        p::scan_paths(root.path(), &["unsafe.link".into()]).unwrap(),
        vec!["unsafe.link: private-user-path"]
    );
}

#[cfg(unix)]
#[test]
fn byte_copy_does_not_propagate_source_mode() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let source = ".claude/skills/github-repository-audit/SKILL.md";
    write(root.path(), source, &skill("github-repository-audit"));
    fs::set_permissions(root.path().join(source), fs::Permissions::from_mode(0o400)).unwrap();
    s::sync(root.path()).unwrap();
    let mode = fs::metadata(
        root.path()
            .join(".agents/skills/github-repository-audit/SKILL.md"),
    )
    .unwrap()
    .permissions()
    .mode();
    assert_ne!(mode & 0o200, 0);
}

#[cfg(unix)]
#[test]
fn fifo_sync_refuses_within_deadline_and_preserves_generated_bytes() {
    use std::{
        process::Stdio,
        thread,
        time::{Duration, Instant},
    };
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        ".claude/skills/github-repository-audit/SKILL.md",
        &skill("github-repository-audit"),
    );
    write(root.path(), ".agents/skills/sentinel.txt", b"before");
    assert!(
        Command::new("mkfifo")
            .arg(
                root.path()
                    .join(".claude/skills/github-repository-audit/pipe")
            )
            .status()
            .unwrap()
            .success()
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_github-skills"))
        .args(["validate-skills", "--sync", "--root"])
        .arg(root.path())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("synchronisation blocked on a FIFO");
        }
        thread::sleep(Duration::from_millis(10));
    };
    assert_eq!(status.code(), Some(2));
    assert_eq!(
        fs::read(root.path().join(".agents/skills/sentinel.txt")).unwrap(),
        b"before"
    );
}

#[test]
fn quick_start_and_permissions_contract() {
    let workflow = "permissions:\n  contents: read\njobs:\n  validate:\n    steps:\n      - name: Check\n        run: python check.py\n";
    let readme = format!(
        "## Quick start\n```shell\ngit clone {}\ncd {}\npython check.py\n```\n## Other\n```shell\nignored\n```\n",
        p::CLONE_URL,
        p::REPOSITORY_NAME
    );
    assert!(
        p::quick_start_failures(&readme, workflow)
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        p::permissions_block(workflow).unwrap(),
        Some(vec!["  contents: read".into()])
    );
    assert_eq!(
        p::quick_start_failures(
            &readme.replace("python check.py", "python wrong.py"),
            workflow
        )
        .unwrap()
        .len(),
        1
    );
}

#[test]
fn cli_uses_explicit_root_and_reports_failures_without_stdout() {
    let root = tempfile::tempdir().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_github-skills"))
        .args(["validate-skills", "--strict", "--root"])
        .arg(root.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(String::from_utf8(output.stderr).unwrap().lines().count(), 5);
    let output = Command::new(env!("CARGO_BIN_EXE_github-skills"))
        .args(["check-public-files", "--sync"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}
