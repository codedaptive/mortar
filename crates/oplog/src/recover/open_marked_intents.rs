//! The open intents in a marked log, with their values and remote marks: what cli's status reads without
//! settling them (S-85).

use super::open_intents_in;
use crate::store::MarkedRecord;

/// Each Intent record whose op_id has no later Done or Failed record, in log order, as its MarkedRecord.
/// Intents marked with a remote are included: the caller filters on `remote`.
pub fn open_marked_intents(records: &[MarkedRecord]) -> Vec<&MarkedRecord> {
    open_intents_in(records, |m| &m.record)
}
