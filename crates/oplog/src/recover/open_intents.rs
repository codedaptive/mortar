//! The intents in a log that no record closes.

use std::collections::HashSet;

use crate::store::{OpPhase, OpRecord};

/// Each Intent record whose op_id has no later Done or Failed record, in log order.
pub fn open_intents(records: &[OpRecord]) -> Vec<&OpRecord> {
    // Walking backwards, `closed` holds the op_ids closed later in the log than the record at hand.
    let mut closed: HashSet<&str> = HashSet::new();
    let mut open = Vec::new();
    for r in records.iter().rev() {
        match r.phase {
            OpPhase::Intent if !closed.contains(r.op_id.as_str()) => open.push(r),
            OpPhase::Intent => {}
            OpPhase::Done | OpPhase::Failed => {
                closed.insert(r.op_id.as_str());
            }
        }
    }
    open.reverse();
    open
}
