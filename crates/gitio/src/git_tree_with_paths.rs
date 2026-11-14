//! Git::tree_with_paths: tree_with_paths_typed with each refusal as a GitError. It only runs git, so it has no unit
//! tests; its integration cases are listed with tree_with_paths_typed (git_tree_with_paths_typed.rs).

use crate::{Git, GitError, Oid};
use std::path::PathBuf;

impl Git {
    /// The same tree as tree_with_paths_typed, each refusal as a GitError with no exit code, no args and the
    /// refusal's Display text as stderr; git's own failures are its GitError unchanged. New callers use
    /// tree_with_paths_typed, which says which refusal it was: a TreeWithPathsRefusal of FolderInFrom, FolderInBase,
    /// Overlap, NotAsStored or Unusable.
    pub fn tree_with_paths(&self, base: &Oid, from: &Oid, paths: &[PathBuf]) -> Result<Oid, GitError> {
        self.tree_with_paths_typed(base, from, paths).map_err(GitError::from)
    }
}
