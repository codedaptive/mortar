//! SavedWithSnapshot: what a save_with_snapshot or save_from_snapshot call left behind. A declaration only;
//! saved_with_snapshot_from_tests builds it.

use crate::SaveOutcome;

/// What save_with_snapshot and save_from_snapshot answer: save's outcome, and whether the call's own snapshot step
/// moved the snapshot ref.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedWithSnapshot {
    /// What save returns for the same call.
    pub outcome: SaveOutcome,
    /// Whether save_with_snapshot's snapshot step wrote a snapshot commit and moved the target's snapshot ref
    /// (false: the tree equalled the last snapshot's and nothing was written). The snapshot step runs, and can move
    /// the ref, on both of save's Ok paths, including the one that writes no save commit. save_from_snapshot takes
    /// no snapshot, so it is always false there, though the caller's own snapshot call may have moved the ref.
    pub snapshot_written: bool,
}
