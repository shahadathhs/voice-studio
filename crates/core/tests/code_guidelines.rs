//! Enforces the file-size rule from docs/guidelines.md so it runs with plain `cargo test`.

use std::fs;
use std::path::{Path, PathBuf};

const MAX_LINES: usize = 300;

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "target") {
                rust_files(&path, out);
            }
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

#[test]
fn no_rust_file_exceeds_the_line_limit() {
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = Vec::new();
    rust_files(&crates_dir, &mut files);
    assert!(
        !files.is_empty(),
        "no source files found under {}",
        crates_dir.display()
    );

    let offenders: Vec<String> = files
        .iter()
        .filter_map(|f| {
            let lines = fs::read_to_string(f).ok()?.lines().count();
            (lines > MAX_LINES).then(|| format!("{} ({lines} lines)", f.display()))
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "files over {MAX_LINES} lines, split them by responsibility:\n  {}",
        offenders.join("\n  ")
    );
}
