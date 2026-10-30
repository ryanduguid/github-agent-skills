use fancy_regex::Regex;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs, io,
    path::{Path, PathBuf},
    sync::LazyLock,
};

pub const APPROVED_NAMES: [&str; 5] = [
    "github-issue-to-pr",
    "github-profile-curator",
    "github-readme-polish",
    "github-release-prep",
    "github-repository-audit",
];
pub const CANONICAL: &str = ".claude/skills";
pub const RUNTIME: &str = ".agents/skills";
pub const RUN_DIRECTORIES: [&str; 2] = ["validation/baselines", "validation/forward"];
// A recorded run names its skill by file stem and records the skill digest as
// "- Skill digest v1:" followed by the backquoted digest, on the same line or
// the next.
static RUN_DIGEST: LazyLock<Regex> = LazyLock::new(|| {
    crate::public_files::python_regex(r"^- Skill digest v1:\s+`([0-9a-f]{64})`\.$", false, true)
});
static RUN_DIGEST_LABEL: LazyLock<Regex> =
    LazyLock::new(|| crate::public_files::python_regex(r"^- Skill digest\b", false, true));
// Limits from https://agentskills.io/specification, as scripts/validate_skills.py
// applies them.
const MAX_DESCRIPTION: usize = 1024;
const MAX_COMPATIBILITY: usize = 500;
const UNSAFE_PLAIN_START: &str = "!\"#%&'*>?@[]`{|},";
static REFERENCE: LazyLock<Regex> = LazyLock::new(|| {
    crate::public_files::python_regex(
        r"\]\(([^)\s]+)|(?:^|[^A-Za-z0-9_./-])((?:scripts|references|assets)/[A-Za-z0-9_./-]*[A-Za-z0-9_])",
        false,
        true,
    )
});
static EXTERNAL_REFERENCE: LazyLock<Regex> = LazyLock::new(|| {
    crate::public_files::python_regex(r"\A(?:[A-Za-z][A-Za-z0-9+.-]*:|#)", false, false)
});
static PLACEHOLDER: LazyLock<Regex> = LazyLock::new(|| {
    crate::public_files::python_regex(
        r"\b(TBD|TODO|FIXME|placeholder|scaffold(?:ing)?)\b",
        true,
        false,
    )
});

pub fn is_link(path: &Path) -> io::Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                Ok(metadata.file_attributes() & 0x400 != 0)
            }
            #[cfg(not(windows))]
            {
                Ok(metadata.file_type().is_symlink())
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

fn relative(path: &Path, root: &Path) -> String {
    let name = path
        .strip_prefix(root)
        .expect("all walked paths are beneath the supplied root");
    if name.as_os_str().is_empty() {
        ".".into()
    } else {
        crate::posix(name)
    }
}

fn link_failure(path: &Path, root: &Path) -> String {
    format!(
        "{}: is a link, not an ordinary file or directory",
        relative(path, root)
    )
}

fn entries(directory: &Path) -> io::Result<Vec<PathBuf>> {
    fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect()
}

// Strict Python comparisons sort POSIX display strings, while path identity
// must retain raw filesystem bytes. Never use a lossy display name as a key.
fn display_order<'a>(paths: impl Iterator<Item = &'a PathBuf>) -> Vec<&'a PathBuf> {
    let mut paths: Vec<_> = paths.collect();
    paths.sort_by_key(|path| crate::posix(path));
    paths
}

fn path_order(path: &Path) -> String {
    let name = crate::posix(path);
    // pathlib sorts Windows Path objects using their lowercase parts.
    if cfg!(windows) {
        name.to_lowercase()
    } else {
        name
    }
}

fn ordinary_entries(
    tree: &Path,
    root: &Path,
    failures: &mut Vec<String>,
) -> io::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut pending = vec![tree.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let mut children = entries(&directory)?;
        children.sort_by_key(|path| path_order(path));
        for path in children {
            if is_link(&path)? {
                failures.push(link_failure(&path, root));
                continue;
            }
            if path.is_dir() {
                pending.push(path.clone());
            }
            found.push(path);
        }
    }
    Ok(found)
}

fn linked_ancestor(tree: &Path, root: &Path) -> io::Result<Option<PathBuf>> {
    for path in tree.ancestors() {
        if is_link(path)? {
            return Ok(Some(path.to_path_buf()));
        }
        if path == root {
            break;
        }
    }
    Ok(None)
}

pub fn frontmatter(path: &Path) -> io::Result<Result<BTreeMap<String, String>, String>> {
    let text = match crate::read_text(path) {
        Ok(text) => text,
        Err(e) if e.kind() == io::ErrorKind::InvalidData => {
            return Ok(Err("not valid UTF-8 text".into()));
        }
        Err(e) => return Err(e),
    };
    let lines = crate::splitlines(&text);
    if lines.first() != Some(&"---") {
        return Ok(Err("missing YAML frontmatter delimiters".into()));
    }
    let Some(end) = lines
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(i, line)| (*line == "---").then_some(i))
    else {
        return Ok(Err("missing YAML frontmatter delimiters".into()));
    };
    let mut metadata = BTreeMap::new();
    for line in &lines[1..end] {
        if let Some((key, value)) = line.split_once(':')
            && matches!(key, "name" | "description" | "compatibility")
        {
            metadata.insert(
                key.to_owned(),
                value.trim_matches(crate::python_whitespace).to_owned(),
            );
        }
    }
    Ok(Ok(metadata))
}

fn unsafe_plain_scalar(value: &str) -> bool {
    value.starts_with(|c| UNSAFE_PLAIN_START.contains(c))
        || value.contains(": ")
        || value.contains(" #")
        || value.ends_with(':')
}

/// The first way the frontmatter values break the Agent Skills specification.
fn specification_failure(description: &str, compatibility: Option<&str>) -> Option<String> {
    if description.chars().count() > MAX_DESCRIPTION {
        return Some(format!("description exceeds {MAX_DESCRIPTION} characters"));
    }
    if let Some(value) = compatibility
        && !(1..=MAX_COMPATIBILITY).contains(&value.chars().count())
    {
        return Some(format!(
            "compatibility must be 1 to {MAX_COMPATIBILITY} characters"
        ));
    }
    for (key, value) in [
        ("description", description),
        ("compatibility", compatibility.unwrap_or("")),
    ] {
        if unsafe_plain_scalar(value) {
            return Some(format!(
                "{key} must be a plain YAML value: no ': ', ' #', trailing ':' or leading indicator"
            ));
        }
    }
    None
}

/// The first relative reference in text that is not a file in the skill directory.
fn missing_reference(directory: &Path, text: &str) -> io::Result<Option<String>> {
    for captures in REFERENCE.captures_iter(text) {
        let captures = captures.map_err(io::Error::other)?;
        let target = captures
            .get(1)
            .or_else(|| captures.get(2))
            .map_or("", |m| m.as_str());
        if EXTERNAL_REFERENCE
            .is_match(target)
            .map_err(io::Error::other)?
        {
            continue;
        }
        let relative = target.split('#').next().unwrap_or("");
        if relative.starts_with('/')
            || relative.split('/').any(|part| part == "..")
            || !directory.join(relative).is_file()
        {
            return Ok(Some(target.to_owned()));
        }
    }
    Ok(None)
}

pub fn validate(root: &Path, strict: bool) -> io::Result<Vec<String>> {
    let canonical = root.join(CANONICAL);
    let destination = root.join(RUNTIME);
    let mut failures = Vec::new();
    for tree in [&canonical, &destination] {
        if let Some(path) = linked_ancestor(tree, root)? {
            failures.push(link_failure(&path, root));
        }
    }
    if !failures.is_empty() {
        return Ok(failures);
    }
    let mut skills = Vec::new();
    if canonical.is_dir() {
        let mut children = entries(&canonical)?;
        // Skill discovery explicitly sorts by name, even on Windows.
        children.sort_by_key(|path| path.file_name().unwrap().to_os_string());
        for path in children {
            if is_link(&path)? || path.is_dir() {
                skills.push(path);
            }
        }
    }
    let mut valid = Vec::new();
    for directory in &skills {
        let name = directory.file_name().unwrap().to_string_lossy();
        let directory_name = relative(directory, root);
        if !APPROVED_NAMES.contains(&name.as_ref()) {
            failures.push(format!("{directory_name}: unexpected skill name"));
            continue;
        }
        if is_link(directory)? {
            failures.push(link_failure(directory, root));
            continue;
        }
        let path = directory.join("SKILL.md");
        let file_name = relative(&path, root);
        if is_link(&path)? {
            failures.push(link_failure(&path, root));
            continue;
        }
        if !path.is_file() {
            failures.push(format!("{directory_name}: missing SKILL.md"));
            continue;
        }
        let metadata = match frontmatter(&path)? {
            Ok(metadata) => metadata,
            Err(error) => {
                failures.push(format!("{file_name}: {error}"));
                continue;
            }
        };
        let name_value = metadata.get("name").map(String::as_str).unwrap_or("");
        let description = metadata
            .get("description")
            .map(String::as_str)
            .unwrap_or("");
        let error = if name_value.is_empty() {
            Some("missing name")
        } else if description.is_empty() {
            Some("missing description")
        } else if name_value != name {
            Some("name must match directory name")
        } else if !description.starts_with("Use when") {
            Some("description must start with 'Use when'")
        } else {
            None
        };
        if let Some(error) = error {
            failures.push(format!("{file_name}: {error}"));
            continue;
        }
        let compatibility = metadata.get("compatibility").map(String::as_str);
        if let Some(error) = specification_failure(description, compatibility) {
            failures.push(format!("{file_name}: {error}"));
            continue;
        }
        let text = crate::read_text(&path)?;
        if let Some(matched) = PLACEHOLDER.find(&text).map_err(io::Error::other)? {
            failures.push(format!(
                "{file_name}: forbidden placeholder '{}'",
                matched.as_str().to_uppercase()
            ));
            continue;
        }
        if let Some(missing) = missing_reference(directory, &text)? {
            failures.push(format!(
                "{file_name}: referenced file '{missing}' is not in the skill directory"
            ));
            continue;
        }
        valid.push((name.into_owned(), path));
    }
    if strict {
        let present: BTreeSet<_> = skills
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy())
            .collect();
        for name in APPROVED_NAMES {
            if !present.contains(name) {
                failures.push(format!("{CANONICAL}: missing required skill '{name}'"));
            }
        }
    }
    if !strict {
        for (name, source) in valid {
            let copy = destination.join(name).join("SKILL.md");
            let parent = copy.parent().unwrap();
            if let Some(link) = [parent, &copy]
                .into_iter()
                .find_map(|p| match is_link(p) {
                    Ok(true) => Some(Ok(p)),
                    Ok(false) => None,
                    Err(e) => Some(Err(e)),
                })
                .transpose()?
            {
                failures.push(link_failure(link, root));
            } else if copy.is_file() && fs::read(&source)? != fs::read(&copy)? {
                failures.push(format!(
                    "{}: differs from {}",
                    relative(&copy, root),
                    relative(&source, root)
                ));
            }
        }
    } else {
        let before = failures.len();
        let expected = if canonical.is_dir() {
            ordinary_entries(&canonical, root, &mut failures)?
        } else {
            vec![]
        };
        let actual = if destination.is_dir() {
            ordinary_entries(&destination, root, &mut failures)?
        } else {
            vec![]
        };
        if failures.len() != before {
            let mut seen = BTreeSet::new();
            failures.retain(|failure| seen.insert(failure.clone()));
            return Ok(failures);
        }
        let sets = |paths: &[PathBuf], base: &Path, directories: bool| -> BTreeSet<PathBuf> {
            paths
                .iter()
                .filter(|p| if directories { p.is_dir() } else { p.is_file() })
                .map(|p| p.strip_prefix(base).unwrap().to_path_buf())
                .collect()
        };
        let expected_directories = sets(&expected, &canonical, true);
        let expected_files = sets(&expected, &canonical, false);
        let actual_directories = sets(&actual, &destination, true);
        let actual_files = sets(&actual, &destination, false);
        for name in display_order(expected_directories.difference(&actual_directories)) {
            failures.push(format!(
                "{RUNTIME}/{}: missing generated directory",
                crate::posix(name)
            ));
        }
        for name in display_order(expected_files.iter()) {
            let source = canonical.join(name);
            let copy = destination.join(name);
            let name = crate::posix(name);
            if !copy.is_file() {
                failures.push(format!("{RUNTIME}/{name}: missing generated copy"));
            } else if fs::read(source)? != fs::read(copy)? {
                failures.push(format!("{RUNTIME}/{name}: differs from {CANONICAL}/{name}"));
            }
        }
        for name in display_order(actual_files.difference(&expected_files)) {
            failures.push(format!(
                "{RUNTIME}/{}: unexpected generated file",
                crate::posix(name)
            ));
        }
        for name in display_order(actual_directories.difference(&expected_directories)) {
            failures.push(format!(
                "{RUNTIME}/{}: unexpected generated directory",
                crate::posix(name)
            ));
        }
    }
    Ok(failures)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Skill digest v1 of a canonical skill directory, as recorded in run files.
///
/// The manifest has one line per regular file under the directory, at any
/// depth: the lowercase hex SHA-256 of the file's bytes, 2 spaces, the path
/// relative to the directory with '/' separators, then a line feed. Lines are
/// sorted by that path in UTF-8 byte order. The digest is the lowercase hex
/// SHA-256 of the manifest. Bytes are hashed as stored. Callers validate the
/// tree first, because links are skipped here.
pub fn skill_digest(directory: &Path) -> io::Result<String> {
    let mut files = Vec::new();
    for path in ordinary_entries(directory, directory, &mut Vec::new())? {
        if !path.is_file() {
            continue;
        }
        let name = path
            .strip_prefix(directory)
            .expect("all walked paths are beneath the skill directory");
        // Python cannot encode a non-Unicode name into the UTF-8 manifest.
        if name.to_str().is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "skill file name is not valid Unicode",
            ));
        }
        files.push((crate::posix(name), path));
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    let mut manifest = String::new();
    for (name, path) in files {
        manifest.push_str(&format!(
            "{}  {name}\n",
            hex(&Sha256::digest(fs::read(path)?))
        ));
    }
    Ok(hex(&Sha256::digest(manifest)))
}

/// `Path.glob("*.txt")` within one directory, in `sorted()` order.
fn recorded_runs(directory: &Path) -> Vec<PathBuf> {
    // pathlib's glob yields nothing for a directory it cannot list.
    let Ok(children) = entries(directory) else {
        return vec![];
    };
    let mut runs: Vec<_> = children
        .into_iter()
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy();
            // pathlib matches case-insensitively on Windows.
            if cfg!(windows) {
                name.to_ascii_lowercase().ends_with(".txt")
            } else {
                name.ends_with(".txt")
            }
        })
        .collect();
    runs.sort_by_key(|path| path_order(path));
    runs
}

fn collect_matches(
    matches: impl Iterator<Item = fancy_regex::Result<String>>,
) -> io::Result<Vec<String>> {
    matches.collect::<Result<_, _>>().map_err(io::Error::other)
}

/// Failures for stale or malformed recorded runs, and notices for runs without
/// a skill digest.
pub fn check_runs(root: &Path) -> io::Result<(Vec<String>, Vec<String>)> {
    let mut failures = Vec::new();
    let mut notices = Vec::new();
    let mut current = BTreeMap::new();
    for directory in RUN_DIRECTORIES {
        for run in recorded_runs(&root.join(directory)) {
            let name = relative(&run, root);
            let stem = run.file_stem().unwrap().to_string_lossy().into_owned();
            if !APPROVED_NAMES.contains(&stem.as_str()) {
                failures.push(format!("{name}: names no approved skill"));
                continue;
            }
            let text = crate::read_text(&run)?;
            let labels = collect_matches(
                RUN_DIGEST_LABEL
                    .find_iter(text.as_str())
                    .map(|found| found.map(|found| found.as_str().to_owned())),
            )?;
            if labels.is_empty() {
                notices.push(format!(
                    "{name}: no skill digest recorded; its results are not tied to a skill revision"
                ));
                continue;
            }
            let recorded = collect_matches(
                RUN_DIGEST
                    .captures_iter(text.as_str())
                    .map(|found| found.map(|found| found[1].to_owned())),
            )?;
            let ([_], [recorded]) = (labels.as_slice(), recorded.as_slice()) else {
                failures.push(format!(
                    "{name}: malformed skill digest; record exactly one '- Skill digest v1:' line"
                ));
                continue;
            };
            if !current.contains_key(&stem) {
                let digest = skill_digest(&root.join(CANONICAL).join(&stem))?;
                current.insert(stem.clone(), digest);
            }
            let digest = &current[&stem];
            if recorded != digest {
                failures.push(format!(
                    "{name}: stale; recorded skill digest {recorded} differs from \
                     {CANONICAL}/{stem} digest {digest}"
                ));
            }
        }
    }
    Ok((failures, notices))
}

pub fn sync(root: &Path) -> io::Result<Vec<String>> {
    let canonical = root.join(CANONICAL);
    let destination = root.join(RUNTIME);
    let mut failures = Vec::new();
    let mut trees = Vec::new();
    for tree in [&canonical, &destination] {
        if let Some(path) = linked_ancestor(tree, root)? {
            failures.push(link_failure(&path, root));
        } else if tree == &canonical && !tree.exists() {
            failures.push(format!("{CANONICAL}: is missing"));
        } else if tree.exists() && !tree.is_dir() {
            failures.push(format!(
                "{}: is not an ordinary directory",
                relative(tree, root)
            ));
        } else {
            trees.push(if tree.is_dir() {
                ordinary_entries(tree, root, &mut failures)?
            } else {
                vec![]
            });
        }
    }
    if !failures.is_empty() {
        return Ok(failures);
    }
    let approved: Vec<_> = trees[0]
        .iter()
        .filter(|path| {
            let relative = path.strip_prefix(&canonical).unwrap();
            APPROVED_NAMES.iter().any(|name| relative.starts_with(name))
        })
        .collect();
    // shutil.copyfile rejects a FIFO instead of blocking on its open. Refuse
    // every special approved source entry before removing generated content.
    for source in &approved {
        if !source.is_dir() && !source.is_file() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "canonical skill contains a special filesystem entry",
            ));
        }
    }
    let mut obsolete: Vec<_> = trees[1].iter().collect();
    obsolete.sort_by_key(|path| std::cmp::Reverse(path.components().count()));
    // Every entry and both roots have been checked before touching the generated tree.
    // Like the original local authoring tool, this assumes no concurrent filesystem mutation.
    for path in obsolete {
        if path.is_dir() {
            fs::remove_dir(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    fs::create_dir_all(&destination)?;
    for source in approved {
        let copy = destination.join(source.strip_prefix(&canonical).unwrap());
        if source.is_dir() {
            fs::create_dir(&copy)?;
        } else {
            // shutil.copyfile copies bytes, without copying the source mode.
            io::copy(&mut fs::File::open(source)?, &mut fs::File::create(copy)?)?;
        }
    }
    Ok(vec![])
}
