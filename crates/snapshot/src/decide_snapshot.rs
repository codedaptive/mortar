//! decide_snapshot: whether a folder's tree needs a new snapshot commit.

use crate::previous_snapshot::PreviousSnapshot;
use crate::snapshot_decision::SnapshotDecision;
use gitio::Oid;

/// Unchanged with the previous commit when its tree equals `tree`, so a repeat writes nothing; Write when the
/// trees differ or there is no previous snapshot.
pub(crate) fn decide_snapshot(tree: &Oid, previous: Option<&PreviousSnapshot>) -> SnapshotDecision {
    match previous {
        Some(previous) if previous.tree == *tree => SnapshotDecision::Unchanged(previous.commit.clone()),
        _ => SnapshotDecision::Write,
    }
}
