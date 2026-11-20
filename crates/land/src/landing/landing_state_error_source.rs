//! LandingStateError as a std error: its source is the io error for Read, Write and Remove; Corrupt has none.

use crate::landing::LandingStateError;

impl std::error::Error for LandingStateError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LandingStateError::Read { source, .. }
            | LandingStateError::Write { source, .. }
            | LandingStateError::Remove { source, .. } => Some(source),
            LandingStateError::Corrupt { .. } => None,
        }
    }
}
