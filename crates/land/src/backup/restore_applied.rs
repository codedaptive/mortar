//! restore_applied: what restore_backup makes of an apply's answer. restore_applied_tests cover it.

use crate::backup::BackupError;
use oplog::{ApplyOutcome, OplogError};

/// Done is success. Repeated is refused as OpRepeated with the earlier done record: each restore op's key carries
/// a fresh op id, so a done record with that key means apply moved nothing for this attempt, and answering
/// Created would be false. Failed (a local ref was no longer absent, nothing moved) is refused as OpFailed, never
/// retried; an apply error is Oplog.
pub(crate) fn restore_applied(outcome: Result<ApplyOutcome, OplogError>) -> Result<(), BackupError> {
    match outcome.map_err(BackupError::Oplog)? {
        ApplyOutcome::Done(_) => Ok(()),
        ApplyOutcome::Repeated(record) => Err(BackupError::OpRepeated(record)),
        ApplyOutcome::Failed(record) => Err(BackupError::OpFailed(record)),
    }
}
