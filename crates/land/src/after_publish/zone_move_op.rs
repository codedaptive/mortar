//! zone_move_op: the op that moves a zone branch after a publish.
//! No sibling unit test (S-104): every test of it runs gitio's Oid Display in the key and reason (S-101). Covered by
//! tests/after_publish_zit (zone_moves.rs): another_zone_is_replayed_onto_the_new_origin and
//! the_publishing_zone_with_no_later_joins_is_put_on_origin read its kind and move from the log, and
//! a_repeat_moves_nothing counts its records; its unit cases are Nagatha's RITs (owed).

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind `after-publish`, one move of `zone_ref` from `old` to `new`. The key is
/// `after-publish:<ref>:<old>:<new>:<op_id>`: it carries the op id so each attempt is its own op, as squash_op's
/// and integration_op's keys do, because a done record of an earlier move (one later undone, say) must never answer
/// this one. The reason names the ref and both ids.
pub(crate) fn zone_move_op(zone_ref: &str, old: &Oid, new: &Oid, op_id: String, actor: &str, at: u64) -> Op {
    Op {
        // Before op_id, which the next field moves.
        key: format!("after-publish:{zone_ref}:{old}:{new}:{op_id}"),
        op_id,
        kind: "after-publish".to_string(),
        actor: actor.to_string(),
        reason: format!("after-publish moves {zone_ref} from {old} to {new}"),
        at,
        moves: vec![RefMove { refname: zone_ref.to_string(), old: Some(old.clone()), new: new.clone() }],
    }
}
