// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Skill validation and synchronisation contracts, ported from the retired
//! tests/test_validate_skills.py and tests/test_sync_skills.py. Link cases use
//! NTFS junctions on Windows and symbolic links elsewhere.

mod common;

use common::{cli, link_directory, link_file, write};
use github_skills::skills as s;
use std::{
    fs,
    path::{Path, PathBuf},
};

const NAME: &str = "github-repository-audit";
const SKILL_FILE: &str = ".claude/skills/github-repository-audit/SKILL.md";
const LINK: &str = "is a link, not an ordinary file or directory";

fn skill_text(name: &str, description: &str) -> String {
    format!("---\nname: {name}\ndescription: Use when {description}.\n---\n\n# {name}\n")
}

/// A canonical skill and its matching generated copy.
fn valid_skill(root: &Path, name: &str) {
    let text = skill_text(name, "assessing a repository");
    write(
        root,
        &format!("{}/{name}/SKILL.md", s::CANONICAL),
        text.as_bytes(),
    );
    write(
        root,
        &format!("{}/{name}/SKILL.md", s::RUNTIME),
        text.as_bytes(),
    );
}

fn valid_skill_set(root: &Path) {
    for name in s::APPROVED_NAMES {
        valid_skill(root, name);
    }
}

fn skill_with(root: &Path, description: &str, extra: &str, body: &str) -> Vec<String> {
    let text = format!("---\nname: {NAME}\ndescription: {description}\n{extra}---\n{body}");
    write(root, SKILL_FILE, text.as_bytes());
    s::validate(root, false).unwrap()
}

fn incremental_and_strict(root: &Path) -> [Vec<String>; 2] {
    [false, true].map(|strict| s::validate(root, strict).unwrap())
}

#[test]
fn approved_names_meet_the_specification() {
    for name in s::APPROVED_NAMES {
        assert!(name.len() <= 64, "{name}");
        // Lowercase letters and digits in runs joined by single hyphens.
        assert!(
            name.split('-').all(|part| !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())),
            "{name}"
        );
    }
}

#[test]
fn frontmatter_needs_both_delimiters_and_ignores_other_lines() {
    let root = tempfile::tempdir().unwrap();
    for text in [
        "name: github-repository-audit\n",
        "---\nname: github-repository-audit\ndescription: Use when auditing.\n",
    ] {
        write(root.path(), SKILL_FILE, text.as_bytes());
        assert_eq!(
            s::validate(root.path(), false).unwrap(),
            [format!("{SKILL_FILE}: missing YAML frontmatter delimiters")],
            "{text:?}"
        );
    }
    let text = skill_text(NAME, "assessing a repository").replacen(
        "---\nname:",
        "---\nlicence: MIT\nno separator here\nname:",
        1,
    );
    write(root.path(), SKILL_FILE, text.as_bytes());
    write(
        root.path(),
        &format!("{}/{NAME}/SKILL.md", s::RUNTIME),
        text.as_bytes(),
    );
    assert!(s::validate(root.path(), false).unwrap().is_empty());
}

#[test]
fn each_skill_rule_reports_its_own_failure() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir_all(root.path().join(s::CANONICAL).join(NAME)).unwrap();
    assert_eq!(
        s::validate(root.path(), false).unwrap(),
        [format!("{}/{NAME}: missing SKILL.md", s::CANONICAL)]
    );
    let mut utf16 = vec![0xff, 0xfe];
    utf16.extend(
        skill_text(NAME, "x")
            .encode_utf16()
            .flat_map(u16::to_le_bytes),
    );
    let cases = [
        (utf16, "not valid UTF-8 text"),
        (b"\xff\xfe\x00garbage".to_vec(), "not valid UTF-8 text"),
        (
            b"---\ndescription: Use when assessing a repository.\n---\n".to_vec(),
            "missing name",
        ),
        (
            b"---\nname: github-repository-audit\n---\n".to_vec(),
            "missing description",
        ),
        (
            skill_text("github-readme-polish", "polishing a README").into_bytes(),
            "name must match directory name",
        ),
        (
            b"---\nname: github-repository-audit\ndescription: Assess a repository.\n---\n"
                .to_vec(),
            "description must start with 'Use when'",
        ),
        (
            format!(
                "{}\nTODO: finish\n",
                skill_text(NAME, "assessing a repository")
            )
            .into_bytes(),
            "forbidden placeholder 'TODO'",
        ),
    ];
    for (bytes, failure) in cases {
        write(root.path(), SKILL_FILE, &bytes);
        assert_eq!(
            s::validate(root.path(), false).unwrap(),
            [format!("{SKILL_FILE}: {failure}")]
        );
    }
    fs::remove_dir_all(root.path().join(s::CANONICAL)).unwrap();
    write(
        root.path(),
        ".claude/skills/not-approved/SKILL.md",
        skill_text("not-approved", "doing something").as_bytes(),
    );
    assert_eq!(
        s::validate(root.path(), false).unwrap(),
        [".claude/skills/not-approved: unexpected skill name"]
    );
}

#[test]
fn hidden_characters_are_refused_with_their_position() {
    let root = tempfile::tempdir().unwrap();
    let check = |text: &str| {
        write(root.path(), SKILL_FILE, text.as_bytes());
        s::validate(root.path(), false).unwrap()
    };
    let hidden = |c: char, line: usize, column: usize| {
        [format!(
            "{SKILL_FILE}: invisible or bidirectional control character U+{:04X} at line {line}, column {column}",
            u32::from(c)
        )]
    };
    // Six lines, so the body text below starts on line 7.
    let clean = skill_text(NAME, "assessing a repository");
    assert!(check(&format!("{clean}Run it now.\n")).is_empty());
    // Neighbours of the refused ranges.
    for c in ['\u{200a}', '\u{2010}', '\u{205f}', 'é'] {
        assert!(
            check(&format!("{clean}Run it{c} now.\n")).is_empty(),
            "{c:?}"
        );
    }
    // Both ends of each class: the Arabic letter mark, zero-width characters and
    // directional marks, embeddings and overrides, the word joiner and invisible
    // operators, isolates, the byte order mark and the tag block. Then the soft
    // hyphen, combining grapheme joiner, Mongolian vowel separator and variation
    // selectors, which are as invisible.
    for c in [
        '\u{61c}',
        '\u{200b}',
        '\u{200f}',
        '\u{202a}',
        '\u{202e}',
        '\u{2060}',
        '\u{2064}',
        '\u{2066}',
        '\u{2069}',
        '\u{feff}',
        '\u{e0000}',
        '\u{e007f}',
        '\u{ad}',
        '\u{34f}',
        '\u{180e}',
        '\u{fe0f}',
        '\u{e0100}',
        '\u{e01ef}',
    ] {
        assert_eq!(check(&format!("{clean}Run it{c} now.\n")), hidden(c, 7, 7));
    }
    // Lines end at LF, CR or CRLF; columns count code points, so an astral
    // character is one column. Only the first hidden character is reported.
    assert_eq!(
        check(&format!(
            "{}\r\n\r\u{1f600}\u{2067}x\u{202e}\n",
            clean.replace('\n', "\r\n")
        )),
        hidden('\u{2067}', 9, 2)
    );
    // Tag characters can spell out text no reviewer sees: here 'hello friend'.
    let smuggled: String = "hello friend"
        .chars()
        .map(|c| char::from_u32(0xe0000 + u32::from(c)).unwrap())
        .collect();
    assert_eq!(
        check(&format!("{clean}Hello{smuggled}\n")),
        hidden('\u{e0068}', 7, 6)
    );
    // So do emoji built with a zero width joiner, VARIATION SELECTOR-16 or tag
    // characters, such as Scotland's flag; skill instructions are plain text.
    for (emoji, c) in [
        ("\u{1f469}\u{200d}\u{1f4bb}", '\u{200d}'),
        ("\u{26a0}\u{fe0f}", '\u{fe0f}'),
        (
            "\u{1f3f4}\u{e0067}\u{e0062}\u{e0073}\u{e0063}\u{e0074}\u{e007f}",
            '\u{e0067}',
        ),
    ] {
        assert_eq!(check(&format!("{clean}{emoji}\n")), hidden(c, 7, 2));
    }
    // Reported before the frontmatter checks it would otherwise mislead.
    assert_eq!(
        check(&clean.replacen(NAME, &format!("{NAME}\u{200b}"), 1)),
        hidden('\u{200b}', 2, 7 + NAME.len())
    );
    assert_eq!(check(&format!("\u{feff}{clean}")), hidden('\u{feff}', 1, 1));
}

#[test]
fn hidden_characters_cover_unicode_default_ignorables() {
    // Listed code points a skill may still use. Empty: skill instructions are
    // plain text, and no skill needs one.
    const ALLOWED: [char; 0] = [];
    let mut ranges = Vec::new();
    for line in include_str!("fixtures/hidden-code-points.txt").lines() {
        if line.starts_with('#') {
            continue;
        }
        let range = line.split(';').next().unwrap().trim();
        let (first, last) = range.split_once("..").unwrap_or((range, range));
        let code = |hex| u32::from_str_radix(hex, 16).unwrap();
        ranges.push(code(first)..=code(last));
    }
    // 4,174 Default_Ignorable_Code_Point and 12 Bidi_Control code points.
    assert_eq!(
        ranges.iter().map(|r| r.clone().count()).sum::<usize>(),
        4186
    );
    // Every listed character is refused and every other one is not.
    for c in '\0'..=char::MAX {
        let listed = ranges.iter().any(|r| r.contains(&u32::from(c)));
        assert_eq!(
            s::is_hidden(c),
            listed && !ALLOWED.contains(&c),
            "U+{:04X}",
            u32::from(c)
        );
    }
}

#[test]
fn compatibility_length_follows_the_specification() {
    let root = tempfile::tempdir().unwrap();
    let description = "Use when assessing a repository.";
    let extra = |value: &str| format!("compatibility: {value}\n");
    assert!(skill_with(root.path(), description, &extra(&"x".repeat(500)), "").is_empty());
    for value in [String::new(), "x".repeat(501)] {
        assert_eq!(
            skill_with(root.path(), description, &extra(&value), ""),
            [format!(
                "{SKILL_FILE}: compatibility must be 1 to 500 characters"
            )],
            "{}",
            value.len()
        );
    }
}

#[test]
fn values_a_yaml_parser_would_read_differently_are_refused() {
    let root = tempfile::tempdir().unwrap();
    let failure = "must be a plain YAML string: no ': ', ' #', trailing ':', leading indicator or implicit type";
    for description in ["Use when a: b.", "Use when a #b.", "Use when a:"] {
        assert_eq!(
            skill_with(root.path(), description, "", ""),
            [format!("{SKILL_FILE}: description {failure}")],
            "{description}"
        );
    }
    let description = "Use when assessing a repository.";
    for value in [
        ">-",
        "|",
        "\"quoted\"",
        "'quoted'",
        "[a]",
        "{a}",
        "*a",
        "&a",
        "!a",
        "a: b",
        "true",
        "False",
        "null",
        "~",
        "yes",
        "Off",
        "123",
        "-1",
        "1.5",
        "1e3",
        "0x1F",
        ".inf",
        "2026-10-30",
        "- Linux",
        "-",
        "? a",
    ] {
        assert_eq!(
            skill_with(
                root.path(),
                description,
                &format!("compatibility: {value}\n"),
                ""
            ),
            [format!("{SKILL_FILE}: compatibility {failure}")],
            "{value}"
        );
    }
    assert!(skill_with(root.path(), "Use when a:b or c#d.", "", "").is_empty());
    for value in [
        "Requires git and Python 3.14+",
        "Linux",
        "3.14+",
        "-Linux",
        "2 cores",
        "true story",
        "no-network",
    ] {
        assert!(
            skill_with(
                root.path(),
                description,
                &format!("compatibility: {value}\n"),
                ""
            )
            .is_empty(),
            "{value}"
        );
    }
}

#[test]
fn referenced_files_must_be_in_the_skill_directory() {
    let root = tempfile::tempdir().unwrap();
    let description = "Use when assessing a repository.";
    write(
        root.path(),
        ".claude/skills/github-repository-audit/references/guide.md",
        b"guide\n",
    );
    write(
        root.path(),
        ".claude/skills/github-repository-audit/scripts/run.py",
        b"pass\n",
    );
    write(root.path(), ".claude/skills/outside.md", b"outside\n");
    for body in [
        "[guide](references/guide.md)",
        "[guide](references/guide.md#usage)",
        "[guide](references/guide.md \"Guide\")",
        "Run scripts/run.py first.",
        "Run ./scripts/run.py first.",
        "`./references/guide.md`",
        "`references/guide.md`",
        "[site](https://example.test/references/absent.md)",
        "[mail](mailto:a@example.test)",
        "[here](#usage)",
        "https://example.test/scripts/absent.py",
        "scripts/ and assets/ hold nothing else.",
    ] {
        assert!(
            skill_with(root.path(), description, "", &format!("{body}\n")).is_empty(),
            "{body}"
        );
    }
    for (body, target) in [
        ("[guide](references/absent.md)", "references/absent.md"),
        ("See\nscripts/absent.py", "scripts/absent.py"),
        ("Run ./scripts/absent.py first.", "./scripts/absent.py"),
        ("`./references/absent.md`", "./references/absent.md"),
        ("Run ../scripts/run.py first.", "../scripts/run.py"),
        ("[up](../outside.md)", "../outside.md"),
        ("[root](/etc/hosts)", "/etc/hosts"),
        ("[directory](references)", "references"),
    ] {
        assert_eq!(
            skill_with(root.path(), description, "", &format!("{body}\n")),
            [format!(
                "{SKILL_FILE}: referenced file '{target}' is not in the skill directory"
            )],
            "{body}"
        );
    }
}

#[test]
fn generated_copies_must_match_in_both_modes() {
    let root = tempfile::tempdir().unwrap();
    valid_skill(root.path(), NAME);
    assert!(s::validate(root.path(), false).unwrap().is_empty());
    let copy = format!("{}/{NAME}/SKILL.md", s::RUNTIME);
    write(root.path(), &copy, b"different\n");
    let differs = [format!("{copy}: differs from {SKILL_FILE}")];
    assert_eq!(s::validate(root.path(), false).unwrap(), differs);
    valid_skill_set(root.path());
    write(root.path(), &copy, b"different\n");
    assert_eq!(s::validate(root.path(), true).unwrap(), differs);
}

#[test]
fn ordinary_files_are_not_skills() {
    let root = tempfile::tempdir().unwrap();
    valid_skill_set(root.path());
    write(root.path(), ".claude/skills/README.md", b"Skill notes.\n");
    write(root.path(), ".agents/skills/README.md", b"Skill notes.\n");
    assert_eq!(
        incremental_and_strict(root.path()),
        [Vec::<String>::new(), vec![]]
    );
    for tree in [s::CANONICAL, s::RUNTIME] {
        let path = root.path().join(tree).join(NAME);
        fs::remove_dir_all(&path).unwrap();
        fs::write(&path, b"Skill notes.\n").unwrap();
    }
    assert_eq!(
        incremental_and_strict(root.path()),
        [
            vec![],
            vec![format!("{}: missing required skill '{NAME}'", s::CANONICAL)]
        ]
    );
}

#[test]
fn strict_mode_requires_the_complete_generated_tree() {
    let root = tempfile::tempdir().unwrap();
    valid_skill(root.path(), NAME);
    assert_eq!(
        s::validate(root.path(), true).unwrap(),
        [
            ".claude/skills: missing required skill 'github-issue-to-pr'",
            ".claude/skills: missing required skill 'github-profile-curator'",
            ".claude/skills: missing required skill 'github-readme-polish'",
            ".claude/skills: missing required skill 'github-release-prep'",
        ]
    );
    valid_skill_set(root.path());
    write(
        root.path(),
        ".claude/skills/github-repository-audit/references/checklist.md",
        b"canonical\n",
    );
    assert_eq!(
        s::validate(root.path(), true).unwrap(),
        [
            ".agents/skills/github-repository-audit/references: missing generated directory",
            ".agents/skills/github-repository-audit/references/checklist.md: missing generated copy",
        ]
    );
    fs::remove_dir_all(root.path().join(".agents")).unwrap();
    assert!(
        s::validate(root.path(), true)
            .unwrap()
            .iter()
            .any(|failure| failure.contains("missing generated"))
    );
    let root = tempfile::tempdir().unwrap();
    valid_skill_set(root.path());
    write(
        root.path(),
        ".agents/skills/github-repository-audit/extra.md",
        b"extra\n",
    );
    write(
        root.path(),
        ".agents/skills/not-approved/SKILL.md",
        b"extra\n",
    );
    assert_eq!(
        s::validate(root.path(), true).unwrap(),
        [
            ".agents/skills/github-repository-audit/extra.md: unexpected generated file",
            ".agents/skills/not-approved/SKILL.md: unexpected generated file",
            ".agents/skills/not-approved: unexpected generated directory",
        ]
    );
}

#[test]
fn recorded_runs_go_stale_on_any_byte_change() {
    let root = tempfile::tempdir().unwrap();
    let skill = root.path().join(s::CANONICAL).join(NAME);
    write(&skill, "SKILL.md", b"---\nname: x\n---\n");
    write(&skill, "references/checklist.md", b"check\n");
    let run = format!("validation/forward/{NAME}.txt");
    // A run without a digest is a notice, and needs no assessor.
    write(
        root.path(),
        &run,
        b"Scenario: x\n\nAudit metadata:\n- Run exit status: 0.\n",
    );
    assert!(s::check_runs(root.path()).unwrap().0.is_empty());
    let digest = s::skill_digest(&skill).unwrap();
    write(
        root.path(),
        &run,
        format!(
            "Audit metadata:\n- Skill digest v1:\n  `{digest}`.\n- Assessed by: Ryan Duguid.\n"
        )
        .as_bytes(),
    );
    assert_eq!(s::check_runs(root.path()).unwrap(), (vec![], vec![]));
    for (file, bytes) in [
        ("SKILL.md", b"---\nname: y\n---\n".as_slice()),
        ("references/checklist.md", b"changed\n"),
        ("references/checklist.md", b"check\r\n"),
        ("scripts/new.py", b"changed\n"),
    ] {
        let before = fs::read(skill.join(file)).ok();
        write(&skill, file, bytes);
        let current = s::skill_digest(&skill).unwrap();
        assert_eq!(
            s::check_runs(root.path()).unwrap(),
            (
                vec![format!(
                    "{run}: stale; recorded skill digest {digest} differs from {}/{NAME} digest {current}",
                    s::CANONICAL
                )],
                vec![]
            ),
            "{file} {bytes:?}"
        );
        match before {
            Some(before) => write(&skill, file, &before),
            None => fs::remove_file(skill.join(file)).unwrap(),
        }
    }
}

#[test]
fn a_missing_frontmatter_file_preserves_the_io_error() {
    let root = tempfile::tempdir().unwrap();
    let error = s::frontmatter(&root.path().join("SKILL.md")).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
}

#[cfg(unix)]
#[test]
fn a_file_in_place_of_a_generated_skill_directory_is_an_io_error() {
    let root = tempfile::tempdir().unwrap();
    valid_skill(root.path(), NAME);
    write(root.path(), &format!("{}/{NAME}", s::RUNTIME), b"fixture");
    let error = s::validate(root.path(), false).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::NotADirectory);
}

#[cfg(target_os = "linux")]
#[test]
fn a_skill_digest_refuses_a_non_unicode_file_name() {
    use std::os::unix::ffi::OsStrExt;

    let root = tempfile::tempdir().unwrap();
    let name = std::ffi::OsStr::from_bytes(b"checklist-\xff.md");
    fs::write(root.path().join(name), b"fixture").unwrap();
    let error = s::skill_digest(root.path()).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "skill file name is not valid Unicode");
}

#[test]
fn cli_sync_runs_before_validation_and_a_refusal_stops_it() {
    let root = tempfile::tempdir().unwrap();
    let text = skill_text(NAME, "assessing a repository");
    write(root.path(), SKILL_FILE, text.as_bytes());
    let output = cli(root.path(), &["validate-skills", "--sync"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        fs::read(root.path().join(s::RUNTIME).join(NAME).join("SKILL.md")).unwrap(),
        text.as_bytes()
    );
    fs::remove_dir_all(root.path().join(s::CANONICAL)).unwrap();
    // Strict validation would add a line for each missing skill.
    let output = cli(root.path(), &["validate-skills", "--sync", "--strict"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap(),
        ".claude/skills: is missing\n"
    );
}

#[test]
fn a_linked_canonical_tree_is_refused_before_it_is_read() {
    let root = tempfile::tempdir().unwrap();
    let outside = root.path().join("outside");
    fs::create_dir_all(outside.join(NAME)).unwrap();
    link_directory(&outside, &root.path().join(s::CANONICAL));
    let failure = format!("{}: {LINK}", s::CANONICAL);
    assert_eq!(
        incremental_and_strict(root.path()),
        [vec![failure.clone()], vec![failure]]
    );
}

#[test]
fn a_linked_canonical_skill_is_refused_in_both_modes() {
    let failure = format!("{}/{NAME}: {LINK}", s::CANONICAL);
    // A dangling link, then a link to a matching copy outside the repository.
    for matching in [false, true] {
        let root = tempfile::tempdir().unwrap();
        valid_skill_set(root.path());
        let skill = root.path().join(s::CANONICAL).join(NAME);
        let outside = root.path().join("outside");
        if matching {
            fs::rename(&skill, &outside).unwrap();
        } else {
            fs::remove_dir_all(&skill).unwrap();
        }
        link_directory(&outside, &skill);
        assert_eq!(
            incremental_and_strict(root.path()),
            [vec![failure.clone()], vec![failure.clone()]],
            "matching {matching}"
        );
    }
}

#[test]
fn linked_skill_files_are_refused() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("outside.md");
    fs::write(&target, skill_text(NAME, "x")).unwrap();
    if !link_file(&target, &root.path().join(SKILL_FILE)) {
        eprintln!("skipped: file links need a privilege this account lacks");
        return;
    }
    assert_eq!(
        s::validate(root.path(), false).unwrap(),
        [format!("{SKILL_FILE}: {LINK}")]
    );
    let root = tempfile::tempdir().unwrap();
    valid_skill_set(root.path());
    let skill = root.path().join(s::CANONICAL).join(NAME);
    fs::remove_dir_all(&skill).unwrap();
    fs::write(root.path().join("target.txt"), b"fabricated fixture\n").unwrap();
    assert!(link_file(&root.path().join("target.txt"), &skill));
    let failure = format!("{}/{NAME}: {LINK}", s::CANONICAL);
    assert_eq!(
        incremental_and_strict(root.path()),
        [vec![failure.clone()], vec![failure]]
    );
}

#[test]
fn incremental_mode_refuses_a_linked_generated_skill() {
    let root = tempfile::tempdir().unwrap();
    valid_skill(root.path(), NAME);
    let copy = root.path().join(s::RUNTIME).join(NAME);
    let outside = root.path().join("outside-generated");
    fs::rename(&copy, &outside).unwrap();
    link_directory(&outside, &copy);
    assert!(
        s::validate(root.path(), false)
            .unwrap()
            .contains(&format!("{}/{NAME}: {LINK}", s::RUNTIME))
    );
}

#[test]
fn strict_mode_reports_links_without_following_them() {
    // A generated skill linked back at its own tree would never end if followed.
    let root = tempfile::tempdir().unwrap();
    valid_skill_set(root.path());
    let runtime = root.path().join(s::RUNTIME);
    fs::remove_dir_all(runtime.join(NAME)).unwrap();
    link_directory(&runtime, &runtime.join(NAME));
    assert!(
        s::validate(root.path(), true)
            .unwrap()
            .contains(&format!("{}/{NAME}: {LINK}", s::RUNTIME))
    );

    // Both trees are reported before comparison stops.
    let root = tempfile::tempdir().unwrap();
    valid_skill_set(root.path());
    write(
        root.path(),
        &format!("{}/{NAME}/references/checklist.md", s::CANONICAL),
        b"canonical\n",
    );
    let links = [
        format!("{}/{NAME}/references", s::CANONICAL),
        format!("{}/{NAME}", s::RUNTIME),
    ];
    for (index, link) in links.iter().enumerate() {
        let link = root.path().join(link);
        let outside = root.path().join(format!("outside-{index}"));
        fs::rename(&link, &outside).unwrap();
        link_directory(&outside, &link);
    }
    assert_eq!(
        s::validate(root.path(), true).unwrap(),
        links.map(|link| format!("{link}: {LINK}"))
    );

    // A linked canonical skill is reported once, not again by the tree walk.
    let root = tempfile::tempdir().unwrap();
    valid_skill_set(root.path());
    let skill = root.path().join(s::CANONICAL).join(NAME);
    let outside = root.path().join("outside-canonical");
    fs::rename(&skill, &outside).unwrap();
    link_directory(&outside, &skill);
    assert_eq!(
        s::validate(root.path(), true).unwrap(),
        [format!("{}/{NAME}: {LINK}", s::CANONICAL)]
    );
}

/// A repository whose one canonical skill has been synchronised, and a
/// directory outside it.
fn synced() -> (tempfile::TempDir, tempfile::TempDir) {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), SKILL_FILE, b"canonical\n");
    assert!(s::sync(root.path()).unwrap().is_empty());
    (root, tempfile::tempdir().unwrap())
}

fn sentinel(directory: &Path) -> PathBuf {
    let path = directory.join("sentinel.txt");
    write(directory, "sentinel.txt", b"outside\n");
    path
}

fn read(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap()
}

#[test]
fn sync_removes_unexpected_generated_content() {
    let (root, _outside) = synced();
    let runtime = root.path().join(s::RUNTIME);
    write(&runtime, "retired-skill/SKILL.md", b"retired\n");
    write(&runtime, "obsolete.txt", b"obsolete\n");
    assert!(s::sync(root.path()).unwrap().is_empty());
    let names: Vec<_> = fs::read_dir(&runtime)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(names, [NAME]);
}

#[test]
fn sync_refuses_a_runtime_root_that_is_a_file() {
    let (root, _outside) = synced();
    let runtime = root.path().join(s::RUNTIME);
    fs::remove_dir_all(&runtime).unwrap();
    fs::write(&runtime, b"not a directory\n").unwrap();
    assert!(
        s::sync(root.path())
            .unwrap()
            .contains(&format!("{}: is not an ordinary directory", s::RUNTIME))
    );
    assert_eq!(read(&runtime), b"not a directory\n");
}

#[test]
fn sync_refuses_a_linked_runtime_root_before_writing_outside() {
    let (root, outside) = synced();
    fs::remove_dir_all(root.path().join(".agents")).unwrap();
    write(outside.path(), &format!("{NAME}/SKILL.md"), b"outside\n");
    link_directory(outside.path(), &root.path().join(s::RUNTIME));
    assert!(
        s::sync(root.path())
            .unwrap()
            .contains(&format!("{}: {LINK}", s::RUNTIME))
    );
    assert_eq!(
        read(&outside.path().join(NAME).join("SKILL.md")),
        b"outside\n"
    );
}

#[test]
fn sync_refuses_any_linked_entry_before_any_mutation() {
    // Each link, relative to the repository, is refused with its own failure.
    for link in [
        format!("{}/{NAME}", s::CANONICAL),
        format!("{}/{NAME}/references/linked", s::RUNTIME),
        format!("{}/{NAME}/references/linked", s::CANONICAL),
    ] {
        let (root, outside) = synced();
        let target = outside.path().join("target");
        let sentinel = sentinel(&target);
        let path = root.path().join(&link);
        if path.exists() {
            fs::remove_dir_all(&path).unwrap();
        }
        link_directory(&target, &path);
        let copy = root.path().join(s::RUNTIME).join(NAME).join("SKILL.md");
        let canonical = root.path().join(SKILL_FILE);
        fs::write(&copy, b"agents-before\n").unwrap();
        let canonical_before = fs::read(&canonical).ok();
        assert!(
            s::sync(root.path())
                .unwrap()
                .contains(&format!("{link}: {LINK}")),
            "{link}"
        );
        assert_eq!(read(&sentinel), b"outside\n", "{link}");
        assert_eq!(read(&copy), b"agents-before\n", "{link}");
        assert_eq!(fs::read(&canonical).ok(), canonical_before, "{link}");
    }
}

#[test]
fn sync_refuses_linked_parents_and_a_linked_root() {
    // A generated tree whose parent is a link outside the repository.
    let (root, outside) = synced();
    let parent = outside.path().join("destination-parent");
    fs::rename(root.path().join(".agents"), &parent).unwrap();
    let sentinel = sentinel(&parent.join("skills"));
    let copy = parent.join("skills").join(NAME).join("SKILL.md");
    fs::write(&copy, b"outside-copy\n").unwrap();
    link_directory(&parent, &root.path().join(".agents"));
    assert!(
        s::sync(root.path())
            .unwrap()
            .contains(&format!(".agents: {LINK}"))
    );
    assert_eq!(read(&sentinel), b"outside\n");
    assert_eq!(read(&copy), b"outside-copy\n");

    // A canonical tree whose parent is a link.
    let (root, outside) = synced();
    let parent = outside.path().join("canonical-parent");
    fs::rename(root.path().join(".claude"), &parent).unwrap();
    link_directory(&parent, &root.path().join(".claude"));
    let copy = root.path().join(s::RUNTIME).join(NAME).join("SKILL.md");
    fs::write(&copy, b"agents-before\n").unwrap();
    assert!(
        s::sync(root.path())
            .unwrap()
            .contains(&format!(".claude: {LINK}"))
    );
    assert_eq!(
        read(&parent.join("skills").join(NAME).join("SKILL.md")),
        b"canonical\n"
    );
    assert_eq!(read(&copy), b"agents-before\n");

    // The repository root itself reached through a link.
    let (root, outside) = synced();
    let alias = outside.path().join("repository");
    link_directory(root.path(), &alias);
    let copy = root.path().join(s::RUNTIME).join(NAME).join("SKILL.md");
    fs::write(&copy, b"agents-before\n").unwrap();
    assert!(s::sync(&alias).unwrap().contains(&format!(".: {LINK}")));
    assert_eq!(read(&copy), b"agents-before\n");
}
