//! The op an undo reverses.

use crate::store::{OpPhase, OpRecord};

/// The index in `records` of the last done record with a move of any ref named in `scope`, matched by whole
/// name. Intent and failed records do not count. An undo's own done record counts, so undoing twice redoes.
pub fn find_undo_target(records: &[OpRecord], scope: &[String]) -> Option<usize> {
    records.iter().rposition(|r| {
        r.phase == OpPhase::Done && r.moves.iter().any(|m| scope.contains(&m.refname))
    })
}
