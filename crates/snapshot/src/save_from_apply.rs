//! save_from_apply: whether save's branch move went through, from oplog apply's answer.

use crate::SnapshotError;
use oplog::ApplyOutcome;

/// Done: the branch moved. Repeated: OpRepeated, since apply found a done record for the key and moved nothing.
/// Failed: BranchMoved, the branch was not at the HEAD commit read. Each error carries apply's record.
pub(crate) fn save_from_apply(outcome: ApplyOutcome) -> Result<(), SnapshotError> {
    match outcome {
        ApplyOutcome::Done(_) => Ok(()),
        ApplyOutcome::Repeated(record) => Err(SnapshotError::OpRepeated(record)),
        ApplyOutcome::Failed(record) => Err(SnapshotError::BranchMoved(record)),
    }
}
