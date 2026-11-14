//! squash_op: the op that moves a zone's squash ref to a new squash commit.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind `land-squash`, one move of `squash_ref` from `old` (None: the ref must not exist yet) to `new`. The key
/// is `land-squash:<ref>:<old or none>:<new>:<op_id>`: it carries the op id so each squash attempt is its own op,
/// because an earlier done op for the same move (one later undone, say) must never answer this one. The reason
/// names the ref and both ids.
pub(crate) fn squash_op(squash_ref: &str, old: Option<&Oid>, new: &Oid, op_id: String, actor: &str, at: u64) -> Op {
    let old_key = old.map_or("none", Oid::as_str);
    let old_reason = old.map_or("nothing", Oid::as_str);
    Op {
        // Before op_id, which the next field moves.
        key: format!("land-squash:{squash_ref}:{old_key}:{new}:{op_id}"),
        op_id,
        kind: "land-squash".to_string(),
        actor: actor.to_string(),
        reason: format!("squash moves {squash_ref} from {old_reason} to {new}"),
        at,
        moves: vec![RefMove { refname: squash_ref.to_string(), old: old.cloned(), new: new.clone() }],
    }
}
