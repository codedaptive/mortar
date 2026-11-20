//! The git run that points a folder's HEAD at its branch.

use std::ffi::OsString;
use std::path::Path;

use gitio::RunOpts;

/// `symbolic-ref HEAD refs/heads/<branch>` run in `folder`; a write command. gitio's worktree_add can only
/// create a branch or detach, and an existing branch (every folder's branch is made first through oplog::apply,
/// or already exists) cannot be created again, so its folder is added detached at the branch's commit and this run then points its HEAD
/// at the branch.
pub(crate) fn attach_head_request(folder: &Path, branch: &str) -> (Vec<OsString>, RunOpts) {
    let args = vec![
        OsString::from("symbolic-ref"),
        OsString::from("HEAD"),
        OsString::from(format!("refs/heads/{branch}")),
    ];
    let opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        ..RunOpts::default()
    };
    (args, opts)
}
