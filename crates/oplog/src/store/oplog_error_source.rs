//! OplogError as a std error: Io and Git refusals expose the underlying error as their source; the rest have none.

use super::OplogError;

impl std::error::Error for OplogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            OplogError::Io { source, .. } => Some(source),
            OplogError::Git(e) => Some(e),
            OplogError::Malformed { .. }
            | OplogError::Mixed { .. }
            | OplogError::BadMove { .. }
            | OplogError::Protected { .. }
            | OplogError::NothingToUndo { .. }
            | OplogError::MovedSince { .. }
            | OplogError::UnknownOp { .. }
            | OplogError::ValueUnknown { .. } => None,
        }
    }
}
