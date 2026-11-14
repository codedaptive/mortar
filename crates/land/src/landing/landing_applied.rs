//! landing_applied: what land, back_out and publish make of an apply's answer. landing_applied_tests covers it.

use crate::landing::LandError;
use oplog::{ApplyOutcome, OplogError};

/// Done is success. Repeated is refused as OpRepeated with the earlier done record: each integration op's key
/// carries a fresh op id, so a done record with that key means apply moved nothing for this attempt, and answering
/// a move would be false. Failed (the ref was not at its expected value, nothing moved) is refused as OpFailed,
/// never retried; an apply error is Oplog.
pub(crate) fn landing_applied(outcome: Result<ApplyOutcome, OplogError>) -> Result<(), LandError> {
    match outcome.map_err(LandError::Oplog)? {
        ApplyOutcome::Done(_) => Ok(()),
        ApplyOutcome::Repeated(record) => Err(LandError::OpRepeated(record)),
        ApplyOutcome::Failed(record) => Err(LandError::OpFailed(record)),
    }
}
