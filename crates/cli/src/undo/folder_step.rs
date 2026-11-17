//! folder_step: the folder step that follows an undo: SetIndex sets the index to the branch's tree; MoveFolder
//! finishes the pending folder move; RestoreJoin puts the paused join back, then sets the index to the branch's tree
//! (S-73, S-143, S-148, S-155).
//! No unit test (S-104): it runs git, snapshot or the op log; its decisions have their tests.

use ::join::{restore_join_state, RestoredJoin};
use oplog::ValuedRecord;

use super::restore_failed_report;
use super::set_index_to_branch::set_index_to_branch;
use super::UndoStep;
use crate::join::{finish_pending_move, RequestRun};
use crate::report::Report;

const VERB: &str = "undo";

/// SetIndex and MoveFolder make their step and give None. RestoreJoin writes the paused join `target` (the undone
/// resolve's Done record) recorded back into the folder, then sets the index to the branch's tree, the files kept,
/// and gives what the restore answered. The restore runs first so that a refused restore leaves the index as it
/// was; its refusal is restore_failed_report naming `target`'s op and `undo_op` (exit 1, next mr undo, S-155).
pub(super) fn folder_step(
    run: &RequestRun,
    step: UndoStep,
    target: &ValuedRecord,
    undo_op: &str,
) -> Result<Option<RestoredJoin>, Report> {
    match step {
        UndoStep::SetIndex => set_index_to_branch(run).map(|()| None),
        UndoStep::MoveFolder => finish_pending_move(run, VERB).map(|_| None),
        UndoStep::RestoreJoin => {
            let restored = restore_join_state(&run.git, &run.folder.toplevel, target)
                .map_err(|error| restore_failed_report(&error, &target.record.op_id, undo_op))?;
            set_index_to_branch(run)?;
            Ok(Some(restored))
        }
    }
}
