//! The intents in a log that no record closes.

use super::open_intents_in;
use crate::store::OpRecord;

/// Each Intent record whose op_id has no later Done or Failed record, in log order.
pub fn open_intents(records: &[OpRecord]) -> Vec<&OpRecord> {
    open_intents_in(records, |r| r)
}
