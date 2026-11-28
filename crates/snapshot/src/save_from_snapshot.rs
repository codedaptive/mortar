//! save_from_snapshot: save_with_snapshot for a caller that has already taken the folder's snapshot, so the folder
//! is not snapshotted a second time under the same lock. It runs no commit hooks; save_from_snapshot_with_hooks is
//! its twin that does. Its body is save_from_snapshot_through with a hook step that keeps the message: it reads the
//! sentence and HEAD (save_message, save_head) and hands the passed snapshot to save_snapshot_onto.
//! No unit test: it runs git; its one decision, snapshot_written false (made in save_from_snapshot_through), is
//! proved by the changed case below, and save_message, save_head and save_snapshot_onto's decisions have their tests.
//! The integration tests in tests/snapshot/save_from_snapshot_cases.rs, in a throwaway repository, cover:
//! - a changed folder is saved with no second snapshot, parent `built_on`, snapshot_written false and the snapshot
//!   ref left where the caller's snapshot put it: changed_snapshot_is_saved_without_another_snapshot;
//! - a folder unchanged from `built_on` writes no commit and leaves the branch there:
//!   unchanged_snapshot_leaves_the_branch_at_built_on;
//! - a branch moved after the snapshot gives BranchMoved and leaves the foreign commit and the files alone:
//!   moved_branch_refuses_changed_snapshot_and_preserves_foreign_commit_and_files;
//! - the same on the unchanged shape, with the index not reset:
//!   moved_branch_refuses_unchanged_snapshot_without_resetting_index.
//!
//! The hooks_ case in that file runs save_from_snapshot_with_hooks and is listed in save_from_snapshot_with_hooks.rs.

use crate::save_from_snapshot_through::save_from_snapshot_through;
use crate::{SavedWithSnapshot, SnapshotError, SnapshotOutcome, SnapshotTarget};
use gitio::{Git, Oid};
use std::path::Path;

/// Saves `folder`'s files from `snap` as save_with_snapshot does, with `snap` in place of its snapshot step: no
/// second snapshot is taken. `built_on` is the HEAD the snapshot's tree was built on, which the caller reads after
/// snapshot_formatted (or snapshot) returns, under the same lock; it is the commit's parent and the source of the
/// change ID. `git` gives the binary and environment; every run is in `folder`, which must be the folder's top.
///
/// `snapshot_written` is always false: the caller's own snapshot call reported whether its ref moved. When HEAD is
/// no longer at `built_on`, the branch's op apply refuses with BranchMoved, with the commit written and the branch
/// not moved, and this holds on the unchanged shape too, where the index is never set. A branch switched to another
/// at the same commit is saved onto, as save_with_snapshot does. No commit hooks run.
///
/// Errors: BadSentence before anything is read or written; UnbornHead and DetachedHead from HEAD as read now; then
/// save_with_snapshot's errors after its snapshot step. The caller holds the lock and runs oplog::recover.
pub fn save_from_snapshot(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    built_on: &Oid,
    snap: SnapshotOutcome,
) -> Result<SavedWithSnapshot, SnapshotError> {
    save_from_snapshot_through(git, folder, target, sentence, built_on, snap, |_, _, message, _| Ok(message))
}
