//! save_snapshot_onto: everything a save does after its snapshot is in hand: build the message, run the caller's hook
//! step on it, commit the snapshot's tree onto the commit it was built on, move the branch through the op log and set
//! the folder's index. save_through and save_from_snapshot_through both end here; the hookless saves pass a step that
//! keeps the message, and the hooked twins pass run_save_hooks.
//! No unit test: it runs git and appends to the op log, and holds no decision of its own; save_unchanged,
//! save_change_id, save_trailers, save_op, save_from_apply, saved_with_snapshot_from and real_index_git are its
//! decisions and have their tests. The integration cases are listed in save_with_snapshot.rs (through save),
//! save_from_snapshot.rs (tests/snapshot/save_from_snapshot_cases.rs), and, for the hooked twins, save_with_hooks.rs
//! and save_from_snapshot_with_hooks.rs.

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
/// branch is not at `built_on`, and the index is set to the saved tree. `snapshot_written` is `snap.written`.
///
/// `hooks` runs on the written shape only, after the message (`subject` with the Mortar trailers) is built and
/// before the commit is written: it gets the snapshot's tree, that message and the trailer pairs, and answers the
/// message to commit. Its error is returned as it is, with no commit written, no branch moved, no save op logged
/// and the index as it was. The unchanged shape calls it not at all. Errors are save_with_snapshot's from after its
/// snapshot step, then `hooks`' own.
pub(crate) fn save_snapshot_onto<E: From<SnapshotError>>(
    git: &Git,
    target: &SnapshotTarget,
    subject: &str,
    head: (String, Oid),
    built_on: Oid,
    snap: SnapshotOutcome,
    hooks: impl FnOnce(&Git, &Oid, String, &[(String, String)]) -> Result<String, E>,
) -> Result<SavedWithSnapshot, E> {
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
    let pairs = save_trailers(&change_id, target);
    let message = git.with_trailers(subject, &pairs).map_err(SnapshotError::Git)?;
    let message = hooks(git, &snap.tree, message, &pairs)?;
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
