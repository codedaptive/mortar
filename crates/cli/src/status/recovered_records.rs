//! recovered_records: the op log as oplog's recover would leave it, computed in memory, so status can read a move an
//! open intent left undone without settling it (S-85: status writes nothing and takes no lock).
//! The walk mirrors oplog's crate-private open_intents_in and recoverable_intents (crates/oplog/src/recover/), which
//! cli cannot call: an open intent is an Intent record with no later Done or Failed record of its op_id, and one
//! marked with a remote is left for its caller, as recover leaves it.

use std::collections::HashSet;

use oplog::{MarkedRecord, OpPhase, ValuedRecord};

/// Every record of `records` as a ValuedRecord, in log order; then, for each open intent whose remote is None, in log
/// order, the record `close` gives it, appended after all the records, where recover appends. `close` is called once
/// per such intent and never for a remote-marked one; None leaves that intent open. The first Err from `close` is
/// returned.
pub fn recovered_records<E>(
    records: &[MarkedRecord],
    close: impl Fn(&MarkedRecord) -> Result<Option<ValuedRecord>, E>,
) -> Result<Vec<ValuedRecord>, E> {
    let mut settled: Vec<ValuedRecord> =
        records.iter().map(|m| ValuedRecord { record: m.record.clone(), values: m.values.clone() }).collect();
    for intent in open_intents(records).into_iter().filter(|m| m.remote.is_none()) {
        settled.extend(close(intent)?);
    }
    Ok(settled)
}

/// Each Intent record with no later Done or Failed record of the same op_id, in log order.
fn open_intents(records: &[MarkedRecord]) -> Vec<&MarkedRecord> {
    // Walking backwards, `closed` holds the op_ids closed later in the log than the record at hand.
    let mut closed: HashSet<&str> = HashSet::new();
    let mut open = Vec::new();
    for m in records.iter().rev() {
        match m.record.phase {
            OpPhase::Intent if !closed.contains(m.record.op_id.as_str()) => open.push(m),
            OpPhase::Intent => {}
            OpPhase::Done | OpPhase::Failed => {
                closed.insert(m.record.op_id.as_str());
            }
        }
    }
    open.reverse();
    open
}
