//! Git::tree_entry_typed: the blob a path names in a tree, with a typed refusal. It only runs git, so it has no unit
//! tests; the zone's integration test tests/typed_refusals/tree_entry.rs covers it: "tree_entry finding a blob, None
//! for a missing path, a folder path refused as NotABlob { kind: Tree }, a gitlink as NotABlob { kind: Commit }". A
//! symlink and a name starting with ':' are not covered yet.

use crate::read_tree_entry::read_tree_entry;
use crate::tree_entry_request::tree_entry_request;
use crate::{Git, Oid, TreeEntryError};
use std::path::Path;

impl Git {
    /// The id of the blob (a file or a symlink) at `path`, taken from the root of `tree` (a tree or commit id), or
    /// None when the tree has no such path. A path naming a folder, a submodule or another non-blob entry is refused
    /// as NotABlob, carrying the path and the kind; every other refusal is Git.
    pub fn tree_entry_typed(&self, tree: &Oid, path: &Path) -> Result<Option<Oid>, TreeEntryError> {
        let (args, opts) = tree_entry_request(tree, path).map_err(TreeEntryError::Git)?;
        read_tree_entry(&self.run(&args, &opts).map_err(TreeEntryError::Git)?.stdout, path)
    }
}
