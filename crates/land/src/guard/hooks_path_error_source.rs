//! HooksPathError as a std error: its source is the inner GitError; a Relative refusal has none.

use super::HooksPathError;

impl std::error::Error for HooksPathError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            HooksPathError::Git(e) => Some(e),
            HooksPathError::Relative { .. } => None,
        }
    }
}
