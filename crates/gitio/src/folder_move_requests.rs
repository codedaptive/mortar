//! folder_move_requests: the two git runs that move a folder's index and files from one tree to another.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// `update-index -q --unmerged --refresh`, then `read-tree -m -u --end-of-options <old> <new>`, both run in `folder`
/// and neither read-only. The refresh writes the index's stat data, so a file only touched (same content, new mtime)
/// is not taken for a local edit; `--unmerged` keeps it at exit 0 on an unmerged index, which read-tree then reports.
/// read-tree's exit 128 is an answer: it is how git refuses a move over a local change (read_folder_move tells it from
/// git's other failures). An empty folder path, or one holding a NUL, is refused naming it.
pub(crate) fn folder_move_requests(
    folder: &Path,
    old: &Oid,
    new: &Oid,
) -> Result<[(Vec<OsString>, RunOpts); 2], GitError> {
    usable_worktree_path(folder)?;
    let refresh = (
        ["update-index", "-q", "--unmerged", "--refresh"].map(OsString::from).to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            ..RunOpts::default()
        },
    );
    let read_tree = (
        ["read-tree", "-m", "-u", "--end-of-options", old.as_str(), new.as_str()].map(OsString::from).to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            answer_codes: vec![128],
            ..RunOpts::default()
        },
    );
    Ok([refresh, read_tree])
}
