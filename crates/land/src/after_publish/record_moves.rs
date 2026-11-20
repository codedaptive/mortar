//! record_moves: the replay record moves one zone-move op carries. record_moves_tests covers it.

use oplog::RefMove;

/// `deletes` (published_records: the records at or behind the published source), then `writes` (replay_records: the
/// records a clean replay writes and carries), each in its own order. A delete whose ref a write also moves is left
/// out: the write wins, because the replay just made that record name the zone commit standing for its source, and
/// git refuses two moves of one ref in one update.
pub(crate) fn record_moves(deletes: Vec<RefMove>, writes: Vec<RefMove>) -> Vec<RefMove> {
    let mut moves: Vec<RefMove> =
        deletes.into_iter().filter(|delete| !writes.iter().any(|write| write.refname == delete.refname)).collect();
    moves.extend(writes);
    moves
}
