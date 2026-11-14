//! The detector of the folder move of a request's last rebase, which a kill may have left undone (S-86);
//! join and resolve finish the move, status only reports it (S-85).

use gitio::Oid;
use oplog::{OpPhase, OpRecord};

use super::PendingMove;

/// The last Done record in `records` (log order) holding a move of `request_ref` that changes it decides. Intent
/// and Failed records moved nothing and are skipped; recover has settled open intents before this is read.
/// Some only when that record is a "join-rebase" (the op kind join's rebase and resolve's final join step write), its
/// move has an old tip, and its new tip is `branch_at`, where the request branch stands; then the move from that old
/// tip to that new tip.
/// Any other last mover (a save, a folder move, an undo) or a branch elsewhere or absent is None.
///
/// After the folder move has finished this still answers Some, because move_folder logs no op when the branch
/// already stands at the new tip; finishing it again is a no-op (S-49; git's two-tree move keeps an index already
/// at the new tip, S-87(2)). The next save that moves the branch writes a later op, which ends it.
pub fn find_pending_move(records: &[OpRecord], request_ref: &str, branch_at: Option<&Oid>) -> Option<PendingMove> {
    let (record, last) = records.iter().rev().filter(|record| record.phase == OpPhase::Done).find_map(|record| {
        record
            .moves
            .iter()
            .find(|one| one.refname == request_ref && one.old.as_ref() != Some(&one.new))
            .map(|one| (record, one))
    })?;
    if record.kind != "join-rebase" || branch_at != Some(&last.new) {
        return None;
    }
    Some(PendingMove {
        request_ref: request_ref.to_string(),
        old: last.old.clone()?,
        new: last.new.clone(),
        op_id: record.op_id.clone(),
    })
}
