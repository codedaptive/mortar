//! Appending to the user's savings.jsonl.

use std::fs::{create_dir_all, OpenOptions};
use std::io::Write;
use std::path::Path;

/// Makes `state_dir` if missing and appends `line` to its savings.jsonl, creating the file. Every error is
/// ignored and nothing is printed: the log never changes an answer or its exit code.
pub fn append_savings_log(state_dir: &Path, line: &str) {
    let _ = create_dir_all(state_dir)
        .and_then(|()| OpenOptions::new().append(true).create(true).open(state_dir.join("savings.jsonl")))
        .and_then(|mut file| file.write_all(line.as_bytes()));
}
