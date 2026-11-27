//! folder_step: the folder step that follows an undo: SetIndex sets the index to the branch's tree; MoveFolder
//! finishes the pending folder move, which puts the folder record's base back first (finish_pending_move);
//! RestoreJoin puts the folder record back on the base the undone resolve's op recorded before it, puts the paused
//! join back, then sets the index to the branch's tree.
//! No unit test: it runs git, snapshot or the op log; op_base_tests covers reading the recorded base, and the
//! integration tests crates/cli/tests/across_kits/undo_save_join_resolve/{base_undo,resolve_undo}.rs cover the steps.

use ::join::{restore_join_state, RestoredJoin};
use oplog::ValuedRecord;

use super::restore_failed_report;
use super::set_index_to_branch::set_index_to_branch;
use super::UndoStep;
use crate::join::{finish_pending_move, rewrite_record_to, snapshot_failure, RequestRun};
use crate::report::{failure_report, Report};
use crate::request_base::op_base;

const VERB: &str = "undo";

/// SetIndex and MoveFolder make their step and give None. RestoreJoin rewrites the folder record to the base `target`
/// (the undone resolve's Done record) recorded before the resolve, the base of the paused join (rewrite_record_to;
/// none when the op recorded none), writes the paused join `target` recorded back into the folder, then sets the
/// index to the branch's tree, the files kept, and gives what the restore answered. A recorded base that cannot be
/// read is failure_report naming `target`'s op, the paused join not put back. The restore runs before the index is
/// set so that a refused restore leaves the index as it was; its refusal is restore_failed_report naming `target`'s
/// op and `undo_op` (exit 1, next mr undo).
pub(super) fn folder_step(
    run: &RequestRun,
    step: UndoStep,
    target: &ValuedRecord,
    undo_op: &str,
) -> Result<Option<RestoredJoin>, Report> {
    match step {
        UndoStep::SetIndex => set_index_to_branch(run).map(|()| None),
        UndoStep::MoveFolder => finish_pending_move(run, VERB, &|error| snapshot_failure(VERB, error)).map(|_| None),
        UndoStep::RestoreJoin => {
            let op_id = &target.record.op_id;
            match op_base(target) {
                Ok(Some((before, _))) => rewrite_record_to(run, VERB, Ok(Some(before)), None, None)?,
                Ok(None) => {}
                Err(why) => {
                    let what =
                        format!("the base op {op_id} recorded cannot be read; the paused join was not put back: {why}");
                    return Err(failure_report(VERB, &what));
                }
            }
            let restored = restore_join_state(&run.git, &run.folder.toplevel, target)
                .map_err(|error| restore_failed_report(&error, op_id, undo_op))?;
            set_index_to_branch(run)?;
            Ok(Some(restored))
        }
    }
}
