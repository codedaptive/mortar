//! BackupError as a std error: its source is the wrapped lock, git or op log error.
//! backup_error_error_tests cover it.

use crate::backup::BackupError;

impl std::error::Error for BackupError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BackupError::Host(e) => Some(e),
            BackupError::Git(e) => Some(e),
            BackupError::Oplog(e) => Some(e),
            BackupError::BadHost(_)
            | BackupError::Refused { .. }
            | BackupError::OpFailed(_)
            | BackupError::OpRepeated(_) => None,
        }
    }
}
