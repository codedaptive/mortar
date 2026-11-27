//! op_base: the request base a join op recorded before and after the join, as the folders crate's RecordedBase, so
//! the cli can finish a folder record's rewrite after a kill (finish_pending_move, follow_answer) or put it back on an
//! undo (undo's folder_step). Tested by op_base_tests.rs.

use ::join::{recorded_base, RecordedBase as JoinRecorded};
use folders::RecordedBase;
use oplog::ValuedRecord;

use super::recorded_base_of;

/// join's recorded_base on `op`, each base converted with recorded_base_of: Recorded is (before, after); NotRecorded
/// (another kind, or an op written before join recorded the base) is None; Unreadable is join's reason.
pub(crate) fn op_base(op: &ValuedRecord) -> Result<Option<(RecordedBase, RecordedBase)>, String> {
    match recorded_base(op) {
        JoinRecorded::Recorded { before, after } => Ok(Some((recorded_base_of(&before), recorded_base_of(&after)))),
        JoinRecorded::NotRecorded => Ok(None),
        JoinRecorded::Unreadable { reason } => Err(reason),
    }
}
