//! SnapshotDecision: whether a snapshot writes a commit. A declaration only; decide_snapshot_tests exercises it.

use gitio::Oid;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum SnapshotDecision {
    /// The tree equals the last snapshot's; carries that snapshot's commit, returned as it is.
    Unchanged(Oid),
    /// The tree is new, or there is no snapshot yet: write a commit and move the ref.
    Write,
}
