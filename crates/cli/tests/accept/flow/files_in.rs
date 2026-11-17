//! Lists the files a folder holds. Real file I/O: check 1 (flow_requests.rs) lists every request folder through it
//! on every drive.

use std::fs;
use std::io;
use std::path::Path;

/// Every file under `folder`, relative and `/`-separated, sorted; the worktree's own `.git` file at the top is left
/// out.
pub fn files_in(folder: &Path) -> io::Result<Vec<String>> {
    let mut found = Vec::new();
    walk(folder, "", &mut found)?;
    found.sort();
    Ok(found)
}

fn walk(dir: &Path, prefix: &str, found: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if prefix.is_empty() && name == ".git" {
            continue;
        }
        let relative = if prefix.is_empty() { name } else { format!("{prefix}/{name}") };
        if entry.file_type()?.is_dir() {
            walk(&entry.path(), &relative, found)?;
        } else {
            found.push(relative);
        }
    }
    Ok(())
}
