use fancy_regex::Regex;
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

fn ordinary_entries(
    tree: &Path,
    root: &Path,
    failures: &mut Vec<String>,
) -> io::Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    let mut pending = vec![tree.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let mut children = entries(&directory)?;
        children.sort_by_key(|path| {
            let name = crate::posix(path);
            // pathlib sorts Windows Path objects using their lowercase parts.
            if cfg!(windows) {
                name.to_lowercase()
            } else {
                name
            }
        });
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
            && matches!(key, "name" | "description")
        {
            metadata.insert(
                key.to_owned(),
                value.trim_matches(crate::python_whitespace).to_owned(),
            );
        }
    }
    Ok(Ok(metadata))
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
        if let Some(matched) = PLACEHOLDER
            .find(&crate::read_text(&path)?)
            .map_err(io::Error::other)?
        {
            failures.push(format!(
                "{file_name}: forbidden placeholder '{}'",
                matched.as_str().to_uppercase()
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
