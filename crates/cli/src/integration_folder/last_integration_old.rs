//! last_integration_old: where the integration branch stood before the last op that moved it (S-137), the folder's
//! tip a stopped run would have left behind.

use gitio::Oid;
use oplog::{OpPhase, OpRecord};

/// The last Done record in `records` (log order) holding a move of `integration_ref` that changes it (old is not
/// the new tip) decides: its move's old, or None when that move has no old. Intent and Failed records moved nothing
/// and are skipped, as are moves of other refs and moves that leave the ref where it was. No such record is None.
/// Both Nones mean the same to the caller: no stopped folder move to finish.
pub fn last_integration_old(records: &[OpRecord], integration_ref: &str) -> Option<Oid> {
    records
        .iter()
        .rev()
        .filter(|record| record.phase == OpPhase::Done)
        .find_map(|record| {
            record.moves.iter().find(|one| one.refname == integration_ref && one.old.as_ref() != Some(&one.new))
        })?
        .old
        .clone()
}
