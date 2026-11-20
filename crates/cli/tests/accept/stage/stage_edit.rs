//! Writes one file in a folder, as an agent's edit. Real file I/O: step 1 (flow/run_flow.rs) edits through it on
//! every drive.

use std::fs;
use std::io;
use std::path::Path;

/// Writes `text` to `file` (a path relative to `folder`), making its parent folders first.
pub fn stage_edit(folder: &Path, file: &str, text: &str) -> io::Result<()> {
    let path = folder.join(file);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)
}
