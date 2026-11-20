//! save_snapshot_onto: everything a save does after its snapshot is in hand: commit the snapshot's tree onto the
//! commit it was built on, move the branch through the op log and set the folder's index. save_with_snapshot and
//! save_from_snapshot both end here.
//! No unit test: it runs git and appends to the op log, and holds no decision of its own; save_unchanged,
//! save_change_id, save_trailers, save_op, save_from_apply, saved_with_snapshot_from and real_index_git are its
//! decisions and have their tests. The integration cases are listed in save_with_snapshot.rs (through save) and
//! save_from_snapshot.rs (tests/snapshot/save_from_snapshot_cases.rs).

use crate::real_index_git::real_index_git;
use crate::save_change_id::save_change_id;
use crate::save_from_apply::save_from_apply;
use crate::save_op::save_op;
use crate::save_trailers::save_trailers;
use crate::save_unchanged::save_unchanged;
use crate::saved_with_snapshot_from::saved_with_snapshot_from;
use crate::{SavedWithSnapshot, SnapshotError, SnapshotOutcome, SnapshotTarget};
use gitio::{Git, Oid};
use oplog::{apply, new_change_id, new_op_id, read_mortar_trailers, unix_now};

/// Saves `snap`'s tree as a commit on `head`'s branch with `subject` and the Mortar trailers. `git` is rooted at the
/// folder (`git.repo`). `head` is save_head's (branch, HEAD as read now); `built_on` is the commit `snap` was taken
/// on, and is the commit's parent, the source of the change ID and the op's old value. A tree equal to `built_on`'s
/// with HEAD still at `built_on` writes no commit and sets the folder's index to that tree; otherwise the commit is
/// written and the branch moves from `built_on` through one op log entry, which refuses with BranchMoved when the
/// branch is not at `built_on`, and the index is set to the saved tree. `snapshot_written` is `snap.written`. Errors are
/// save_with_snapshot's from after its snapshot step.
pub(crate) fn save_snapshot_onto(
    git: &Git,
    target: &SnapshotTarget,
    subject: &str,
    head: (String, Oid),
    built_on: Oid,
    snap: SnapshotOutcome,
) -> Result<SavedWithSnapshot, SnapshotError> {
    let folder = &git.repo;
    let (branch, head_now) = head;
    let info = git.commit_info(&built_on).map_err(SnapshotError::Git)?;
    if let Some(index_tree) = save_unchanged(&snap.tree, &info.tree, head_now != built_on) {
        real_index_git(git)
            .set_index(folder, &index_tree)
            .map_err(|error| SnapshotError::IndexNotSet { commit: built_on.clone(), error })?;
        return Ok(saved_with_snapshot_from(snap, built_on, false, None));
    }
    let trailers = read_mortar_trailers(git, &info.message).map_err(SnapshotError::Trailers)?;
    let change_id = match save_change_id(&built_on, &trailers, target)? {
        Some(id) => id,
        None => new_change_id().map_err(SnapshotError::ChangeId)?,
    };
    let message = git
        .with_trailers(subject, &save_trailers(&change_id, target))
        .map_err(SnapshotError::Git)?;
    let commit = git
        .commit_tree(&snap.tree, std::slice::from_ref(&built_on), &message, None, None)
        .map_err(SnapshotError::Git)?;
    let common = git.common_dir().map_err(SnapshotError::Git)?;
    let op = save_op(&branch, &built_on, &commit, &target.actor, new_op_id(), unix_now());
    save_from_apply(apply(git, &common, &op).map_err(SnapshotError::Oplog)?)?;
    real_index_git(git)
        .set_index(folder, &snap.tree)
        .map_err(|error| SnapshotError::IndexNotSet { commit: commit.clone(), error })?;
    Ok(saved_with_snapshot_from(snap, commit, true, Some(change_id)))
}
