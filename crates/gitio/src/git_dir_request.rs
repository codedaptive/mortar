//! git_dir_request: the git run that names a folder's own git directory.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `rev-parse --path-format=absolute --git-dir`, run in `folder`: a linked worktree answers its own
/// `.git/worktrees/<name>`, not the shared directory. `--path-format=absolute` because git prints a relative `.git`
/// without it. An empty folder path, or one holding a NUL, is refused naming it.
pub(crate) fn git_dir_request(folder: &Path) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    Ok((
        ["rev-parse", "--path-format=absolute", "--git-dir"].map(OsString::from).to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            read_only: true,
            ..RunOpts::default()
        },
    ))
}
