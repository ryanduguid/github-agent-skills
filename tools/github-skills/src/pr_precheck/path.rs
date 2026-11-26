// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: Apache-2.0

//! The parts of `PurePosixPath` the classifications use.

pub(super) struct PosixPath<'a> {
    root: &'static str,
    parts: Vec<&'a str>,
}

impl<'a> PosixPath<'a> {
    pub(super) fn parse(path: &'a str) -> Self {
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

    pub(super) fn name(&self) -> &'a str {
        if !self.root.is_empty() && self.parts.len() == 1 {
            ""
        } else {
            self.parts.last().copied().unwrap_or("")
        }
    }

    pub(super) fn dir_parts(&self) -> &[&'a str] {
        self.parts
            .len()
            .checked_sub(1)
            .map_or(&[], |n| &self.parts[..n])
    }

    pub(super) fn last_part(&self) -> Option<&'a str> {
        self.parts.last().copied()
    }

    pub(super) fn as_posix(&self) -> String {
        if !self.root.is_empty() {
            format!("{}{}", self.root, self.parts[1..].join("/"))
        } else if self.parts.is_empty() {
            ".".to_owned()
        } else {
            self.parts.join("/")
        }
    }

    pub(super) fn parent_as_posix(&self) -> String {
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

    pub(super) fn has_yaml_suffix(&self) -> bool {
        let name = self.name();
        let stem = name
            .strip_suffix(".yml")
            .or_else(|| name.strip_suffix(".yaml"));
        stem.is_some_and(|prefix| prefix.bytes().any(|b| b != b'.'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
