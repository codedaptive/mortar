//! The record that closes an intent once its ref transaction has answered.

use gitio::{Oid, UpdateRefsOutcome};

use crate::store::{close_record, OpPhase, OpRecord, OpResult};

/// Updated: done, Moved. Stale: failed, naming the refused ref and `actual`, the value it held (None:
/// absent). `actual` is read only for Stale.
pub fn close_after_update(
    intent: &OpRecord,
    outcome: &UpdateRefsOutcome,
    actual: Option<Oid>,
    at: u64,
) -> OpRecord {
    match outcome {
        UpdateRefsOutcome::Updated => close_record(intent, OpPhase::Done, OpResult::Moved, at),
        UpdateRefsOutcome::Stale { refname } => close_record(
            intent,
            OpPhase::Failed,
            OpResult::Stale {
                refname: refname.clone(),
                actual,
            },
            at,
        ),
    }
}
