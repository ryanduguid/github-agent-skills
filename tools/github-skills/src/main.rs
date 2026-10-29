use std::{env, io, path::PathBuf, process::ExitCode};

const HELP: &str = "Usage: github-skills <validate-skills|check-public-files> [--root PATH] [--strict] [--sync]\n\nLocal Rust candidate. The root defaults to the current directory.\n--strict and --sync apply only to validate-skills.\n";

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
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("--root") => {
                root = PathBuf::from(args.next().ok_or("--root requires a path")?);
            }
            Some("--strict") if command == "validate-skills" => {
                strict = true;
            }
            Some("--sync") if command == "validate-skills" => {
                sync = true;
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
    let result: io::Result<Vec<String>> = if command == "check-public-files" {
        github_skills::public_files::check(&root)
    } else {
        (|| {
            let failures = if sync {
                github_skills::skills::sync(&root)?
            } else {
                vec![]
            };
            if failures.is_empty() {
                github_skills::skills::validate(&root, strict)
            } else {
                Ok(failures)
            }
        })()
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
