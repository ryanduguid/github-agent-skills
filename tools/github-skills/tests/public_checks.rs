//! Public-file scanner and `check-public-files` contracts, ported from the
//! retired tests/test_public_repository.py. Unsafe sample text is split with
//! concat! so this file does not match the rules it tests.

mod common;

use common::{link_file, write};
use github_skills::public_files as p;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn scan(root: &Path, relative: &str, bytes: &[u8]) -> Vec<String> {
    write(root, relative, bytes);
    p::scan_paths(root, &[relative.into()]).unwrap()
}

fn git(root: &Path, args: &[&str]) {
    let status = Command::new("git")
        .args(args)
        .current_dir(root)
        .status()
        .unwrap();
    assert!(status.success(), "git {args:?}");
}

fn utf16_le(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xff, 0xfe];
    bytes.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
    bytes
}

#[test]
fn unsafe_content_is_named_by_rule_without_echoing_it() {
    let root = tempfile::tempdir().unwrap();
    let public_url = b"https://api.github.com/users/ryanduguid/repos?per_page=100&type=all&sort=full_name&direction=asc\n";
    assert!(scan(root.path(), "public-source.md", public_url).is_empty());
    let cases: [(&str, &str, &str); 25] = [
        (
            "notes/transcript.txt",
            concat!("transcripts", "/session-42.txt\n"),
            "transcript-path",
        ),
        (
            "notes.txt",
            concat!(".superpowers/sdd/run", "/raw/output.txt\n"),
            "raw-sdd-path",
        ),
        (
            "notes.txt",
            concat!("C:", "\\Users\\Pat\\notes"),
            "private-user-path",
        ),
        (
            "notes.txt",
            concat!("/", "home/pat/notes"),
            "private-user-path",
        ),
        (
            "notes.txt",
            concat!("/", "Users/pat/notes"),
            "private-user-path",
        ),
        (
            "notes.txt",
            concat!("db", "_password", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("deploy", "_secret", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("auth", "_token", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("api", "_key", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("access", "_key", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("client", "_secret", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("private", "_key", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("client", "_data", ": ", "do-not-echo"),
            "client-data",
        ),
        (
            "notes.txt",
            concat!("stripe", "_key", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("customer", "_key", " = ", "do-not-echo"),
            "credential-assignment",
        ),
        (
            "notes.json",
            concat!("{\"api", "_key\"", ":\"do-not-echo\"}"),
            "credential-assignment",
        ),
        (
            "notes.json",
            concat!("{\"to", "ken\"", ":\"do-not-echo\"}"),
            "credential-assignment",
        ),
        (
            "notes.toml",
            concat!("\"access", "_key\"", " = \"do-not-echo\""),
            "credential-assignment",
        ),
        (
            "notes.yaml",
            concat!("'client", "_secret'", ": 'do-not-echo'"),
            "credential-assignment",
        ),
        (
            "notes.txt",
            concat!("C:", "/", "Users/Pat/notes"),
            "private-user-path",
        ),
        (
            "notes.txt",
            concat!("file:", "///", "Users/pat/notes"),
            "private-user-path",
        ),
        (
            "notes.txt",
            concat!("https://example.test/download?path=", "/", "home/pat/notes"),
            "private-user-path",
        ),
        (
            "notes.txt",
            "\\\\server\\Users\\Pat\\notes",
            "private-user-path",
        ),
        (
            "notes.txt",
            concat!(
                "https://api.github.com/users/ryanduguid/repos\nLocal copy: ",
                "C:",
                "\\Users\\Pat\\notes"
            ),
            "private-user-path",
        ),
        (
            "notes.txt",
            concat!(
                "https://api.github.com/users/ryanduguid/repos\nLocal copy: ",
                "\\\\server\\Users\\Pat\\notes"
            ),
            "private-user-path",
        ),
    ];
    for (relative, content, rule) in cases {
        let failures = scan(root.path(), relative, content.as_bytes());
        assert_eq!(failures, [format!("{relative}: {rule}")], "{content:?}");
        assert!(!failures.concat().contains("do-not-echo"));
    }
}

#[test]
fn private_configuration_and_client_data_paths_are_refused() {
    let root = tempfile::tempdir().unwrap();
    let names = [
        ".env",
        ".netrc",
        ".npmrc",
        "credentials.json",
        ".ssh/config",
        ".aws/credentials",
        "home/pat/.config/settings",
    ];
    for name in names {
        write(root.path(), name, b"safe placeholder\n");
    }
    let paths: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();
    assert_eq!(
        p::scan_paths(root.path(), &paths).unwrap(),
        names.map(|name| format!("{name}: private-config"))
    );
    let names = ["client-data.csv", "exports/customer_records.csv"];
    for name in names {
        write(root.path(), name, b"safe\n");
    }
    let paths: Vec<PathBuf> = names.iter().map(PathBuf::from).collect();
    assert_eq!(
        p::scan_paths(root.path(), &paths).unwrap(),
        names.map(|name| format!("{name}: client-data"))
    );
    let policy = b"Do not submit customer data or client records.\n";
    assert!(scan(root.path(), "policy.md", policy).is_empty());
}

#[test]
fn policy_prose_and_public_examples_pass() {
    let root = tempfile::tempdir().unwrap();
    for content in [
        concat!("public", "_key", " = ", "example-value"),
        concat!(
            "Reject quoted credential keys such as \"api",
            "_key\" or \"to",
            "ken\" when they are assigned values.\n"
        ),
    ] {
        assert!(
            scan(root.path(), "policy.md", content.as_bytes()).is_empty(),
            "{content}"
        );
    }
}

#[test]
fn workflow_expressions_and_permissions_are_not_credentials() {
    let root = tempfile::tempdir().unwrap();
    let workflow = ".github/workflows/policy.yml";
    let reference = concat!("          GH_TOKEN", ": ", "${{ github.token }}\n");
    let named = concat!("      GH_TOKEN", ": ", "${{ secrets.TRIGGER_TOKEN }}\n");
    let permission = concat!("      id-to", "ken", ": write # Publish results.\n");
    let prefixed = concat!(
        "      OPENAI",
        "_API",
        "_KEY",
        ": ",
        "${{ secrets.OPENAI_API_KEY }}\n"
    );
    let cases = [
        (workflow, reference.to_owned(), true),
        (workflow, named.to_owned(), true),
        (workflow, permission.to_owned(), true),
        (workflow, prefixed.to_owned(), true),
        ("notes.md", reference.to_owned(), false),
        ("notes.md", named.to_owned(), false),
        ("notes.md", permission.to_owned(), false),
        (
            workflow,
            format!("{}-literal\n", reference.trim_end()),
            false,
        ),
        (workflow, format!("{}-literal\n", named.trim_end()), false),
        (
            workflow,
            concat!("      id-to", "ken", ": do-not-echo\n").to_owned(),
            false,
        ),
        (
            workflow,
            format!(
                "{reference}{}",
                concat!("          api", "_key", ": ", "do-not-echo\n")
            ),
            false,
        ),
    ];
    for (relative, content, allowed) in cases {
        let expected = if allowed {
            vec![]
        } else {
            vec![format!("{relative}: credential-assignment")]
        };
        assert_eq!(
            scan(root.path(), relative, content.as_bytes()),
            expected,
            "{relative} {content:?}"
        );
    }
    for name in [
        concat!("OPENAI", "_API", "_KEY"),
        concat!("SSH", "_PRIVATE", "_KEY"),
    ] {
        assert_eq!(
            scan(
                root.path(),
                "notes.md",
                format!("{name}=do-not-echo\n").as_bytes()
            ),
            ["notes.md: credential-assignment"]
        );
    }
}

#[test]
fn quoted_client_data_keys_are_refused_in_ordinary_files() {
    let root = tempfile::tempdir().unwrap();
    for (relative, content) in [
        (
            "records.json",
            concat!("{\"", "client", "_data\": \"do-not-echo\"}\n"),
        ),
        (
            "config.yaml",
            concat!("  \"", "customer", "-records\": do-not-echo\n"),
        ),
        (
            "config.toml",
            concat!("[section]\n\"", "client", "_export\" = \"do-not-echo\"\n"),
        ),
    ] {
        assert_eq!(
            scan(root.path(), relative, content.as_bytes()),
            [format!("{relative}: client-data")]
        );
    }
}

#[test]
fn link_text_is_scanned_without_following_the_target() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "target.txt",
        concat!("api", "_key", " = ", "do-not-echo\n").as_bytes(),
    );
    for (name, target, expected) in [
        ("safe.link", "target.txt", vec![]),
        (
            "unsafe.link",
            concat!("/home", "/pat/notes"),
            vec!["unsafe.link: private-user-path".to_owned()],
        ),
    ] {
        if !link_file(Path::new(target), &root.path().join(name)) {
            eprintln!("skipped: file links need a privilege this account lacks");
            return;
        }
        assert_eq!(
            p::scan_paths(root.path(), &[name.into()]).unwrap(),
            expected
        );
    }
}

#[test]
fn encodings_are_decoded_or_classified() {
    let root = tempfile::tempdir().unwrap();
    let credential = concat!("api", "_key", " = ", "do-not-echo");
    assert_eq!(
        scan(root.path(), "notes.txt", b"note=\xffvalue\n"),
        ["notes.txt: undecodable-text"]
    );
    for little in [true, false] {
        let mut bytes = if little {
            vec![0xff, 0xfe, 0, 0]
        } else {
            vec![0, 0, 0xfe, 0xff]
        };
        for c in credential.chars() {
            let c = u32::from(c);
            bytes.extend(if little {
                c.to_le_bytes()
            } else {
                c.to_be_bytes()
            });
        }
        assert_eq!(
            scan(root.path(), "utf32.txt", &bytes),
            ["utf32.txt: credential-assignment"]
        );
    }
    assert_eq!(
        scan(root.path(), "utf16.txt", &utf16_le(credential)),
        ["utf16.txt: credential-assignment"]
    );
    for (relative, bytes) in [
        ("embedded-null.txt", b"note=\0value\n".as_slice()),
        ("bad32.txt", b"\xff\xfe\0\0\xff\xff\xff\xff"),
        ("bad16.txt", b"\xff\xfe\x00\xd8"),
    ] {
        assert_eq!(
            scan(root.path(), relative, bytes),
            [format!("{relative}: unclassifiable-text")]
        );
    }
    for (relative, bytes) in [
        ("image.bin", b"\0\x01\x02\x03\0\xfe".as_slice()),
        ("nul.bin", b"\0\0"),
        ("empty.txt", b""),
    ] {
        assert!(scan(root.path(), relative, bytes).is_empty(), "{relative}");
    }
}

#[test]
fn an_unreadable_tracked_path_is_reported_and_scanning_continues() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("gitlink")).unwrap();
    write(
        root.path(),
        "notes.md",
        concat!("api", "_key", " = ", "do-not-echo\n").as_bytes(),
    );
    let failures = p::scan_paths(root.path(), &["gitlink".into(), "notes.md".into()]).unwrap();
    assert_eq!(failures.len(), 2);
    assert!(
        failures[0].starts_with("gitlink: unreadable (") && failures[0].ends_with("Error)"),
        "{}",
        failures[0]
    );
    assert_eq!(failures[1], "notes.md: credential-assignment");
}

#[test]
fn only_files_git_tracks_are_scanned() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    let report = ".superpowers/sdd/run/task-4-report.md";
    let credential = concat!("auth", "_token", " = ", "do-not-echo\n");
    write(root.path(), report, credential.as_bytes());
    write(root.path(), "untracked.md", credential.as_bytes());
    git(root.path(), &["add", report]);
    let failures = p::tracked_failures(root.path()).unwrap();
    assert_eq!(failures, [format!("{report}: credential-assignment")]);
    assert!(!failures.concat().contains("do-not-echo"));
}

#[test]
fn cli_check_public_files_reports_every_failure_in_order() {
    let root = tempfile::tempdir().unwrap();
    git(root.path(), &["init", "-q"]);
    let readme = format!(
        "# Fixture\n\n## Repository checks\n\n```shell\ngit clone {}\ncd {}\npython -m unittest\n```\n",
        p::CLONE_URL,
        p::REPOSITORY_NAME
    );
    let workflow = |access: &str| {
        format!(
            "permissions:\n  contents: {access}\n\njobs:\n  validate:\n    steps:\n      - name: Unit tests\n        run: python -m unittest\n"
        )
    };
    let workflow_path = ".github/workflows/validate.yml";
    write(root.path(), "README.md", readme.as_bytes());
    write(root.path(), workflow_path, workflow("read").as_bytes());
    git(root.path(), &["add", "-A"]);
    let check = || {
        Command::new(env!("CARGO_BIN_EXE_github-skills"))
            .args(["check-public-files", "--root"])
            .arg(root.path())
            .output()
            .unwrap()
    };
    let output = check();
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty() && output.stderr.is_empty());
    write(
        root.path(),
        "notes.md",
        concat!("api", "_key", " = ", "do-not-echo\n").as_bytes(),
    );
    write(root.path(), "README.md", b"# Fixture\n");
    write(root.path(), workflow_path, workflow("write").as_bytes());
    git(root.path(), &["add", "notes.md"]);
    let output = check();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(
        stderr.lines().collect::<Vec<_>>(),
        [
            "notes.md: credential-assignment",
            "README.md: Repository checks commands differ from the checks the Validate workflow runs",
            ".github/workflows/validate.yml: workflow permissions are not read-only contents",
        ]
    );
    assert!(!stderr.contains("do-not-echo"));
}

#[test]
fn the_readme_lists_the_validate_job_checks_and_any_change_is_caught() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let readme = github_skills::read_text(&repository.join("README.md")).unwrap();
    let workflow =
        github_skills::read_text(&repository.join(".github/workflows/validate.yml")).unwrap();
    // The validate job is the gate, so a check moved out of it must fail here too.
    let checker = "cargo run --manifest-path tools/github-skills/Cargo.toml --locked --";
    assert_eq!(
        p::supported_quick_start(&workflow).unwrap()[2..],
        [
            "python -m unittest discover -s tests -v".to_owned(),
            "rustup toolchain install".to_owned(),
            format!("{checker} validate-skills --strict"),
            format!("{checker} check-public-files"),
        ]
    );
    assert!(
        p::quick_start_failures(&readme, &workflow)
            .unwrap()
            .is_empty()
    );
    let commands = p::quick_start_commands(&readme).unwrap();
    let last = commands.len() - 1;
    let with = |index: usize, line: &str| {
        let mut changed = commands.clone();
        changed[index] = line.to_owned();
        changed
    };
    let mut reordered = commands.clone();
    reordered.swap(0, 1);
    let mut unknown = commands.clone();
    unknown.push("python scripts/missing.py".into());
    for (name, changed) in [
        (
            "wrong clone",
            with(
                0,
                "git clone https://github.com/ryanduguid/missing-skills.git",
            ),
        ),
        ("wrong directory", with(1, "cd missing-skills")),
        (
            "extra operand",
            with(last, &format!("{} --input missing.json", commands[last])),
        ),
        ("omitted line", commands[..last].to_vec()),
        ("reordered line", reordered),
        ("unknown command", unknown),
    ] {
        let changed = readme.replace(&commands.join("\n"), &changed.join("\n"));
        assert_ne!(changed, readme, "{name}");
        assert!(
            !p::quick_start_failures(&changed, &workflow)
                .unwrap()
                .is_empty(),
            "{name}"
        );
    }
    assert!(
        p::quick_start_commands("# Title\n\nNo quick start here.\n")
            .unwrap()
            .is_empty()
    );
}
