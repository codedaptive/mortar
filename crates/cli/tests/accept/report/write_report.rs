//! Writes the report file. Real file I/O with no sibling check: the drive (main.rs, `done_list`) writes the report
//! through it on every run.

use std::fs;
use std::io;
use std::path::Path;

/// Makes the report's folder if it is missing and writes the text, replacing an earlier report.
pub fn write_report(path: &Path, text: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text)
}
