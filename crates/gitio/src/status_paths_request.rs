//! status_paths_request: the git run that lists a folder's changed, untracked and conflicted paths.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `status --porcelain=v2 -z --untracked-files=all --no-renames`, read-only, run in `folder`. `-z` keeps paths holding
/// spaces, quotes or line breaks as they are; `--untracked-files=all` names each untracked file, not its folder;
/// `--no-renames` shows a rename as its old path deleted and its new path added. An empty folder path, or one holding
/// a NUL, is refused naming it.
pub(crate) fn status_paths_request(folder: &Path) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    let args = ["status", "--porcelain=v2", "-z", "--untracked-files=all", "--no-renames"].map(OsString::from).to_vec();
    let opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        read_only: true,
        ..RunOpts::default()
    };
    Ok((args, opts))
}
