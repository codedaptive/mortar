//! HookedCommitFilesError: why commit_files_with_hooks was refused. A declaration only, with no sibling test;
//! tests/commit_files_zit/hooks.rs exercises Refused, hooked_commit_files_error_from_tests the CommitFiles arm.

use crate::landing::CommitFilesError;

/// A commit_files_with_hooks refusal. Debug only: CommitFilesError is neither Clone nor Eq.
#[derive(Debug)]
pub enum HookedCommitFilesError {
    /// Every refusal commit_files makes, as it makes it.
    CommitFiles(CommitFilesError),
    /// pre-commit or commit-msg refused the commit: no commit, no landing-state write, no op, integration unmoved,
    /// the folder's index untouched.
    Refused(gitio::HookRefusal),
}
