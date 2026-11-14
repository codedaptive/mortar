//! snapshot_from_apply: what a snapshot returns once oplog apply has run its ref move.

use crate::{RefusedFile, SnapshotError, SnapshotOutcome};
use gitio::Oid;
use oplog::ApplyOutcome;

/// Done: the ref now names `commit`, written true, carrying `refused`. Repeated: OpRepeated, since apply found a
/// done record for the key and moved nothing, so the ref may not name `commit`. Failed: RefMoved, the ref was not
/// at the value read. Each error carries apply's record.
pub(crate) fn snapshot_from_apply(
    outcome: ApplyOutcome,
    commit: Oid,
    tree: Oid,
    refused: Vec<RefusedFile>,
) -> Result<SnapshotOutcome, SnapshotError> {
    match outcome {
        ApplyOutcome::Done(_) => Ok(SnapshotOutcome { commit, tree, written: true, refused }),
        ApplyOutcome::Repeated(record) => Err(SnapshotError::OpRepeated(record)),
        ApplyOutcome::Failed(record) => Err(SnapshotError::RefMoved(record)),
    }
}
