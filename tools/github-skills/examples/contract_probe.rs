//! Test-only JSON bridge for replaying the unchanged original Python assertions.
use github_skills::{public_files as p, skills as s};
use serde_json::{Value, json};
use std::{
    io::{self, BufRead, Write},
    path::{Path, PathBuf},
};

fn probe(request: &Value) -> io::Result<Value> {
    let text = |key| request[key].as_str().unwrap_or("");
    let bytes = || {
        request["bytes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|n| n.as_u64().unwrap() as u8)
            .collect::<Vec<_>>()
    };
    let root = Path::new(text("root"));
    Ok(match text("operation") {
        "validate" => json!(s::validate(
            root,
            request["strict"].as_bool().unwrap_or(false)
        )?),
        "sync" => json!(s::sync(root)?),
        "frontmatter" => match s::frontmatter(Path::new(text("path")))? {
            Ok(metadata) => json!([metadata, Value::Null]),
            Err(error) => json!([Value::Null, error]),
        },
        "scan_paths" => json!(p::scan_paths(
            root,
            &request["paths"]
                .as_array()
                .unwrap()
                .iter()
                .map(|path| PathBuf::from(path.as_str().unwrap()))
                .collect::<Vec<_>>()
        )?),
        "tracked_failures" => json!(p::tracked_failures(root)?),
        "quick_start_commands" => json!(p::quick_start_commands(text("readme"))?),
        "quick_start_failures" => json!(p::quick_start_failures(text("readme"), text("workflow"))?),
        "supported_quick_start" => json!(p::supported_quick_start(text("workflow"))?),
        "permissions_block" => json!(p::permissions_block(text("workflow"))?),
        "decoded_text" => json!(p::decoded_text(&bytes())),
        "looks_like_text" => json!(p::looks_like_text(&bytes())),
        _ => return Err(io::Error::other("unknown probe operation")),
    })
}

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout().lock();
    for line in stdin.lock().lines() {
        let request: Value = serde_json::from_str(&line?).map_err(io::Error::other)?;
        let result = match probe(&request) {
            Ok(value) => json!({"result":value}),
            Err(error) => json!({"error":format!("{:?}: {error}",error.kind())}),
        };
        writeln!(stdout, "{result}")?;
        stdout.flush()?;
    }
    Ok(())
}
