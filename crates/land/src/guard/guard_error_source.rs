//! GuardError as a std error: its source is the inner GitError or io::Error; a refusal has none.

use super::GuardError;

impl std::error::Error for GuardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            GuardError::Git(e) => Some(e),
            GuardError::Io { error, .. } => Some(error),
            GuardError::ForeignHook { .. } | GuardError::BadConfig { .. } => None,
        }
    }
}
