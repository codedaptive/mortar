//! recovered_records: the op log as oplog's recover would leave it, computed in memory, so status can read a move an
//! open intent left undone without settling it (S-85: status writes nothing and takes no lock).
//! The open intents come from oplog's exported walk, open_marked_intents (S-167): an Intent record with no later Done
//! or Failed record of its op_id, remote-marked ones included; this filters those out, as recover leaves them for
//! their caller. recovered_records_tests is a ZUT (S-171): the walk it runs is oplog's, exported, pure and downward.

use oplog::{open_marked_intents, MarkedRecord, ValuedRecord};

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
    for intent in open_marked_intents(records).into_iter().filter(|m| m.remote.is_none()) {
        settled.extend(close(intent)?);
    }
    Ok(settled)
}
