//! The base a joined partner request stands on, read back from join's "join-zone-request" op's moves: pending_base's
//! fallback for an op written before join recorded the base in its values, so a join killed between that op and the
//! folder record's rewrite can still finish the rewrite (finish_pending_move).
//! Tested by joined_zone_base_tests.rs.

use gitio::Oid;
use oplog::ValuedRecord;

use super::find_pending_move;
use crate::undo_reason::done_record;

/// The zone's tip before the join, the commit the request's own commits were replayed onto, when the request ref's last mover is a
/// "join-zone-request": find_pending_move's choice of last mover, so Some only when that move has an old tip and its
/// new tip is `branch_at`. The op moves the zone ref and the request ref (zone_request_op); the base is the old tip
/// of its move of a ref other than `request_ref`. The op leaves the zone move out when the zone already stood at the
/// request's new tip, so with no such move the base is that new tip. Any other last mover is None.
pub(crate) fn joined_zone_base(records: &[ValuedRecord], request_ref: &str, branch_at: Option<&Oid>) -> Option<Oid> {
    let pending = find_pending_move(records, request_ref, branch_at)?;
    let record = &done_record(records, &pending.op_id)?.record;
    if record.kind != "join-zone-request" {
        return None;
    }
    match record.moves.iter().find(|one| one.refname != request_ref) {
        Some(zone_move) => zone_move.old.clone(),
        None => Some(pending.new),
    }
}
