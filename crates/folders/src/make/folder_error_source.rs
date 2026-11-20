//! FolderError as a std error: a wrapped error, or an Io refusal's io::Error, is its source; the rest have none.

use super::FolderError;

impl std::error::Error for FolderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FolderError::Plan(e) => Some(e),
            FolderError::Record { error, .. } => Some(error),
            FolderError::Git(e) => Some(e),
            FolderError::Oplog(e) => Some(e),
            FolderError::Lock(e) => Some(e),
            FolderError::BelowFloor(e) => Some(e),
            FolderError::Io { source, .. } => Some(source),
            FolderError::DiskProbe { .. }
            | FolderError::DifferentFolder { .. }
            | FolderError::NotAFolder { .. }
            | FolderError::BranchCheckedOut { .. }
            | FolderError::BranchExists { .. }
            | FolderError::ZoneBranchMissing { .. }
            | FolderError::RequestBranchMissing { .. }
            | FolderError::AttemptMissing { .. }
            | FolderError::AttemptsUnsaved { .. }
            | FolderError::AttemptsRemain { .. }
            | FolderError::OrphanAttempt { .. }
            | FolderError::OrphanWithoutSnapshot { .. }
            | FolderError::RequestMovedOn { .. }
            | FolderError::AttemptFolder { .. }
            | FolderError::NotSaved { .. }
            | FolderError::MoveRefused { .. }
            | FolderError::BranchElsewhere { .. }
            | FolderError::NotRemovedUnsaved { .. }
            | FolderError::NotJoined { .. }
            | FolderError::ZoneFolder { .. }
            | FolderError::RefMoved { .. }
            | FolderError::AttemptRefsLeft { .. }
            | FolderError::RemovedRefsLeft { .. }
            | FolderError::ExtraRefRefused { .. } => None,
        }
    }
}
