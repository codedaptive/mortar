//! withdraw_op: the op that deletes a zone's squash ref.
//! No sibling unit test: every test of it runs gitio's Oid Display, so its unit cases are repo integration tests
//! (RITs). Covered by tests/withdraw_zit through withdraw:
//! a_withdraw_deletes_the_squash_ref_and_answers_its_commit_and_change_id (one land-withdraw record, the ref gone)
//! and squash_withdraw_squash_draws_a_new_change_id.

use gitio::{GitError, Oid};
use oplog::{Op, RefMove};

/// Kind `land-withdraw`, one move of `squash_ref` from `old` to the all-zero id of `old`'s length, which deletes
/// it. The key is `land-withdraw:<ref>:<old>:<op_id>`: it carries the op id so each withdraw attempt is its own op,
/// because an earlier done op for the same move (one later undone, say) must never answer this one. The reason
/// names the ref and the commit. Building the zero id can fail only as Oid::parse's GitError.
pub(crate) fn withdraw_op(squash_ref: &str, old: &Oid, op_id: String, actor: &str, at: u64) -> Result<Op, GitError> {
    let zero = Oid::parse(&"0".repeat(old.as_str().len()))?;
    Ok(Op {
        // Before op_id, which the next field moves.
        key: format!("land-withdraw:{squash_ref}:{old}:{op_id}"),
        op_id,
        kind: "land-withdraw".to_string(),
        actor: actor.to_string(),
        reason: format!("withdraw deletes {squash_ref} at {old}"),
        at,
        moves: vec![RefMove { refname: squash_ref.to_string(), old: Some(old.clone()), new: zero }],
    })
}
