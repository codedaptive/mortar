//! snapshot_moved: whether the snapshot ref moved by the end of a save, counting save's own snapshot step (join's
//! save takes none, so for join the answer is `earlier`).
//! Shell API: run_join and run_save call it; snapshot_moved_tests covers it.

use snapshot::SavedWithSnapshot;

/// True when a step before the save moved the snapshot ref (`earlier`: the pending move's snapshot or the
/// formatter's) or save's own snapshot step did (`saved.snapshot_written`; always false for join), whether or not the
/// save wrote a commit.
pub(crate) fn snapshot_moved(earlier: bool, saved: &SavedWithSnapshot) -> bool {
    earlier || saved.snapshot_written
}
