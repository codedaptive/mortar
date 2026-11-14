//! From<TreeEntryError> for GitError: a typed tree_entry refusal as the GitError Git::tree_entry returns.

use crate::{GitError, TreeEntryError};

impl From<TreeEntryError> for GitError {
    /// NotABlob becomes a refusal with no args whose stderr is its Display text; Git is its GitError unchanged.
    fn from(error: TreeEntryError) -> GitError {
        match error {
            TreeEntryError::NotABlob { .. } => GitError::refusal(Vec::new(), error.to_string()),
            TreeEntryError::Git(error) => error,
        }
    }
}
