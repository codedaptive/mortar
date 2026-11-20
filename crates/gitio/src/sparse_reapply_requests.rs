//! sparse_reapply_requests: the two git runs that read whether a folder is sparse and reapply its sparse patterns.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `config --type=bool --get core.sparseCheckout`, then `sparse-checkout reapply`, both run in `folder`. The first is
/// read-only and takes exit 1 as an answer: the key unset, so the folder is not sparse (read_sparse_flag reads it).
/// It is the key reapply itself checks, read from config.worktree in a linked worktree; reapply refuses (exit 128)
/// when it is off, so it runs only when the flag is true. The reapply is a write (the index's skip-worktree bits) and
/// must follow a read-tree: with no stat info in the index it marks out-of-cone paths without deleting any file,
/// where on a freshly stat'ed index it would delete clean out-of-cone files. An empty folder path, or one holding a
/// NUL, is refused naming it.
pub(crate) fn sparse_reapply_requests(folder: &Path) -> Result<[(Vec<OsString>, RunOpts); 2], GitError> {
    usable_worktree_path(folder)?;
    let flag = ["config", "--type=bool", "--get", "core.sparseCheckout"].map(OsString::from).to_vec();
    let flag_opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        read_only: true,
        answer_codes: vec![1],
        ..RunOpts::default()
    };
    let reapply = ["sparse-checkout", "reapply"].map(OsString::from).to_vec();
    let reapply_opts = RunOpts {
        cwd: Some(folder.to_path_buf()),
        ..RunOpts::default()
    };
    Ok([(flag, flag_opts), (reapply, reapply_opts)])
}
