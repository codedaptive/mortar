//! no_nul: refuse caller text holding a NUL before it goes into git's argv or stdin.

use crate::GitError;

/// Ok when `value` holds no NUL byte. A NUL cannot pass through argv and would cut or shift git's input, so it is
/// refused with a message naming `what` and the text (lossy UTF-8).
pub(crate) fn no_nul(what: &str, value: &[u8]) -> Result<(), GitError> {
    if !value.contains(&0) {
        return Ok(());
    }
    Err(GitError::refusal(
        Vec::new(),
        format!("{what} holds a NUL: {:?}", String::from_utf8_lossy(value)),
    ))
}
