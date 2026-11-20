//! The open intents in a marked log, with their values and remote marks: for cli's status to read without
//! settling them.
//! No unit test: its decisions run through open_intents_in.
//! crates/oplog/tests/oplog/open_marked_intents.rs proves them on marked logs built from MarkedRecord values.

use super::open_intents_in;
use crate::store::MarkedRecord;

/// Each Intent record whose op_id has no later Done or Failed record, in log order, as its MarkedRecord.
/// Intents marked with a remote are included: the caller filters on `remote`.
pub fn open_marked_intents(records: &[MarkedRecord]) -> Vec<&MarkedRecord> {
    open_intents_in(records, |m| &m.record)
}
