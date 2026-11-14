//! IndexTreeError: why Git::index_tree gave no tree id.

use crate::GitError;
use std::path::PathBuf;

/// A refusal of Git::index_tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IndexTreeError {
    /// The index holds unmerged entries: each conflicted path once, as git stores it, in git's order.
    Unmerged { paths: Vec<PathBuf> },
    /// Git's own failure, an empty or NUL-holding folder path, or a write-tree answer of the wrong shape.
    Git(GitError),
}

impl std::error::Error for IndexTreeError {}
