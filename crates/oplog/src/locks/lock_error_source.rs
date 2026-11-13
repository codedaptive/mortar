//! LockError as a std error: an Io refusal exposes the underlying io::Error as its source.

use super::LockError;

impl std::error::Error for LockError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LockError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
