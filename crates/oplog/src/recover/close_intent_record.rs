//! The closing record close_intent appends for one open intent.
//! No unit test: its decisions run through open_intents_in and close_record (S-102).
//! crates/oplog/tests/oplog/remote_close_record.rs proves them through close_intent on a seeded log.

use super::{open_intents_in, IntentClose};
use crate::store::{close_record, MarkedRecord, OpPhase, OpResult, OplogError};

/// close_record on the last open intent with `op_id` (marked or not), keeping its values and remote mark:
/// Done gives a done record with result Moved, Failed { refs } a failed record with result
/// NotMoved { refs }. No open intent with that op_id (never written, or closed by a later done or failed
/// record) is OplogError::NotOpen naming it.
pub(crate) fn close_intent_record(
    records: &[MarkedRecord],
    op_id: &str,
    close: IntentClose,
    at: u64,
) -> Result<MarkedRecord, OplogError> {
    let intent = open_intents_in(records, |m| &m.record)
        .into_iter()
        .rev()
        .find(|m| m.record.op_id == op_id)
        .ok_or_else(|| OplogError::NotOpen { op_id: op_id.to_string() })?;
    let (phase, result) = match close {
        IntentClose::Done => (OpPhase::Done, OpResult::Moved),
        IntentClose::Failed { refs } => (OpPhase::Failed, OpResult::NotMoved { refs }),
    };
    Ok(MarkedRecord {
        record: close_record(&intent.record, phase, result, at),
        values: intent.values.clone(),
        remote: intent.remote.clone(),
    })
}
