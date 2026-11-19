//! The detector of the folder move of a request's last mover, which a kill or a refused move may have left undone:
//! a join rebase's or an undo's that put a plain join rebase back. An undo of a resolve's final join
//! step is none: undo puts the paused join back and sets the index itself, keeping the folder's files.
//! join, resolve, save and undo finish the move; status only reports it, and only while the folder's index
//! is not at the new tip (read_stalled_move).
//! Tested by find_pending_move_tests.rs, including an undo whose reversed op's Done record decides (a resolve step,
//! one missing from the log or one that only failed).

use ::join::FOLDER_TREE;
use gitio::Oid;
use oplog::{OpPhase, ValuedRecord};

use super::{MoveCause, PendingMove};
use crate::undo_reason::{done_record, is_resolve_step, reversed_op};

/// The last Done record in `records` (log order) holding a move of `request_ref` that changes it decides. Intent
/// and Failed records moved nothing and are skipped; recover has settled open intents before this is read, and status,
/// which runs no recover, gives it the log settled in memory by recovered_records, writing nothing.
/// Some only when that move has an old tip and its new tip is `branch_at`, where the request branch stands, and the
/// record is one of two kinds:
/// - a "join-rebase" (the op kind join's rebase and resolve's final join step write): the move from that old tip to
///   that new tip, cause Rebase, with folder_tree the tree in that record's first value pair named FOLDER_TREE. No
///   such pair, or a value that is not an Oid, is folder_tree None: then move_folder makes the plain move, which git
///   refuses rather than overwrite anything, so nothing is lost;
/// - an "undo" whose reason reversed_op reads as an undo of a "join-rebase" op that was not a resolve step: the move
///   from the rebased tip back to the tip before the rebase, cause Undo, folder_tree None. The reversed op is a
///   resolve step when its done_record is_resolve_step; then the undo is None, since undo itself put the paused join
///   back and set the index to the branch's tree, keeping the folder's files. A reversed op with no Done record in
///   `records` counts as a plain rebase.
///
/// The values of an undo, and of any record other than these two kinds and the reversed op, are never read. Any
/// other last mover (a save, a folder move, an undo of another kind or with a reason it cannot read) or a branch
/// elsewhere or absent is None.
///
/// After the folder move has finished this still answers Some, because move_folder logs no op when the branch
/// already stands at the new tip; finishing it again is a no-op (git's two-tree move keeps an index already
/// at the new tip). The next save that moves the branch writes a later op, which ends it.
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
        "undo" if undoes_plain_rebase(records, &record.reason) => (MoveCause::Undo, None),
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

/// Whether `reason` reads as an undo of a "join-rebase" op whose Done record in `records`, if any, is no resolve step.
fn undoes_plain_rebase(records: &[ValuedRecord], reason: &str) -> bool {
    reversed_op(reason).is_some_and(|(kind, op_id)| {
        kind == "join-rebase" && !done_record(records, &op_id).is_some_and(is_resolve_step)
    })
}
