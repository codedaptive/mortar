//! set_index_paths: set a folder's own index entries for some paths to a tree's, leaving every other entry as staged.
//! No unit test: it only runs git and holds no decision of its own (real_index_git has its tests). The integration
//! tests in tests/snapshot/commit_paths_cases.rs cover it through commit_paths:
//! commit_paths_added_changed_deleted_and_explicit_excluded_files (the named paths staged as the commit has them,
//! a deleted one removed) and commit_paths_preserves_unnamed_worktree_and_index_edits (other staged content kept).

use crate::real_index_git::real_index_git;
use gitio::{Git, GitError, IndexTreeError, Oid};
use std::path::{Path, PathBuf};

/// Reads the tree the index of `folder` holds, takes each of `paths` from `tree` into it (a path `tree` lacks is
/// removed) and sets the index to the result. The worktree is not touched. Runs on real_index_git, so a
/// GIT_INDEX_FILE on `git` does not redirect it from the folder's own index. index_tree's refusals pass through;
/// a refusal of tree_with_paths_typed or a failed set_index is Git.
pub(crate) fn set_index_paths(git: &Git, folder: &Path, tree: &Oid, paths: &[PathBuf]) -> Result<(), IndexTreeError> {
    let real = real_index_git(git);
    let index = real.index_tree(folder)?;
    let merged = real
        .tree_with_paths_typed(&index, tree, paths)
        .map_err(|error| IndexTreeError::Git(GitError::from(error)))?;
    real.set_index(folder, &merged).map_err(IndexTreeError::Git)
}
