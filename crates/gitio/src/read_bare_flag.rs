//! read_bare_flag: read the answer of `rev-parse --is-bare-repository`.

use crate::GitError;

/// Whether the repository is bare: stdout exactly "true\n" is bare and "false\n" is not. Any other stdout is refused
/// naming it.
pub(crate) fn read_bare_flag(stdout: &[u8]) -> Result<bool, GitError> {
    match stdout {
        b"true\n" => Ok(true),
        b"false\n" => Ok(false),
        other => Err(GitError::refusal(
            Vec::new(),
            format!("not an --is-bare-repository answer: {:?}", String::from_utf8_lossy(other)),
        )),
    }
}
