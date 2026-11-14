//! squash_applied: what squash makes of an apply's answer.

use crate::squash::SquashError;
use oplog::{ApplyOutcome, OplogError};

/// Done is success. Repeated is refused as OpRepeated with the earlier done record: each squash op's key carries
/// a fresh op id, so a done record with that key means apply moved nothing for this attempt, and answering a move
/// would be false. Failed (the squash ref was not at its expected value, nothing moved) is refused as OpFailed,
/// never retried; an apply error is Oplog.
pub(crate) fn squash_applied(outcome: Result<ApplyOutcome, OplogError>) -> Result<(), SquashError> {
    match outcome.map_err(SquashError::Oplog)? {
        ApplyOutcome::Done(_) => Ok(()),
        ApplyOutcome::Repeated(record) => Err(SquashError::OpRepeated(record)),
        ApplyOutcome::Failed(record) => Err(SquashError::OpFailed(record)),
    }
}
