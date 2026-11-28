//! HookedCommitFilesError from a CommitFilesError, so `?` carries commit_files_steps' refusals into
//! commit_files_with_hooks. hooked_commit_files_error_from_tests covers it.

use crate::landing::{CommitFilesError, HookedCommitFilesError};

impl From<CommitFilesError> for HookedCommitFilesError {
    /// The refusal unchanged, as CommitFiles.
    fn from(error: CommitFilesError) -> Self {
        HookedCommitFilesError::CommitFiles(error)
    }
}
