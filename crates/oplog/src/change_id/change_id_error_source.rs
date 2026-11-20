//! ChangeIdError as a std error: a Random failure exposes the underlying io::Error as its source.

use super::ChangeIdError;

impl std::error::Error for ChangeIdError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ChangeIdError::Random { source } => Some(source),
            ChangeIdError::Malformed { .. } => None,
        }
    }
}
