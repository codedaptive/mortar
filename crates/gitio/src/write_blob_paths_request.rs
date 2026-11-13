//! write_blob_paths_request: the git run that writes a tree's version of named paths into a folder.

use crate::usable_path::usable_path;
use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// `checkout --end-of-options <tree> -- <paths>...` run in `folder` with GIT_LITERAL_PATHSPECS=1, so each path names
/// a file and never a pattern; paths go into argv as their bytes. Not read-only: it writes the files and the index.
/// An empty list (git with no paths would try to switch the folder to the tree) is refused; an unusable folder
/// path, and an empty path or one holding a NUL, are refused naming it.
pub(crate) fn write_blob_paths_request(
    folder: &Path,
    tree: &Oid,
    paths: &[PathBuf],
) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    if paths.is_empty() {
        return Err(GitError::refusal(Vec::new(), "no paths to write"));
    }
    let mut args: Vec<OsString> = ["checkout", "--end-of-options", tree.as_str(), "--"].map(OsString::from).to_vec();
    for path in paths {
        usable_path(path)?;
        args.push(path.into());
    }
    let opts = RunOpts {
        env: vec![("GIT_LITERAL_PATHSPECS".into(), "1".into())],
        cwd: Some(folder.to_path_buf()),
        ..RunOpts::default()
    };
    Ok((args, opts))
}
