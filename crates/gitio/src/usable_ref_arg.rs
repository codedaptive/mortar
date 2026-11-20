//! usable_ref_arg: refuse a ref name or ref pattern git could not be given safely as one argument or refspec side.

use crate::no_nul::no_nul;
use crate::GitError;

/// Ok for non-empty text without a NUL, a leading '-' or a ':'. Empty names no ref; a NUL cannot pass through argv;
/// a leading '-' could be read as an option; ':' is the refspec separator and no refname may hold one, so text with
/// it would split a refspec where the caller did not mean to. Each is refused naming `what` and the text.
pub(crate) fn usable_ref_arg(what: &str, text: &str) -> Result<(), GitError> {
    if text.is_empty() || text.starts_with('-') || text.contains(':') {
        return Err(GitError::refusal(Vec::new(), format!("not a usable {what}: {text:?}")));
    }
    no_nul(what, text.as_bytes())
}
