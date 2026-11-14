//! The oplog op that moves a folder's branch from the move's old tip to its new tip.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind "folder-move", key "folders:move:<branch_ref>@<old hex>..<new hex>", one move of the full ref `branch_ref`
/// from `old` to `new` (old Some: the ref must be at `old`). The key holds both tips, so a repeat of the same move
/// is answered Repeated by oplog::apply. The caller supplies `op_id` and `at` (unix seconds), so this stays pure.
pub fn move_op(op_id: String, actor: String, at: u64, branch_ref: &str, old: &Oid, new: &Oid) -> Op {
    Op {
        op_id,
        key: format!("folders:move:{branch_ref}@{}..{}", old.as_str(), new.as_str()),
        kind: "folder-move".to_string(),
        actor,
        reason: format!("move folder branch {branch_ref} to {}", new.as_str()),
        at,
        moves: vec![RefMove {
            refname: branch_ref.to_string(),
            old: Some(old.clone()),
            new: new.clone(),
        }],
    }
}
