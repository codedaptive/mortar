//! save_head: the branch and commit a save moves from, out of the folder's HEAD.

use crate::SnapshotError;
use gitio::Oid;
use std::path::Path;

/// `head` is gitio's head(): (the branch's full ref, None when detached; the commit, None when unborn). No commit
/// gives UnbornHead, checked first, so a detached HEAD with no commit is unborn; no branch gives DetachedHead;
/// otherwise (branch, commit).
pub(crate) fn save_head(folder: &Path, head: (Option<String>, Option<Oid>)) -> Result<(String, Oid), SnapshotError> {
    match head {
        (_, None) => Err(SnapshotError::UnbornHead { folder: folder.to_path_buf() }),
        (None, Some(_)) => Err(SnapshotError::DetachedHead { folder: folder.to_path_buf() }),
        (Some(branch), Some(commit)) => Ok((branch, commit)),
    }
}
