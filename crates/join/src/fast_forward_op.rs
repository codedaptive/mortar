//! fast_forward_op: the op that moves a zone ref forward to a request tip. fast_forward_op_tests covers
//! every field and a key unique to each op id.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind `join-fast-forward`, one move of `zone_ref` from `from` to `to`. The key is
/// `join-fast-forward:<zone_ref>:<from>:<to>:<op_id>`: it carries the op id so each join attempt is its own
/// op, because an earlier done op for the same move (one later undone, say) must never answer this one. A
/// repeated join is safe because join reads the refs again under its lock after recover. The reason names
/// the ref and both ids.
pub(crate) fn fast_forward_op(zone_ref: &str, from: &Oid, to: &Oid, op_id: String, actor: &str, at: u64) -> Op {
    Op {
        // Before op_id, which the next field moves.
        key: format!("join-fast-forward:{zone_ref}:{from}:{to}:{op_id}"),
        op_id,
        kind: "join-fast-forward".to_string(),
        actor: actor.to_string(),
        reason: format!("join fast-forwards {zone_ref} from {from} to {to}"),
        at,
        moves: vec![RefMove { refname: zone_ref.to_string(), old: Some(from.clone()), new: to.clone() }],
    }
}
