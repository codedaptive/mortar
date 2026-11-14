//! rebase_op: the op that moves a request ref to its commits replayed onto the zone. rebase_op_tests covers
//! every field and a key unique to each op id.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind `join-rebase`, one move of `request_ref` from `from` to `to`. The key is
/// `join-rebase:<request_ref>:<from>:<onto>:<op_id>`: it carries the op id so each join attempt is its own
/// op, because an earlier done op for the same move (one later undone, say) must never answer this one. A
/// repeated join is safe because join reads the refs again under its lock after recover. The reason names
/// the ref, both ids and `onto`.
pub(crate) fn rebase_op(
    request_ref: &str,
    from: &Oid,
    to: &Oid,
    onto: &Oid,
    op_id: String,
    actor: &str,
    at: u64,
) -> Op {
    Op {
        // Before op_id, which the next field moves.
        key: format!("join-rebase:{request_ref}:{from}:{onto}:{op_id}"),
        op_id,
        kind: "join-rebase".to_string(),
        actor: actor.to_string(),
        reason: format!("join rebases {request_ref} from {from} to {to}, replayed onto {onto}"),
        at,
        moves: vec![RefMove { refname: request_ref.to_string(), old: Some(from.clone()), new: to.clone() }],
    }
}
