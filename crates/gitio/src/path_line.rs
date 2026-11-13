//! path_line: read git output that is one absolute path.

use crate::path_from_bytes::path_from_bytes;
use crate::GitError;
use std::path::PathBuf;

/// The path with one trailing newline removed. An empty or relative path is refused naming the text: callers
/// ask git for absolute paths, so anything else means git answered something other than what was asked.
pub(crate) fn path_line(stdout: &[u8]) -> Result<PathBuf, GitError> {
    let path = path_from_bytes(stdout.strip_suffix(b"\n").unwrap_or(stdout));
    if path.is_absolute() {
        return Ok(path);
    }
    Err(GitError::refusal(
        Vec::new(),
        format!("not an absolute path line: {:?}", String::from_utf8_lossy(stdout)),
    ))
}
