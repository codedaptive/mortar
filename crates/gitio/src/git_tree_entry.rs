//! Git::tree_entry: the blob a path names in a tree, with every refusal a GitError. It only runs git, so it has no
//! unit tests; the integration case lives with Git::tree_entry_typed (git_tree_entry_typed.rs).

use crate::{Git, GitError, Oid};
use std::path::Path;

impl Git {
    /// The same answer as tree_entry_typed, with NotABlob as a GitError with no exit code, no args and its Display
    /// text as stderr (`not a blob: "sub" is a "commit"` for a gitlink). New callers use tree_entry_typed.
    pub fn tree_entry(&self, tree: &Oid, path: &Path) -> Result<Option<Oid>, GitError> {
        self.tree_entry_typed(tree, path).map_err(GitError::from)
    }
}
