//! read_message: git output that must be UTF-8 text.

use crate::GitError;

/// The output unchanged as a String. Output that is not UTF-8 is refused naming the bytes (lossy).
pub(crate) fn read_message(stdout: &[u8]) -> Result<String, GitError> {
    String::from_utf8(stdout.to_vec()).map_err(|_| {
        GitError::refusal(
            Vec::new(),
            format!("git output is not UTF-8: {:?}", String::from_utf8_lossy(stdout)),
        )
    })
}
