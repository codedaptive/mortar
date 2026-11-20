//! saved_with_snapshot_from: the answer save_snapshot_onto gives on either Ok path, for save_with_snapshot and
//! save_from_snapshot.

use crate::{SaveOutcome, SavedWithSnapshot, SnapshotOutcome};
use gitio::Oid;
use oplog::ChangeId;

/// Save's outcome for `commit` (the commit written, or HEAD as found when `written` is false) with `snap`'s tree and
/// refused list, and `snap.written` as `snapshot_written` (save_from_snapshot passes a `snap` with `written` false).
/// `snap.commit` and `snap.formatted` are not part of it.
pub(crate) fn saved_with_snapshot_from(
    snap: SnapshotOutcome,
    commit: Oid,
    written: bool,
    change_id: Option<ChangeId>,
) -> SavedWithSnapshot {
    SavedWithSnapshot {
        outcome: SaveOutcome { commit, tree: snap.tree, written, change_id, refused: snap.refused },
        snapshot_written: snap.written,
    }
}
