//! The open intents recover settles: every open intent not marked with a remote.
//! No unit test: its decisions run through open_intents_in (S-102).
//! crates/oplog/tests/oplog/recoverable.rs proves them through recover on a seeded log.

use super::open_intents_in;
use crate::store::{MarkedRecord, ValuedRecord};

/// Each open intent (no later Done or Failed record of its op_id) whose remote mark is None, with its
/// values, in log order. An intent marked with a remote (open_remote_intent) is left out: its caller
/// closes it with close_intent.
pub(crate) fn recoverable_intents(records: &[MarkedRecord]) -> Vec<ValuedRecord> {
    open_intents_in(records, |m| &m.record)
        .into_iter()
        .filter(|m| m.remote.is_none())
        .map(|m| ValuedRecord { record: m.record.clone(), values: m.values.clone() })
        .collect()
}
