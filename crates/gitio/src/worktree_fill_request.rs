//! worktree_fill_request: the git run that writes every file of a worktree's HEAD commit into its folder.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `read-tree --reset -u --no-sparse-checkout HEAD`, run in `folder`. `--reset` sets the index to HEAD's tree and
/// `-u` writes its files, also over tracked files already present. `--no-sparse-checkout` writes every file even
/// where sparse settings apply: with core.sparseCheckout=true in the shared config a plain `checkout` there skips
/// the files outside the sparse set. Not read-only: it writes the folder's index and files. An empty folder path, or
/// one holding a NUL, is refused naming it.
pub(crate) fn worktree_fill_request(folder: &Path) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    Ok((
        ["read-tree", "--reset", "-u", "--no-sparse-checkout", "HEAD"].map(OsString::from).to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            ..RunOpts::default()
        },
    ))
}
