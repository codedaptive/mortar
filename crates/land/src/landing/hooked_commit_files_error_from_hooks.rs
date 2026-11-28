//! HookedCommitFilesError from snapshot's CommitPathsHooksError, so commit_files_with_hooks carries
//! commit_paths_with_hooks' refusals out. hooked_commit_files_error_from_hooks_tests covers it.

use crate::landing::{CommitFilesError, HookedCommitFilesError};
use snapshot::CommitPathsHooksError;

impl From<CommitPathsHooksError> for HookedCommitFilesError {
    /// Paths(e) as commit_files' refusal CommitFiles(CommitFilesError::Paths(e)); HookRefused(r) as Refused(r).
    fn from(error: CommitPathsHooksError) -> Self {
        match error {
            CommitPathsHooksError::Paths(error) => HookedCommitFilesError::CommitFiles(CommitFilesError::Paths(error)),
            CommitPathsHooksError::HookRefused(refusal) => HookedCommitFilesError::Refused(refusal),
        }
    }
}
