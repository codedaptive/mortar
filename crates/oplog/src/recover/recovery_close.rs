//! The record that settles an open intent, or the refusal for one that stopped part way.

use gitio::Oid;

use super::RecoveryDecision;
use crate::store::{close_record, OpPhase, OpRecord, OpResult, OplogError, RefMove};

/// Done closes the intent as done/Moved, Failed as failed/NotMoved naming each planned ref and its actual
/// value in move order, both at `at`; Mixed is OplogError::Mixed naming the intent's op_id and key and each
/// move with its actual value.
pub fn recovery_close(
    intent: &OpRecord,
    decision: RecoveryDecision,
    at: u64,
) -> Result<OpRecord, OplogError> {
    match decision {
        RecoveryDecision::Done => Ok(close_record(intent, OpPhase::Done, OpResult::Moved, at)),
        RecoveryDecision::Failed(refs) => Ok(close_record(intent, OpPhase::Failed, not_moved(refs), at)),
        RecoveryDecision::Mixed(refs) => Err(OplogError::Mixed {
            op_id: intent.op_id.clone(),
            key: intent.key.clone(),
            refs,
        }),
    }
}

/// Each planned ref by name with what it held; the old and new values stay in the record's moves.
fn not_moved(refs: Vec<(RefMove, Option<Oid>)>) -> OpResult {
    OpResult::NotMoved {
        refs: refs.into_iter().map(|(m, actual)| (m.refname, actual)).collect(),
    }
}
