//! The items of a log whose intent no record closes, for any item that holds an op record.

use std::collections::HashSet;

use crate::store::{OpPhase, OpRecord};

/// Each item whose record is an Intent with no later Done or Failed record of the same op_id, in log
/// order; `record` reads an item's op record.
pub(crate) fn open_intents_in<R>(records: &[R], record: fn(&R) -> &OpRecord) -> Vec<&R> {
    // Walking backwards, `closed` holds the op_ids closed later in the log than the record at hand.
    let mut closed: HashSet<&str> = HashSet::new();
    let mut open = Vec::new();
    for item in records.iter().rev() {
        let r = record(item);
        match r.phase {
            OpPhase::Intent if !closed.contains(r.op_id.as_str()) => open.push(item),
            OpPhase::Intent => {}
            OpPhase::Done | OpPhase::Failed => {
                closed.insert(r.op_id.as_str());
            }
        }
    }
    open.reverse();
    open
}
