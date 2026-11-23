//! The message of an `mr commit-files` commit when the caller gives no --message.
//! default_commit_message_tests covers it.

use std::path::PathBuf;

/// "commit-files: " and the paths as displayed, joined by ", ".
pub fn default_commit_message(paths: &[PathBuf]) -> String {
    let shown: Vec<String> = paths.iter().map(|path| path.display().to_string()).collect();
    format!("commit-files: {}", shown.join(", "))
}
