//! integration_op: the op that moves one ref for a landing, its back-out, a commit_files commit or a publish.
//! No sibling unit test: every test of it runs gitio's Oid Display and oplog's ChangeId Display in the key and
//! reason. Covered by tests/landing_zit (landing.rs,
//! a_squash_on_origin_fast_forwards_integration_to_the_squash: the land op's kind and move); its unit
//! cases are repo integration tests (RITs): tests/land_rit/owed_ops.rs covers them.

use gitio::Oid;
use oplog::{ChangeId, Op, RefMove};

/// Kind `kind` (`land`, `land-back-out`, `land-commit-files`, `publish-push`, `publish-fetched`, `publish-origin` or
/// `publish-squash`), one move of `refname` from `old` to `new`. The key is `<kind>:<ref>:<old>:<new>:<op_id>`: it
/// carries the op id so each attempt is its own op, because a landing backed out and landed again must never be
/// answered by the first one's done record. publish_push_op keeps this op's move and reason and replaces the key with
/// its own, which has no op id. The reason names the change ID, the ref and both ids.
// Eight arguments: each is one field of the op or of its key and reason, and squash_op's shape is kept.
#[allow(clippy::too_many_arguments)]
pub(crate) fn integration_op(
    kind: &str,
    refname: &str,
    old: &Oid,
    new: &Oid,
    change_id: &ChangeId,
    op_id: String,
    actor: &str,
    at: u64,
) -> Op {
    Op {
        // Before op_id, which the next field moves.
        key: format!("{kind}:{refname}:{old}:{new}:{op_id}"),
        op_id,
        kind: kind.to_string(),
        actor: actor.to_string(),
        reason: format!("{kind} {change_id} moves {refname} from {old} to {new}"),
        at,
        moves: vec![RefMove { refname: refname.to_string(), old: Some(old.clone()), new: new.clone() }],
    }
}
