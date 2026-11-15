//! record_moves: the replay record moves one zone-move op carries. record_moves_tests covers it.

use oplog::RefMove;

/// `consumed` (the consumed record's delete), then `carried` (a replay record the replay moved), skipping None.
/// When both name one ref, only `carried`: a queued squash still names that source, so its record is still needed,
/// and git refuses two moves of one ref in one update.
pub(crate) fn record_moves(consumed: Option<RefMove>, carried: Option<RefMove>) -> Vec<RefMove> {
    match (consumed, carried) {
        (Some(consumed), Some(carried)) if consumed.refname == carried.refname => vec![carried],
        (consumed, carried) => consumed.into_iter().chain(carried).collect(),
    }
}
