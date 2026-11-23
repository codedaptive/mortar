//! commit_paths: one commit of a parent's tree with some named paths set as the folder has them, and the folder's
//! index set for those paths. It is for land (the integration folder) and cli (the zone folder); neither calls it yet.
//! No unit test: it runs git and reads the file system; outside_path is its pure decision and has its tests. The
//! integration tests in tests/snapshot/commit_paths_cases.rs cover it, in a throwaway repository:
//! - an added, a changed and a deleted path, a path under target/ and a file over the default size cap in one commit
//!   with the given parent and message, the index set for them:
//!   commit_paths_added_changed_deleted_and_explicit_excluded_files;
//! - unnamed worktree and staged edits kept out of the commit and left in the worktree and the index:
//!   commit_paths_preserves_unnamed_worktree_and_index_edits;
//! - Same writes no commit, ref, index or op log entry, and in that test no object, as every file of the folder is
//!   already stored (snapshot_tree writes a blob for each file the object store lacks, even on Same):
//!   commit_paths_same_leaves_refs_head_index_and_objects_unchanged;
//! - an absolute path, a `..` part and a symlinked ancestor refused with nothing written:
//!   commit_paths_refuses_absolute, commit_paths_refuses_parent_part, commit_paths_refuses_symlink_ancestor;
//! - Committed moves no ref and writes no op log entry:
//!   commit_paths_committed_moves_no_refs_and_writes_no_oplog_entry;
//! - an ignored named file is NotAdded with no commit: commit_paths_refuses_ignored_named_file_without_a_commit.

use crate::outside_path::outside_path;
use crate::set_index_paths::set_index_paths;
use crate::symlinked_ancestor::symlinked_ancestor;
use crate::{CommitPathsError, CommitPathsOutcome};
use gitio::{Git, Oid, TreeEntryError, TreeWithPathsError};
use std::path::{Path, PathBuf};

/// Writes one commit whose tree is `parent`'s with each of `paths` (relative to `folder`) as the folder has it:
/// added or changed to the file's contents, a symlink as a symlink, removed when absent from the folder. Its one
/// parent is `parent` and its message is `message` as given. `git` must be rooted at the folder's top (`git.repo` is
/// `folder`), as for save. No size cap and no excludes apply: the caller named the paths. Objects only: no ref moves
/// and no op is logged; the caller's op moves its ref to the commit.
///
/// Every path is checked before git runs, and refused with nothing written when it is absolute (Absolute), has a
/// `..` part (ParentPart), or has a symlinked folder on the way to it (ThroughSymlink, or Ancestor when that folder
/// cannot be looked at); the path's own last part may be a symlink. The folder is then hashed by gitio's
/// snapshot_tree, starting from `parent`, with no excludes: it hashes every non-ignored file of the folder and
/// writes a blob for each the object store does not yet hold, even when the answer is Same, since gitio has no step
/// that hashes only the named paths. A path list tree_with_paths_typed refuses is Paths. A named path on disk that
/// the hashed tree lacks (git ignores it) is NotAdded, with no commit written.
///
/// A tree equal to `parent`'s answers Same: no commit, and the index is left as it was. Otherwise the commit is
/// written and the folder's index entries for exactly the named paths are set to the commit's (IndexNotSet when
/// that fails); every other index entry keeps its staged content and no file is touched. With `parent` at HEAD, the
/// named paths show as staged against HEAD until the caller moves HEAD's branch to the commit.
pub fn commit_paths(
    git: &Git,
    folder: &Path,
    parent: &Oid,
    paths: &[PathBuf],
    message: &str,
) -> Result<CommitPathsOutcome, CommitPathsError> {
    for path in paths {
        if let Some(refusal) = outside_path(path) {
            return Err(refusal);
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
    let commit = git
        .commit_tree(&tree, std::slice::from_ref(parent), message, None, None)
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
