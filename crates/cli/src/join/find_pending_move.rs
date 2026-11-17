//! The detector of the folder move of a request's last mover, which a kill or a refused move may have left undone:
//! a join rebase's (S-86) or an undo's that put a rebased branch back (S-143). join, resolve, save and undo finish
//! the move; status only reports it, and only while the folder's index is not at the new tip (read_stalled_move,
//! S-85, S-97).

use ::join::FOLDER_TREE;
use gitio::Oid;
use oplog::{OpPhase, ValuedRecord};

use super::{MoveCause, PendingMove};
use crate::undo_reason::reversed_op;

/// The last Done record in `records` (log order) holding a move of `request_ref` that changes it decides. Intent
/// and Failed records moved nothing and are skipped; recover has settled open intents before this is read.
/// Some only when that move has an old tip and its new tip is `branch_at`, where the request branch stands, and the
/// record is one of two kinds:
/// - a "join-rebase" (the op kind join's rebase and resolve's final join step write): the move from that old tip to
///   that new tip, cause Rebase, with folder_tree the tree in that record's first value pair named FOLDER_TREE. No
///   such pair, or a value that is not an Oid, is folder_tree None: then move_folder makes the plain move, which git
///   refuses rather than overwrite anything, so nothing is lost (S-87(4));
/// - an "undo" whose reason reversed_op reads as an undo of a "join-rebase" op: the move from the rebased tip back
///   to the tip before the rebase, cause Undo, folder_tree None.
///
/// The values of any other record, and of an undo, are never read. Any other last mover (a save, a folder move, an
/// undo of another kind or with a reason it cannot read) or a branch elsewhere or absent is None.
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
    if branch_at != Some(&last.new) {
        return None;
    }
    let (cause, folder_tree) = match record.kind.as_str() {
        "join-rebase" => (
            MoveCause::Rebase,
            valued
                .values
                .iter()
                .find(|(name, _)| name == FOLDER_TREE)
                .and_then(|(_, hex)| Oid::parse(hex).ok()),
        ),
        "undo" if reversed_op(&record.reason).is_some_and(|(kind, _)| kind == "join-rebase") => (MoveCause::Undo, None),
        _ => return None,
    };
    Some(PendingMove {
        request_ref: request_ref.to_string(),
        old: last.old.clone()?,
        new: last.new.clone(),
        op_id: record.op_id.clone(),
        folder_tree,
        cause,
    })
}
