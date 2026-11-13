//! Git::tree_entry: the blob a path names in a tree. It only runs git, so it has no unit tests; Nagatha's integration
//! test "tree_entry finding a blob, a symlink and a name starting with ':', None for a missing path, refusing a folder
//! path" is to cover it.

use crate::read_tree_entry::read_tree_entry;
use crate::tree_entry_request::tree_entry_request;
use crate::{Git, GitError, Oid};
use std::path::Path;

impl Git {
    /// The id of the blob (a file or a symlink) at `path`, taken from the root of `tree` (a tree or commit id), or
    /// None when the tree has no such path. A path naming a folder or a submodule is refused as not a blob.
    pub fn tree_entry(&self, tree: &Oid, path: &Path) -> Result<Option<Oid>, GitError> {
        let (args, opts) = tree_entry_request(tree, path)?;
        read_tree_entry(&self.run(&args, &opts)?.stdout, path)
    }
}
