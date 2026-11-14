//! The detector of the folder move of a request's last rebase, which a kill may have left undone (S-86);
//! join and resolve finish the move; status only reports it, and only while the folder's index is not at the new tip
//! (read_stalled_move, S-85, S-97).

use ::join::FOLDER_TREE;
use gitio::Oid;
use oplog::{OpPhase, ValuedRecord};

use super::PendingMove;

/// The last Done record in `records` (log order) holding a move of `request_ref` that changes it decides. Intent
/// and Failed records moved nothing and are skipped; recover has settled open intents before this is read.
/// Some only when that record is a "join-rebase" (the op kind join's rebase and resolve's final join step write), its
/// move has an old tip, and its new tip is `branch_at`, where the request branch stands; then the move from that old
/// tip to that new tip, with folder_tree the tree in that record's first value pair named FOLDER_TREE. No such pair,
/// or a value that is not an Oid, is folder_tree None: then move_folder makes the plain move, which git refuses
/// rather than overwrite anything, so nothing is lost (S-87(4)). The values of any other record are never read.
/// Any other last mover (a save, a folder move, an undo) or a branch elsewhere or absent is None.
///
/// After the folder move has finished this still answers Some, because move_folder logs no op when the branch
/// already stands at the new tip; finishing it again is a no-op (S-49; git's two-tree move keeps an index already
/// at the new tip, S-87(2)). The next save that moves the branch writes a later op, which ends it.
pub fn find_pending_move(records: &[ValuedRecord], request_ref: &str, branch_at: Option<&Oid>) -> Option<PendingMove> {
    let (valued, last) = records.iter().rev().filter(|valued| valued.record.phase == OpPhase::Done).find_map(|valued| {
        valued
            .record
            .moves
            .iter()
            .find(|one| one.refname == request_ref && one.old.as_ref() != Some(&one.new))
            .map(|one| (valued, one))
    })?;
    let record = &valued.record;
    if record.kind != "join-rebase" || branch_at != Some(&last.new) {
        return None;
    }
    Some(PendingMove {
        request_ref: request_ref.to_string(),
        old: last.old.clone()?,
        new: last.new.clone(),
        op_id: record.op_id.clone(),
        folder_tree: valued
            .values
            .iter()
            .find(|(name, _)| name == FOLDER_TREE)
            .and_then(|(_, hex)| Oid::parse(hex).ok()),
    })
}
