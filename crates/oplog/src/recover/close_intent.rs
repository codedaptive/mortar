//! Closes one open intent for the caller that knows its outcome: publish closes its push intent,
//! which recover leaves open.
//! No unit test: it reads and appends to the op log; crates/oplog/tests/oplog/remote_close_record.rs
//! covers the record and the refusal on a seeded log. crates/oplog/tests/oplog/remote_close.rs covers, in a
//! throwaway repository: Done then Failed on separate ops each write one closing record that reads back with
//! its mark; an unknown and an already closed op_id are refused with nothing appended.

use std::path::Path;

use super::{close_intent_record, IntentClose};
use crate::store::{append_marked_record, read_marked_log, unix_now, MarkedRecord, OplogError};

/// Appends the closing record of the open intent `op_id` once and returns it, with the intent's values and
/// remote mark. An op_id with no open intent is OplogError::NotOpen and appends nothing. The caller holds
/// its zone's join lock or the landing lock.
pub fn close_intent(common: &Path, op_id: &str, close: IntentClose) -> Result<MarkedRecord, OplogError> {
    let closing = close_intent_record(&read_marked_log(common)?.records, op_id, close, unix_now())?;
    append_marked_record(common, &closing.record, &closing.values, closing.remote.as_deref())?;
    Ok(closing)
}
