//! op_applied: what join makes of an apply's answer. op_applied_tests covers each outcome.

use crate::JoinRunError;
use oplog::{ApplyOutcome, OplogError};

/// Done is success. Repeated is refused as OpRepeated with the earlier done record: each join op's key carries its
/// op id, so a done record with that key means apply moved nothing for this attempt, and answering a move would be
/// false. Failed (a ref was not at its expected value, nothing moved) is refused as OpFailed, never retried; an
/// apply error, a protected-ref refusal included, is Oplog.
pub(crate) fn op_applied(outcome: Result<ApplyOutcome, OplogError>) -> Result<(), JoinRunError> {
    match outcome.map_err(JoinRunError::Oplog)? {
        ApplyOutcome::Done(_) => Ok(()),
        ApplyOutcome::Repeated(record) => Err(JoinRunError::OpRepeated(record)),
        ApplyOutcome::Failed(record) => Err(JoinRunError::OpFailed(record)),
    }
}
