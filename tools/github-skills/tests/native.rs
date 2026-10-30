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
fn skills_are_held_to_the_agent_skills_specification() {
    let root = tempfile::tempdir().unwrap();
    let file = ".claude/skills/github-repository-audit/SKILL.md";
    let check = |description: &str, extra: &str, body: &str| {
        let text = format!(
            "---\nname: github-repository-audit\ndescription: {description}\n{extra}---\n{body}\n"
        );
        write(root.path(), file, text.as_bytes());
        s::validate(root.path(), false).unwrap()
    };
    let long = format!("Use when {}", "x".repeat(1015));
    assert_eq!(check(&long, "", ""), Vec::<String>::new());
    assert_eq!(
        check(&format!("{long}x"), "", ""),
        [format!("{file}: description exceeds 1024 characters")]
    );
    assert_eq!(
        check("Use when testing.", "compatibility:\n", ""),
        [format!("{file}: compatibility must be 1 to 500 characters")]
    );
    assert_eq!(
        check("Use when a: b.", "", ""),
        [format!(
            "{file}: description must be a plain YAML value: no ': ', ' #', trailing ':' or leading indicator"
        )]
    );
    assert_eq!(
        check("Use when testing.", "", "[guide](references/absent.md)"),
        [format!(
            "{file}: referenced file 'references/absent.md' is not in the skill directory"
        )]
    );
    write(
        root.path(),
        ".claude/skills/github-repository-audit/references/guide.md",
        b"guide",
    );
    assert_eq!(
        check(
            "Use when testing.",
            "",
            "[guide](references/guide.md#a) [x](https://a.test/b)"
        ),
        Vec::<String>::new()
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

#[test]
fn root_operand_cannot_consume_an_option() {
    let root = tempfile::tempdir().unwrap();
    for option in [
        "--strict",
        "--sync",
        "--help",
        "--version",
        "--root",
        "--unknown",
        "-h",
        "-",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_github-skills"))
            .args(["validate-skills", "--root", option])
            .current_dir(root.path())
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2), "{option}");
        assert!(output.stdout.is_empty());
    }
    fs::create_dir(root.path().join("--strict")).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_github-skills"))
        .args(["validate-skills", "--root", "./--strict"])
        .current_dir(root.path())
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
}

#[test]
fn strict_diagnostics_keep_posix_string_order() {
    let root = tempfile::tempdir().unwrap();
    let name = s::APPROVED_NAMES[0];
    write(
        root.path(),
        &format!("{}/{name}/a/note", s::CANONICAL),
        b"safe",
    );
    write(
        root.path(),
        &format!("{}/{name}/a.txt", s::CANONICAL),
        b"safe",
    );
    let failures = s::validate(root.path(), true).unwrap();
    let files: Vec<_> = failures
        .iter()
        .filter(|line| line.contains("missing generated copy"))
        .collect();
    assert_eq!(
        files,
        vec![
            &format!("{}/{name}/a.txt: missing generated copy", s::RUNTIME),
            &format!("{}/{name}/a/note: missing generated copy", s::RUNTIME),
        ]
    );
}

#[cfg(unix)]
#[test]
fn strict_identity_keeps_raw_filename_bytes() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let root = tempfile::tempdir().unwrap();
    for name in s::APPROVED_NAMES {
        write(
            root.path(),
            &format!("{}/{name}/SKILL.md", s::CANONICAL),
            &skill(name),
        );
    }
    let name = s::APPROVED_NAMES[0];
    let source = root.path().join(s::CANONICAL).join(name).join("references");
    let copy = root.path().join(s::RUNTIME).join(name).join("references");
    fs::create_dir(&source).unwrap();
    fs::write(source.join("\u{fffd}.txt"), b"unicode").unwrap();
    fs::create_dir(source.join("\u{fffd}-dir")).unwrap();
    assert!(s::sync(root.path()).unwrap().is_empty());
    assert!(s::validate(root.path(), true).unwrap().is_empty());
    let raw = OsString::from_vec(b"\xff.txt".to_vec());
    let raw_dir = OsString::from_vec(b"\xff-dir".to_vec());
    fs::write(copy.join(&raw), b"surplus").unwrap();
    fs::create_dir(copy.join(&raw_dir)).unwrap();
    let failures = s::validate(root.path(), true).unwrap();
    assert_eq!(failures.len(), 2);
    assert!(
        failures
            .iter()
            .any(|line| line.ends_with(": unexpected generated file"))
    );
    assert!(
        failures
            .iter()
            .any(|line| line.ends_with(": unexpected generated directory"))
    );
    fs::write(source.join(&raw), b"raw bytes").unwrap();
    fs::create_dir(source.join(&raw_dir)).unwrap();
    assert!(s::sync(root.path()).unwrap().is_empty());
    assert!(s::validate(root.path(), true).unwrap().is_empty());
    assert_eq!(fs::read(copy.join(&raw)).unwrap(), b"raw bytes");
    assert_eq!(fs::read(copy.join("\u{fffd}.txt")).unwrap(), b"unicode");
}

// Expected digests come from the original Python skill_digest.
#[test]
fn skill_digest_matches_the_original_manifest() {
    let skill = tempfile::tempdir().unwrap();
    write(skill.path(), "SKILL.md", b"---\nname: x\n---\n");
    write(skill.path(), "references/checklist.md", b"check\n");
    write(skill.path(), "scripts/run.py", b"print(1)\r\n");
    fs::create_dir(skill.path().join("empty-dir")).unwrap();
    assert_eq!(
        s::skill_digest(skill.path()).unwrap(),
        "4fef86bfbdb5b2ade8525c6aae3bedb4ab831b7055c24851177aa921e8363ac2"
    );
    // Whole relative paths sort by code point, so "a b.md" < "a-b.md" < "a.md" < "a/z.md".
    let ordering = tempfile::tempdir().unwrap();
    for (name, bytes) in [
        ("SKILL.md", b"a".as_slice()),
        ("Z.md", b"b"),
        ("a.md", b"c"),
        ("a b.md", b"d"),
        ("\u{e9}.md", b"e"),
        ("a/z.md", b"f"),
        ("a-b.md", b"g"),
        ("empty.md", b""),
    ] {
        write(ordering.path(), name, bytes);
    }
    fs::create_dir(ordering.path().join("empty-dir")).unwrap();
    assert_eq!(
        s::skill_digest(ordering.path()).unwrap(),
        "fbae6503941b82904273110f8052214c41c96d7468c19a5d159a56580b192e70"
    );
}

fn record(root: &Path, name: &str, digest_lines: &str) {
    write(
        root,
        name,
        format!("Scenario: x\n\nAudit metadata:\n{digest_lines}- Run exit status: 0.\n").as_bytes(),
    );
}

#[test]
fn recorded_runs_are_checked_against_the_current_skill() {
    let root = tempfile::tempdir().unwrap();
    let name = "github-repository-audit";
    let run = format!("validation/forward/{name}.txt");
    let skill = root.path().join(s::CANONICAL).join(name);
    write(&skill, "SKILL.md", b"---\nname: x\n---\n");
    assert_eq!(s::check_runs(root.path()).unwrap(), (vec![], vec![]));
    let digest = s::skill_digest(&skill).unwrap();
    record(
        root.path(),
        &run,
        &format!("- Skill digest v1:\n  `{digest}`.\n"),
    );
    record(
        root.path(),
        &format!("validation/baselines/{name}.txt"),
        &format!("- Skill digest v1: `{digest}`.\n"),
    );
    write(root.path(), "validation/forward/unknown.md", b"ignored");
    assert_eq!(s::check_runs(root.path()).unwrap(), (vec![], vec![]));
    for lines in [
        format!("- Skill digest v2:\n  `{digest}`.\n"),
        format!("- Skill digest v1:\n  `{}`.\n", &digest[1..]),
        format!("- Skill digest v1: `{}`.\n", digest.to_uppercase()),
        format!("- Skill digest v1: `{digest}`.\n- Skill digest v1: `{digest}`.\n"),
        format!("- Skill digest v1: `{digest}`. \n"),
    ] {
        record(root.path(), &run, &lines);
        assert_eq!(
            s::check_runs(root.path()).unwrap(),
            (
                vec![format!(
                    "{run}: malformed skill digest; record exactly one '- Skill digest v1:' line"
                )],
                vec![]
            ),
            "{lines:?}"
        );
    }
    // "\b" needs a non-word character after the label for it to count.
    record(
        root.path(),
        &run,
        &format!("- Skill digests: `{digest}`.\n"),
    );
    let notice =
        format!("{run}: no skill digest recorded; its results are not tied to a skill revision");
    assert_eq!(s::check_runs(root.path()).unwrap(), (vec![], vec![notice]));
    record(
        root.path(),
        &run,
        &format!("- Skill digest v1:\r\n  `{digest}`.\r\n"),
    );
    write(&skill, "references/new.md", b"asset");
    let changed = s::skill_digest(&skill).unwrap();
    let stale = |run: &str| {
        format!(
            "{run}: stale; recorded skill digest {digest} differs from {}/{name} digest {changed}",
            s::CANONICAL
        )
    };
    assert_eq!(
        s::check_runs(root.path()).unwrap(),
        (
            vec![
                stale(&format!("validation/baselines/{name}.txt")),
                stale(&run)
            ],
            vec![]
        )
    );
    write(root.path(), "validation/baselines/unknown-skill.txt", b"");
    assert_eq!(
        s::check_runs(root.path()).unwrap(),
        (
            vec![
                stale(&format!("validation/baselines/{name}.txt")),
                "validation/baselines/unknown-skill.txt: names no approved skill".into(),
                stale(&run)
            ],
            vec![]
        )
    );
}

#[test]
fn recorded_run_suffix_follows_the_platform_glob() {
    let root = tempfile::tempdir().unwrap();
    record(
        root.path(),
        "validation/forward/github-readme-polish.TXT",
        "",
    );
    let found = s::check_runs(root.path()).unwrap().1.len();
    assert_eq!(found, usize::from(cfg!(windows)));
}

fn cli(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_github-skills"))
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

#[test]
fn cli_strict_checks_runs_and_prints_the_digest_line() {
    let root = tempfile::tempdir().unwrap();
    let name = "github-release-prep";
    let output = cli(root.path(), &["validate-skills", "--skill-digest", name]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "digest requires strict validation"
    );
    assert!(output.stdout.is_empty());
    for skill_name in s::APPROVED_NAMES {
        write(
            root.path(),
            &format!("{}/{skill_name}/SKILL.md", s::CANONICAL),
            &skill(skill_name),
        );
    }
    assert!(s::sync(root.path()).unwrap().is_empty());
    let run = format!("validation/baselines/{name}.txt");
    record(root.path(), &run, "");
    let output = cli(root.path(), &["validate-skills", "--strict"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{run}: no skill digest recorded; its results are not tied to a skill revision\n")
    );
    assert!(output.stderr.is_empty());
    let digest = s::skill_digest(&root.path().join(s::CANONICAL).join(name)).unwrap();
    let output = cli(root.path(), &["validate-skills", "--skill-digest", name]);
    assert_eq!(output.status.code(), Some(0));
    let line = format!("- Skill digest v1:\n  `{digest}`.\n");
    assert_eq!(String::from_utf8(output.stdout).unwrap(), line);
    record(root.path(), &run, &line);
    let output = cli(root.path(), &["validate-skills", "--strict"]);
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    record(root.path(), &run, &line.replace(&digest[..8], "00000000"));
    let output = cli(
        root.path(),
        &["validate-skills", "--strict", "--skill-digest", name],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty(), "no digest line after a stale run");
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with(&format!("{run}: stale;"))
    );
    for args in [
        ["validate-skills", "--skill-digest", "unknown"].as_slice(),
        &["validate-skills", "--skill-digest", "--strict"],
        &["check-public-files", "--skill-digest", name],
    ] {
        let output = cli(root.path(), args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty());
    }
}
