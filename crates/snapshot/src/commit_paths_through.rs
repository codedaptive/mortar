//! commit_paths_through: the body of commit_paths and commit_paths_with_hooks, with the hook step as a parameter.
//! No unit test: it runs git and reads the file system; outside_path is its pure decision and has its tests.
//! commit_paths.rs has the behaviour and the integration cases.

use crate::outside_path::outside_path;
use crate::set_index_paths::set_index_paths;
use crate::symlinked_ancestor::symlinked_ancestor;
use crate::{CommitPathsError, CommitPathsOutcome};
use gitio::{Git, Oid, TreeEntryError, TreeWithPathsError};
use std::path::{Path, PathBuf};

/// Commits `paths` onto `parent` as commit_paths says, with `hooks` run after the Same check and before the commit
/// is written: it gets the tree and `message` and answers the commit's message. Its error is returned as it is, with
/// no commit written and the index untouched. Errors are commit_paths', then `hooks`' own.
pub(crate) fn commit_paths_through<E: From<CommitPathsError>>(
    git: &Git,
    folder: &Path,
    parent: &Oid,
    paths: &[PathBuf],
    message: &str,
    hooks: impl FnOnce(&Git, &Oid, &str) -> Result<String, E>,
) -> Result<CommitPathsOutcome, E> {
    for path in paths {
        if let Some(refusal) = outside_path(path) {
            return Err(refusal.into());
        }
        symlinked_ancestor(folder, path)?;
    }
    let parent_tree = git.commit_info(parent).map_err(CommitPathsError::Git)?.tree;
    let from = git.snapshot_tree(parent, &[]).map_err(CommitPathsError::Git)?;
    // Runs before refuse_not_added, so a path it refuses (empty, '/'-ended, a '.' part) is Paths, not NotAdded.
    let tree = git.tree_with_paths_typed(&parent_tree, &from, paths).map_err(|error| match error {
        TreeWithPathsError::Refused(refusal) => CommitPathsError::Paths(refusal),
        TreeWithPathsError::Git(error) => CommitPathsError::Git(error),
    })?;
    for path in paths {
        refuse_not_added(git, folder, &from, path)?;
    }
    if tree == parent_tree {
        return Ok(CommitPathsOutcome::Same);
    }
    let message = hooks(git, &tree, message)?;
    let commit = git
        .commit_tree(&tree, std::slice::from_ref(parent), &message, None, None)
        .map_err(CommitPathsError::Git)?;
    set_index_paths(git, folder, &tree, paths)
        .map_err(|error| CommitPathsError::IndexNotSet { commit: commit.clone(), error })?;
    Ok(CommitPathsOutcome::Committed { commit, tree })
}

/// NotAdded when `path` is on disk (lstat: a symlink counts whatever it points at) and `from` has no entry for it.
/// A folder or gitlink entry in `from` counts as present, so tree_with_paths_typed refuses a folder typed
/// (FolderInFrom) and takes a gitlink.
fn refuse_not_added(git: &Git, folder: &Path, from: &Oid, path: &Path) -> Result<(), CommitPathsError> {
    if std::fs::symlink_metadata(folder.join(path)).is_err() {
        return Ok(());
    }
    match git.tree_entry_typed(from, path) {
        Ok(Some(_)) | Err(TreeEntryError::NotABlob { .. }) => Ok(()),
        Ok(None) => Err(CommitPathsError::NotAdded { path: path.to_path_buf() }),
        Err(TreeEntryError::Git(error)) => Err(CommitPathsError::Git(error)),
    }
}
