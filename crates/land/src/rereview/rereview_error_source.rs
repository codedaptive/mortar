//! RereviewError as a std error: its source is the inner GitError or MortarTrailerError; a refusal has none.

use super::RereviewError;

impl std::error::Error for RereviewError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RereviewError::Git(e) => Some(e),
            RereviewError::Trailers(e) => Some(e),
            RereviewError::NotSameChange { .. }
            | RereviewError::NoParent { .. }
            | RereviewError::BasesDiffer { .. } => None,
        }
    }
}
