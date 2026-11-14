//! The oplog op that deletes a removed folder's branch and, when it has one, its snapshot ref.

use gitio::{GitError, Oid};
use oplog::{Op, RefMove};

/// Kind "folder-remove", key "folders:remove:<branch_ref>@<tip hex>": the full ref `branch_ref` deleted from `tip`,
/// then, when `snap` is Some((ref, oid)), that snapshot ref deleted from its oid. Each move's old is Some, so the
/// op moves nothing unless every ref is still where it was read; its new is all zeros the length of `tip`, which
/// deletes the ref (oplog RefMove). The key holds the tip, so a repeat of the same remove is answered Repeated by
/// oplog::apply. The caller supplies `op_id` and `at` (unix seconds), so this stays pure. The Err is Oid::parse's
/// refusal of the zeros, which zeros of an Oid's own length (40 or 64) never get.
pub fn remove_op(
    op_id: String,
    actor: String,
    at: u64,
    branch_ref: &str,
    tip: &Oid,
    snap: Option<(&str, &Oid)>,
) -> Result<Op, GitError> {
    let zero = Oid::parse(&"0".repeat(tip.as_str().len()))?;
    let mut moves = vec![RefMove {
        refname: branch_ref.to_string(),
        old: Some(tip.clone()),
        new: zero.clone(),
    }];
    if let Some((refname, oid)) = snap {
        moves.push(RefMove {
            refname: refname.to_string(),
            old: Some(oid.clone()),
            new: zero,
        });
    }
    Ok(Op {
        op_id,
        key: format!("folders:remove:{branch_ref}@{}", tip.as_str()),
        kind: "folder-remove".to_string(),
        actor,
        reason: format!("remove folder branch {branch_ref}"),
        at,
        moves,
    })
}
