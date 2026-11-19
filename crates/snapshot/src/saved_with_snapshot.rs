//! SavedWithSnapshot: what a save_with_snapshot call left behind. A declaration only;
//! saved_with_snapshot_from_tests builds it.

use crate::SaveOutcome;

/// What save_with_snapshot answers: save's outcome, and whether save's own snapshot step moved the snapshot ref.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SavedWithSnapshot {
    /// What save returns for the same call.
    pub outcome: SaveOutcome,
    /// Whether save's snapshot step wrote a snapshot commit and moved the target's snapshot ref (false: the tree
    /// equalled the last snapshot's and nothing was written). The snapshot step runs, and can move the ref, on both
    /// of save's Ok paths, including the one that writes no save commit.
    pub snapshot_written: bool,
}
