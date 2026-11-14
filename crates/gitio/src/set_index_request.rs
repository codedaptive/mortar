//! set_index_request: the git run that sets a folder's index to a tree.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `read-tree --end-of-options <tree>` run in `folder`: it replaces the folder's own index with `tree`, leaving the
/// files and HEAD untouched (no `-m`, no `-u`), and the same tree twice gives the same index. Not read-only: it writes
/// the index. No GIT_INDEX_FILE is set, so it is the folder's real index. In a sparse folder the skip-worktree bits are
/// as git's read-tree gives them. An empty folder path, or one holding a NUL, is refused naming it.
pub(crate) fn set_index_request(folder: &Path, tree: &Oid) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    let args = ["read-tree", "--end-of-options", tree.as_str()].map(OsString::from).to_vec();
    let opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        ..RunOpts::default()
    };
    Ok((args, opts))
}
