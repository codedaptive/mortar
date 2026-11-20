//! TreeWithPathsError: how Git::tree_with_paths_typed fails.

use crate::{GitError, TreeWithPathsRefusal};

/// A refusal made before git builds anything, or git's own failure (or an ls-tree answer of the wrong shape) as
/// GitError.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TreeWithPathsError {
    /// The paths were refused; no temporary index was made.
    Refused(TreeWithPathsRefusal),
    /// A git run failed, or its answer was not of the shape asked for.
    Git(GitError),
}

impl std::error::Error for TreeWithPathsError {}
