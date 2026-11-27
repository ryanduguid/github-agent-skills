// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: Apache-2.0

//! What each classification matches, as `pr_precheck.py` lists it.

use crate::public_files::python_regex;
use fancy_regex::Regex;
use std::sync::LazyLock;

pub(super) const STATUSES: [&str; 7] = [
    "added",
    "removed",
    "modified",
    "renamed",
    "copied",
    "changed",
    "unchanged",
];

pub(super) const LOCKFILES: [&str; 20] = [
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

pub(super) const MANIFESTS: [&str; 16] = [
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

pub(super) const VENDORED_DIRS: [&str; 4] =
    ["vendor", "third_party", "third-party", "node_modules"];
pub(super) const GENERATED_DIRS: [&str; 2] = ["generated", "__generated__"];
pub(super) const INSTRUCTION_NAMES: [&str; 8] = [
    "AGENTS.md",
    "AGENTS.override.md",
    "CLAUDE.md",
    "GEMINI.md",
    "copilot-instructions.md",
    "SKILL.md",
    ".mcp.json",
    "mcp.json",
];
pub(super) const INSTRUCTION_DIRS: [&str; 5] =
    [".agents", ".claude", ".codex", ".gemini", "skills"];
pub(super) const PERMISSION_FILES: [&str; 2] = ["CODEOWNERS", ".github/settings.yml"];
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

pub(super) static MANIFEST_NAME: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"^requirements[\w.-]*\.(?:txt|in)$|\.(?:gemspec|csproj|fsproj)$",
        false,
        false,
    )
});

pub(super) static GENERATED_NAME: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"\.min\.(?:js|css)$|\.map$|_pb2(?:_grpc)?\.pyi?$|\.pb\.go$|\.g\.dart$|\.generated\.\w+$",
        false,
        false,
    )
});

pub(super) static GENERATED_MARKER: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"@generated|DO NOT EDIT|Code generated .* DO NOT EDIT",
        true,
        false,
    )
});

pub(super) static WORKFLOW_MARKERS: LazyLock<Vec<(&'static str, Regex)>> = LazyLock::new(|| {
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

pub(super) static HUNK: LazyLock<Regex> = LazyLock::new(|| {
    python_regex(
        r"^@@ -([0-9]+)(?:,[0-9]+)? \+([0-9]+)(?:,[0-9]+)? @@",
        false,
        false,
    )
});
