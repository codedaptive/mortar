//! set_index_request: the git run that sets a folder's index to a tree.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `read-tree --end-of-options <tree>` run in `folder`: it replaces the folder's own index with `tree`, leaving the
/// files and HEAD untouched (no `-m`, no `-u`), and the same tree twice gives the same index. Not read-only: it writes
/// the index. The request sets no GIT_INDEX_FILE, so it is the folder's real index unless the caller adds one to
/// `opts.env` or the Git value's env; Git::commit_hooks sets one in the Git value's env to read the tree into a
/// temporary index, running this request itself for a bare repository and through Git::set_index otherwise. The
/// read-tree alone drops a sparse folder's skip-worktree bits; Git::set_index restores them with
/// sparse_reapply_requests' run. An empty folder path, or one holding a NUL, is refused naming it.
pub(crate) fn set_index_request(folder: &Path, tree: &Oid) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    let args = ["read-tree", "--end-of-options", tree.as_str()].map(OsString::from).to_vec();
    let opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        ..RunOpts::default()
    };
    Ok((args, opts))
}
