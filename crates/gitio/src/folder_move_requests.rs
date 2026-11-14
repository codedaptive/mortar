//! folder_move_requests: the three git runs that move a folder's index and files from one tree to another.

use crate::usable_worktree_path::usable_worktree_path;
use crate::{GitError, Oid, RunOpts};
use std::ffi::OsString;
use std::path::Path;

/// Three runs, all in `folder`. First `update-index -q --unmerged --refresh`: it writes the index's stat data, so a
/// file only touched (same content, new mtime) is not taken for a local edit; `--unmerged` keeps it at exit 0 on an
/// unmerged index, which read-tree then reports. Second, the check: `read-tree --no-sparse-checkout -n -m -u
/// --end-of-options <old> <new>`, read-only, a dry run that writes neither index nor files. In a sparse folder git's
/// real run only warns about an untracked, not ignored file at a path it adds inside the cone, and then writes over it;
/// with sparse checkout off the dry run applies git's plain check, which refuses it. An untracked, not ignored file
/// at a path the move adds outside the cone is refused too, though the sparse run would leave it alone. Third, the
/// move: `read-tree -m -u --end-of-options <old> <new>`, not read-only. Exit 128 from the check or the move is an
/// answer: it is how git refuses a move (read_folder_move tells it from git's other failures). An empty folder
/// path, or one holding a NUL, is refused naming it.
pub(crate) fn folder_move_requests(
    folder: &Path,
    old: &Oid,
    new: &Oid,
) -> Result<[(Vec<OsString>, RunOpts); 3], GitError> {
    usable_worktree_path(folder)?;
    let refresh = (
        ["update-index", "-q", "--unmerged", "--refresh"].map(OsString::from).to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            ..RunOpts::default()
        },
    );
    let check = (
        ["read-tree", "--no-sparse-checkout", "-n", "-m", "-u", "--end-of-options", old.as_str(), new.as_str()]
            .map(OsString::from)
            .to_vec(),
        RunOpts {
            cwd: Some(folder.to_path_buf()),
            answer_codes: vec![128],
            read_only: true,
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
    Ok([refresh, check, read_tree])
}
