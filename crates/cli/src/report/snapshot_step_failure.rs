//! snapshot_step_failure: the report of mr save, join or resolve failing in a snapshot or save step, any of which may
//! follow an earlier step's move of the snapshot ref. It sits in report/, the lowest level covering save, join and
//! resolve.

use snapshot::SnapshotError;

use super::{failure_report, snapshot_error_detail, with_moved_first, Report};

/// Exit Failure, outcome "failed", next "repeat the same command", under `verb`. The lines come in this order:
/// `moved` when given (moved_before's line for what the verb's steps before the failed one moved; None when nothing
/// did), so what earlier steps moved is read before anything about the failed step; then snapshot_error_detail's
/// line, whose no-movement and no-commit claims speak for the failed step only; then, for a Git, Trailers,
/// ChangeId, ChangeIdMissing, Oplog, BranchMoved or IndexNotSet error, a line saying what the failed step may have
/// left and a line saying that a repeat finishes the verb. IndexNotSet comes after
/// save's commit and apply (the branch at the new commit) or on save's unchanged path (the branch at HEAD): its
/// commit is where the branch stands either way, so its line names it rather than say whether the branch moved.
/// Every other error gets `moved` when given and its detail; with no `moved`, a variant whose detail is snapshot's
/// Display (BadTarget, say) gives the text crate::join::snapshot_failure prints.
///
/// Each "left" line follows the order of crates/snapshot/src/save_with_snapshot.rs. Its snapshot step (snapshot,
/// which is snapshot_formatted with no formatter; a caller with a formatter runs snapshot_formatted before it) is the
/// first that writes (join's snapshot_formatted is that step; its save_from_snapshot takes none after it): it moves
/// the snapshot ref before commit_info, the trailers read, the change ID, the commit and apply.
/// So any of these errors may follow that move. Trailers, ChangeIdMissing and ChangeId come before the save
/// commit is written. A Git error never follows the branch's move: apply's own git errors come back as Oplog, and
/// set_index's after it as IndexNotSet. BranchMoved is the branch's apply refusing, so the branch did not move.
/// Save sends IndexNotSet to save_error_report; join gets it here.
///
/// An Oplog error comes from an apply. On the snapshot paths that is the snapshot ref's apply; in save it is the
/// snapshot ref's or the branch's. It can come before the intent append (reading the log, or a refused ref name:
/// nothing written, nothing moved), in it, or after it (the ref transaction, the read after a stale refusal, the
/// closing record append). An append may have written its record before its sync failed
/// (crates/oplog/src/store/marked/append_marked_record.rs), so a failed intent append may leave the op open with no ref
/// moved, and a failed closing append may leave it closed. An op left open stays so until the oplog recover in
/// lock/locked_act.rs settles it from the refs when the next command takes the join lock;
/// crates/oplog/src/apply/apply_with_values.rs has the order.
///
/// A repeat writes no snapshot when the snapshot ref already names the folder's tree (snapshot_formatted.rs: a tree
/// equal to the last snapshot's writes nothing). A branch the failed run already moved to the folder's tree takes
/// save's unchanged path, so the repeat writes no second commit.
///
/// Per verb:
/// - save (save/run_save.rs) runs snapshot_formatted (only when a formatter is set), then snapshot::save_with_snapshot,
///   so every line can hold as written; join (join/run_join.rs, save_work) runs snapshot_formatted, then
///   snapshot::save_from_snapshot, which writes no snapshot of its own, so the lines hold with the formatter's
///   snapshot as the one that may have moved the ref. Save sends ChangeIdMissing and
///   BranchMoved to save_error_report instead (save/save_step_failure.rs says why); join gets them here.
/// - The pending move's snapshot (join/finish_pending_move.rs), which save, join and resolve run first, and resolve's
///   own snapshot (resolve/run_resolve.rs) are snapshot::snapshot only: snapshot_formatted with no formatter, which
///   writes no save commit. They give Git and Oplog errors here, never Trailers, ChangeIdMissing, ChangeId or
///   BranchMoved. They move no branch, so the Oplog line's "the branch may have moved" is a hedge there: it covers
///   more than those steps can have done.
pub(crate) fn snapshot_step_failure(verb: &str, error: &SnapshotError, moved: Option<&str>) -> Report {
    let left = step_left(error);
    let repeat = left.is_some();
    let mut lines = with_moved_first(moved, vec![snapshot_error_detail(error)]);
    lines.extend(left);
    if repeat {
        lines.push(format!("a repeat finishes the {verb} once the failure above is fixed"));
    }
    Report { lines, ..failure_report(verb, "") }
}

/// What the failed step may have left, for the errors that can follow the snapshot step's move; None for the rest,
/// whose detail says it.
fn step_left(error: &SnapshotError) -> Option<String> {
    let left = match error {
        SnapshotError::Git(_) | SnapshotError::BranchMoved(_) => {
            "the branch did not move; the snapshot ref may have moved"
        }
        SnapshotError::Trailers(_) | SnapshotError::ChangeId(_) | SnapshotError::ChangeIdMissing { .. } => NO_COMMIT,
        SnapshotError::Oplog(_) => {
            "the snapshot ref and the branch may have moved, and an op may be left open; the next command that takes \
             the join lock settles it"
        }
        SnapshotError::IndexNotSet { commit, .. } => {
            return Some(format!("the branch stands at commit {commit}; the snapshot ref may have moved"))
        }
        _ => return None,
    };
    Some(left.to_string())
}

/// What a Trailers, ChangeIdMissing or ChangeId failure left: each comes before save writes its commit, but after its
/// snapshot step, which may have written a snapshot commit on the snapshot ref.
const NO_COMMIT: &str = "no save commit was written and the branch did not move; the snapshot ref may have moved";
