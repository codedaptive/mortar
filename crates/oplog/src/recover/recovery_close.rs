//! The record that settles an open intent, or the refusal for one that stopped part way.

use super::RecoveryDecision;
use crate::store::{close_record, OpPhase, OpRecord, OpResult, OplogError};

/// Done closes the intent as done/Moved, Failed as failed/NotMoved, both at `at`; Mixed is
/// OplogError::Mixed naming the intent's op_id and key and each move with its actual value.
pub fn recovery_close(
    intent: &OpRecord,
    decision: RecoveryDecision,
    at: u64,
) -> Result<OpRecord, OplogError> {
    match decision {
        RecoveryDecision::Done => Ok(close_record(intent, OpPhase::Done, OpResult::Moved, at)),
        RecoveryDecision::Failed => Ok(close_record(intent, OpPhase::Failed, OpResult::NotMoved, at)),
        RecoveryDecision::Mixed(refs) => Err(OplogError::Mixed {
            op_id: intent.op_id.clone(),
            key: intent.key.clone(),
            refs,
        }),
    }
}
