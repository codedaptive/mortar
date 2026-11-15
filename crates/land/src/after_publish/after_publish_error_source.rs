//! AfterPublishError as a std error: its source is the wrapped git, squash, join or landing error.

use crate::after_publish::AfterPublishError;

impl std::error::Error for AfterPublishError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AfterPublishError::Git(e) => Some(e),
            AfterPublishError::Squash(e) => Some(e),
            AfterPublishError::Join(e) => Some(e),
            AfterPublishError::Land(e) => Some(e),
            AfterPublishError::Unrelated { .. }
            | AfterPublishError::NoSquashOf { .. }
            | AfterPublishError::SquashOfNotInZone { .. } => None,
        }
    }
}
