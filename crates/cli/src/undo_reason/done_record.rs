//! done_record: the Done record of the op an undo reversed, found by the op id reversed_op reads back from the undo's
//! reason, so its kind and values can be read.

use oplog::{OpPhase, ValuedRecord};

/// The last record in `records` (log order) with phase Done and op id `op_id`; None when there is none. Intent and
/// Failed records of that id are skipped.
pub fn done_record<'a>(records: &'a [ValuedRecord], op_id: &str) -> Option<&'a ValuedRecord> {
    records.iter().rev().find(|valued| valued.record.phase == OpPhase::Done && valued.record.op_id == op_id)
}
