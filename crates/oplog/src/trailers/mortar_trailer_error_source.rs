//! MortarTrailerError as a std error: its source is the inner GitError or ChangeIdError.

use super::MortarTrailerError;

impl std::error::Error for MortarTrailerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MortarTrailerError::Git(e) => Some(e),
            MortarTrailerError::ChangeId(e) => Some(e),
        }
    }
}
