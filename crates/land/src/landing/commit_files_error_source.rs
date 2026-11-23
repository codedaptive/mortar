//! CommitFilesError as a std error: its source is the wrapped commit_paths or land error.
//! commit_files_error_source_tests covers every variant.

use crate::landing::CommitFilesError;

impl std::error::Error for CommitFilesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CommitFilesError::Paths(e) => Some(e),
            CommitFilesError::Land(e) => Some(e),
            CommitFilesError::NotLanding | CommitFilesError::NotLanded { .. } | CommitFilesError::FolderOffTip { .. } => {
                None
            }
        }
    }
}
