//! worktree_add_request: the git run that adds a worktree without filling its folder.

use crate::no_nul::no_nul;
use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `worktree add --no-checkout -b <branch> -- <path> <commit>`, or `--detach` in place of `-b <branch>` when no
/// branch is given. `--no-checkout` leaves the folder empty so the caller can set sparse paths before any file is
/// written. An empty path or one holding a NUL is refused; so is a branch that is empty, holds a NUL or starts with
/// '-' (git would read it as an option). Not read-only: it writes the worktree and the branch.
pub(crate) fn worktree_add_request(
    path: &Path,
    commit: &Oid,
    branch: Option<&str>,
) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(path)?;
    let mut args: Vec<OsString> = ["worktree", "add", "--no-checkout"].map(OsString::from).to_vec();
    match branch {
        Some(name) => {
            if name.is_empty() || name.starts_with('-') {
                return Err(GitError::refusal(
                    Vec::new(),
                    format!("not a usable branch name (empty or starting with '-'): {name:?}"),
                ));
            }
            no_nul("branch", name.as_bytes())?;
            args.extend(["-b".into(), name.into()]);
        }
        None => args.push("--detach".into()),
    }
    args.extend(["--".into(), path.as_os_str().to_os_string(), commit.as_str().into()]);
    Ok((args, RunOpts::default()))
}
