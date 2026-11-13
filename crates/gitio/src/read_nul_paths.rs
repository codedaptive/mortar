//! read_nul_paths: read a list of NUL-ended paths, as `-z` output gives it.

use crate::path_from_bytes::path_from_bytes;
use crate::GitError;
use std::path::PathBuf;

/// Each NUL-ended path in order, bytes as they are (spaces, newlines and non-UTF-8 kept). Empty output is no paths.
/// An empty path or a last path without its NUL is refused naming the output.
pub(crate) fn read_nul_paths(stdout: &[u8]) -> Result<Vec<PathBuf>, GitError> {
    if stdout.is_empty() {
        return Ok(Vec::new());
    }
    let refuse = |why: &str| {
        GitError::refusal(
            Vec::new(),
            format!("not a NUL-ended path list ({why}): {:?}", String::from_utf8_lossy(stdout)),
        )
    };
    let body = stdout.strip_suffix(b"\0").ok_or_else(|| refuse("no final NUL"))?;
    body.split(|b| *b == 0)
        .map(|path| {
            if path.is_empty() {
                return Err(refuse("an empty path"));
            }
            Ok(path_from_bytes(path))
        })
        .collect()
}
