//! save_step_failure: the report of mr save failing in a snapshot or save step: an error from the pending move's
//! snapshot (finish_pending_move), or the SnapshotError inside SaveHooksError::Snapshot from join's
//! save_from_formatted (snapshot_formatted, then snapshot::save_from_snapshot_with_hooks: save with a formatter) or
//! snapshot::save_with_hooks (save without one); a hook's refusal goes to hook_refusal_report instead, in run_save.
//! Its Git, Trailers, ChangeId and Oplog errors are snapshot_step_failure's; every other error is save_error_report's,
//! which mr status shares.

use snapshot::SnapshotError;

use super::save_error_report;
use crate::report::{snapshot_step_failure, with_moved_first, Report};

/// `moved` is moved_before's line for what save's steps before the failed one moved (None when nothing did).
/// snapshot_step_failure under "save", with `moved`, for a Git, Trailers, ChangeId or Oplog error (its doc says why
/// each line holds for save's steps, and `moved` comes first there too); save_error_report for every other error,
/// with `moved` as its first line when given, before any line about the failed step, on a refusal, a busy, an
/// already-done and a failure alike. ChangeIdMissing and BranchMoved stay with
/// save_error_report although snapshot_step_failure handles them: save's report of ChangeIdMissing is a refusal
/// (exit 3) naming HEAD as the owner and the mend (ask the zone's coordinator to mend HEAD's trailers), since a
/// repeat cannot finish a save until HEAD's trailers are mended, and of BranchMoved a busy (exit 4), since another
/// writer moved the branch; snapshot_step_failure's "repeat the same command" would hide either.
pub(crate) fn save_step_failure(error: &SnapshotError, moved: Option<&str>) -> Report {
    match error {
        SnapshotError::Git(_) | SnapshotError::Trailers(_) | SnapshotError::ChangeId(_) | SnapshotError::Oplog(_) => {
            snapshot_step_failure("save", error, moved)
        }
        _ => {
            let report = save_error_report(error);
            Report { lines: with_moved_first(moved, report.lines), ..report }
        }
    }
}
