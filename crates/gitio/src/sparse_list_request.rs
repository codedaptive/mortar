//! sparse_list_request: the git run that lists a folder's sparse paths.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `-c core.sparseCheckoutCone=true sparse-checkout list`, read-only, run in `folder`. Plain `list` prints a non-cone
/// folder's raw patterns, which cannot be told from paths; with cone mode forced, git also warns on stderr about a
/// pattern cone mode cannot read, so read_sparse_list can refuse it. An unusable folder path is refused naming it.
pub(crate) fn sparse_list_request(folder: &Path) -> Result<(Vec<OsString>, RunOpts), GitError> {
    usable_worktree_path(folder)?;
    let args = ["-c", "core.sparseCheckoutCone=true", "sparse-checkout", "list"].map(OsString::from).to_vec();
    let opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        read_only: true,
        ..RunOpts::default()
    };
    Ok((args, opts))
}
