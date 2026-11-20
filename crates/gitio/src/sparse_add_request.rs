//! sparse_add_request: the git run that adds folders to a folder's sparse paths.

use crate::usable_sparse_path::usable_sparse_path;
use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `sparse-checkout add -- <paths>...` run in `folder`; git fills the added folders itself. Not read-only: it writes
/// the folder's sparse settings, index and files. An empty list (nothing to add) is refused; an unusable folder
/// path or sparse path (see usable_sparse_path) is refused naming it.
pub(crate) fn sparse_add_request(folder: &Path, paths: &[String]) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    if paths.is_empty() {
        return Err(GitError::refusal(Vec::new(), "no sparse paths to add"));
    }
    let mut args: Vec<OsString> = ["sparse-checkout", "add", "--"].map(OsString::from).to_vec();
    for path in paths {
        usable_sparse_path(path)?;
        args.push(path.into());
    }
    let opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        ..RunOpts::default()
    };
    Ok((args, opts))
}
