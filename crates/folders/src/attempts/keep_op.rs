//! The oplog op that moves the request branch to the kept attempt's tip and deletes every attempt branch.

use gitio::{GitError, Oid};
use oplog::{Op, RefMove};

use super::Kept;

/// Kind "folder-keep", key "folders:keep:<request_ref>@<kept tip hex>", reason "keep <kept tip hex> on
/// <request_ref>". Its first move takes the full ref `request_ref` from `request_tip` to `kept.tip`; then each of
/// `kept.delete` (refname, oid), in order, is deleted from that oid: new is all zeros the oid's length, which
/// deletes the ref (oplog RefMove). Every old is Some, so the op moves nothing unless every ref is still where it
/// was read. The key holds the kept tip, so a repeat of this keep after the op was logged is answered Repeated by oplog::apply. The caller
/// supplies `op_id` and `at` (unix seconds), so this stays pure. The Err is Oid::parse's refusal of the zeros,
/// which zeros of an Oid's own length (40 or 64) never get.
pub fn keep_op(
    op_id: String,
    actor: String,
    at: u64,
    request_ref: &str,
    request_tip: &Oid,
    kept: &Kept,
) -> Result<Op, GitError> {
    let mut moves = vec![RefMove {
        refname: request_ref.to_string(),
        old: Some(request_tip.clone()),
        new: kept.tip.clone(),
    }];
    for (refname, oid) in &kept.delete {
        moves.push(RefMove {
            refname: refname.clone(),
            old: Some(oid.clone()),
            new: Oid::parse(&"0".repeat(oid.as_str().len()))?,
        });
    }
    Ok(Op {
        op_id,
        key: format!("folders:keep:{request_ref}@{}", kept.tip.as_str()),
        kind: "folder-keep".to_string(),
        actor,
        reason: format!("keep {} on {request_ref}", kept.tip.as_str()),
        at,
        moves,
    })
}
