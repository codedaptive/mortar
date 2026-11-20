//! worktree_remove_request: the git run that removes a worktree and its folder.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `worktree remove [--force] -- <path>`. An empty path or one holding a NUL is refused. Not read-only: it deletes
/// the folder and the worktree's records.
pub(crate) fn worktree_remove_request(path: &Path, force: bool) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(path)?;
    let mut args: Vec<OsString> = ["worktree", "remove"].map(OsString::from).to_vec();
    if force {
        args.push("--force".into());
    }
    args.extend(["--".into(), path.as_os_str().to_os_string()]);
    Ok((args, RunOpts::default()))
}
