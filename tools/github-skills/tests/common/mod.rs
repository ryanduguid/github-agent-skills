#![allow(dead_code)] // Each test crate uses its own subset of these helpers.

use std::{fs, path::Path, process::Command};

pub fn write(root: &Path, name: &str, bytes: &[u8]) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

pub fn skill(name: &str) -> Vec<u8> {
    format!("---\nname: {name}\ndescription: Use when testing.\n---\n# Skill\n").into_bytes()
}

pub fn cli(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_github-skills"))
        .args(args)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

/// Point `link` at the directory `target`, which need not exist: an NTFS
/// junction on Windows, which needs no privilege, or a symbolic link elsewhere.
pub fn link_directory(target: &Path, link: &Path) {
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    #[cfg(windows)]
    {
        // cmd reads a forward slash as the start of a switch.
        let native = |path: &Path| path.to_string_lossy().replace('/', "\\");
        let output = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(native(link))
            .arg(native(target))
            .output()
            .unwrap();
        assert!(output.status.success(), "mklink /J failed: {output:?}");
    }
    #[cfg(not(windows))]
    std::os::unix::fs::symlink(target, link).unwrap();
}

/// Point `link` at the file `target`. Windows file links need a privilege that
/// ordinary accounts lack, so this returns false when one cannot be made.
pub fn link_file(target: &Path, link: &Path) -> bool {
    fs::create_dir_all(link.parent().unwrap()).unwrap();
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_file(target, link).is_ok()
    }
    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(target, link).unwrap();
        true
    }
}
