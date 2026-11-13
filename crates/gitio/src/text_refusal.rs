//! text_refusal: a refusal naming the text that was refused.

use crate::GitError;

/// A refusal whose message is `<what>: <text as a quoted, escaped string>`, so a NUL or line break in the text shows.
pub(crate) fn text_refusal(what: &str, text: &str) -> GitError {
    GitError::refusal(Vec::new(), format!("{what}: {text:?}"))
}
