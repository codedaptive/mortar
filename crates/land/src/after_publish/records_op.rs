//! records_op: the op that moves only a zone's replay records, for a publishing zone that is already current.
//! records_op_tests covers it.

use oplog::{Op, RefMove};

/// Kind `after-publish`, the same kind as zone_move_op's, moving `records` and nothing else (the zone ref stays).
/// The key is `after-publish-records:<zone_ref>:<op_id>`: it carries the op id so each attempt is its own op, as
/// zone_move_op's key does. The reason names the zone ref.
pub(crate) fn records_op(zone_ref: &str, records: Vec<RefMove>, op_id: String, actor: &str, at: u64) -> Op {
    Op {
        // Before op_id, which the next field moves.
        key: format!("after-publish-records:{zone_ref}:{op_id}"),
        op_id,
        kind: "after-publish".to_string(),
        actor: actor.to_string(),
        reason: format!("after-publish deletes replay records of {zone_ref}"),
        at,
        moves: records,
    }
}
