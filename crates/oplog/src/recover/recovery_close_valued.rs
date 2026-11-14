//! The record that settles an open intent read with its values, the values kept.

use super::{recovery_close, RecoveryDecision};
use crate::store::{OplogError, ValuedRecord};

/// recovery_close on the intent's record, carrying the intent's values into the closing record in their
/// order; Mixed is recovery_close's OplogError::Mixed.
pub fn recovery_close_valued(
    intent: &ValuedRecord,
    decision: RecoveryDecision,
    at: u64,
) -> Result<ValuedRecord, OplogError> {
    Ok(ValuedRecord {
        record: recovery_close(&intent.record, decision, at)?,
        values: intent.values.clone(),
    })
}
