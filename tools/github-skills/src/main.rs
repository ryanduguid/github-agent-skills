use github_skills::skills;
use std::{
    env, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

const HELP: &str = "Usage: github-skills <validate-skills|check-public-files> [--root PATH] [--strict] [--sync] [--skill-digest NAME]\n\nLocal Rust candidate. The root defaults to the current directory.\n--strict, --sync and --skill-digest apply only to validate-skills.\n--skill-digest prints the skill digest line to record in a new run file\nfor approved skill NAME, after strict validation.\n";

/// Mirrors `validate_skills.py`: run notices and the digest line use stdout.
fn validate_skills(
    root: &Path,
    strict: bool,
    sync: bool,
    digest: Option<&str>,
) -> io::Result<Vec<String>> {
    let mut failures = if sync { skills::sync(root)? } else { vec![] };
    if failures.is_empty() {
        failures = skills::validate(root, strict || digest.is_some())?;
    }
    if failures.is_empty() && strict {
        let notices;
        (failures, notices) = skills::check_runs(root)?;
        for notice in notices {
            println!("{notice}");
        }
    }
    if failures.is_empty()
        && let Some(name) = digest
    {
        let value = skills::skill_digest(&root.join(skills::CANONICAL).join(name))?;
        println!("- Skill digest v1:\n  `{value}`.");
    }
    Ok(failures)
}

fn run() -> Result<u8, String> {
    let mut args = env::args_os().skip(1);
    let Some(command) = args.next() else {
        return Err(HELP.into());
    };
    if command == "--help" || command == "-h" {
        print!("{HELP}");
        return Ok(0);
    }
    if command == "--version" {
        println!("github-skills {}", env!("CARGO_PKG_VERSION"));
        return Ok(0);
    }
    if command != "validate-skills" && command != "check-public-files" {
        return Err("unknown command".into());
    }
    let mut root = env::current_dir().map_err(|_| "cannot read current directory")?;
    let mut strict = false;
    let mut sync = false;
    let mut digest = None;
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--root") => {
                let path = args
                    .next()
                    .filter(|path| !path.as_encoded_bytes().starts_with(b"-"))
                    .ok_or("--root requires a path")?;
                root = PathBuf::from(path);
            }
            Some("--strict") if command == "validate-skills" => {
                strict = true;
            }
            Some("--sync") if command == "validate-skills" => {
                sync = true;
            }
            Some("--skill-digest") if command == "validate-skills" => {
                let name = args.next().ok_or("--skill-digest requires a skill name")?;
                let approved = skills::APPROVED_NAMES
                    .into_iter()
                    .find(|approved| name == *approved)
                    .ok_or("--skill-digest requires an approved skill name")?;
                digest = Some(approved);
            }
            Some("--help" | "-h") => {
                print!("{HELP}");
                return Ok(0);
            }
            _ => return Err("unknown argument".into()),
        }
    }
    // Do not canonicalise: that would hide a linked root from sync's preflight.
    if root.is_relative() {
        root = env::current_dir()
            .map_err(|_| "cannot read current directory")?
            .join(root);
    }
    let result = if command == "check-public-files" {
        github_skills::public_files::check(&root)
    } else {
        validate_skills(&root, strict, sync, digest)
    };
    // IO diagnostics deliberately avoid printing file content or an OS error's path.
    let failures =
        result.map_err(|_| "repository check could not read or process an input".to_owned())?;
    if failures.is_empty() {
        Ok(0)
    } else {
        eprintln!("{}", failures.join("\n"));
        Ok(1)
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}
