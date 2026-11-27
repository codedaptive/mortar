//! pending_base: the base the folder record must hold once a pending folder move is finished, read from the base the
//! join op recorded (op_base), so finish_pending_move can finish a rewrite a kill left, or put the record back after
//! an undo of a join rebase, a partner one included. Tested by pending_base_tests.rs.

use folders::RecordedBase;
use oplog::ValuedRecord;

use super::{joined_zone_base, MoveCause, PendingMove};
use crate::request_base::op_base;
use crate::undo_reason::{done_record, reversed_op};

/// For cause Rebase, the base after the join that the Done record of `pending`'s op recorded (a plain or partner
/// rebase, a joined partner request, a resolve's final step). A join-zone-request that recorded none falls back to
/// joined_zone_base with no partners; any other op that recorded none is None, and the record is left as it is.
/// For cause Undo, the base before the join that the Done record of the op the undo reversed recorded; an undo whose
/// reason names no op, a reversed op missing from `records`, or one that recorded no base is None.
/// A recorded value that does not decode is the error, naming the op.
pub(crate) fn pending_base(records: &[ValuedRecord], pending: &PendingMove) -> Result<Option<RecordedBase>, String> {
    let Some(done) = done_record(records, &pending.op_id) else {
        return Ok(None);
    };
    match pending.cause {
        MoveCause::Rebase => match recorded(done)? {
            Some((_, after)) => Ok(Some(after)),
            None if done.record.kind == "join-zone-request" => {
                Ok(joined_zone_base(records, &pending.request_ref, Some(&pending.new))
                    .map(|base| RecordedBase { base, partners: Vec::new() }))
            }
            None => Ok(None),
        },
        MoveCause::Undo => {
            let Some(reversed) = reversed_op(&done.record.reason).and_then(|(_, op_id)| done_record(records, &op_id))
            else {
                return Ok(None);
            };
            Ok(recorded(reversed)?.map(|(before, _)| before))
        }
    }
}

/// op_base of `op`, its error naming the op.
fn recorded(op: &ValuedRecord) -> Result<Option<(RecordedBase, RecordedBase)>, String> {
    op_base(op).map_err(|reason| format!("the base op {} recorded cannot be read: {reason}", op.record.op_id))
}
