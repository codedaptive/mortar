//! The request folder's move a killed `mr folder keep` left undone: its op moved the request branch and deleted
//! the try branches, then the process died before the request folder moved (S-49, S-86).

use gitio::Oid;
use oplog::{OpPhase, OpRecord};

use crate::report::is_zero_oid;

/// The last Done record in `records` (log order) holding any move of `request_ref` decides; Intent and Failed
/// records moved nothing and are skipped. A keep whose kept tip equals the request tip moves the request ref from a
/// tip to the same tip, and that move still counts. Some((old, new)) of that move only when the same record also
/// deletes `kept_branch` (the full name refs/heads/try/<zone>/<request>/<k>) from `new` (its old is `new`, its new
/// all zeros), the move has an old tip, and the request branch stands at `new` (`branch_at`): only keep of attempt
/// k writes that shape. Anything else, a later mover of the request ref among them, is None.
pub fn pending_keep_move(
    records: &[OpRecord],
    request_ref: &str,
    kept_branch: &str,
    branch_at: Option<&Oid>,
) -> Option<(Oid, Oid)> {
    let (record, request_move) = records.iter().rev().filter(|record| record.phase == OpPhase::Done).find_map(|record| {
        record.moves.iter().find(|one| one.refname == request_ref).map(|one| (record, one))
    })?;
    let new = &request_move.new;
    let deletes_kept = record
        .moves
        .iter()
        .any(|one| one.refname == kept_branch && one.old.as_ref() == Some(new) && is_zero_oid(&one.new));
    if !deletes_kept || branch_at != Some(new) {
        return None;
    }
    Some((request_move.old.clone()?, new.clone()))
}
