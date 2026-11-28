//! CommitPathsHooksError: why commit_paths_with_hooks wrote no commit, or wrote one and could not set the folder's
//! index. It replaces CommitPathsError as commit_paths' error once cli and land have moved to the hooked twin, and a
//! later snapshot order removes the hookless commit_paths. A declaration, its From and its std::error::Error impl:
//! its Display is commit_paths_hooks_error_display, tested per variant in commit_paths_hooks_error_display_tests.rs.
//! The integration tests in tests/snapshot/commit_paths_cases.rs get HookRefused (listed in
//! commit_paths_with_hooks.rs).

use crate::CommitPathsError;

/// Derives Debug only, as CommitPathsError does.
#[derive(Debug)]
pub enum CommitPathsHooksError {
    /// Any error the hookless commit_paths gives, unchanged.
    Paths(CommitPathsError),
    /// A pre-commit or commit-msg hook refused the commit: no commit was written and the index was not touched. The
    /// refusal carries the hook's name, exit code, stdout and stderr.
    HookRefused(gitio::HookRefusal),
}

impl From<CommitPathsError> for CommitPathsHooksError {
    fn from(error: CommitPathsError) -> Self {
        CommitPathsHooksError::Paths(error)
    }
}

impl std::error::Error for CommitPathsHooksError {}
