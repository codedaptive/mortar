//! save_op: the op log entry that moves a folder's branch to a save commit.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind and reason "save"; one move of `branch` from `head` (the commit HEAD was read at) to `commit`, so oplog
/// apply refuses the move if the branch is no longer at `head`. The key is `save <branch> <op_id>`; with a fresh op
/// id it is unique per move.
pub(crate) fn save_op(branch: &str, head: &Oid, commit: &Oid, actor: &str, op_id: String, at: u64) -> Op {
    Op {
        key: format!("save {branch} {op_id}"),
        op_id,
        kind: "save".to_string(),
        actor: actor.to_string(),
        reason: "save".to_string(),
        at,
        moves: vec![RefMove { refname: branch.to_string(), old: Some(head.clone()), new: commit.clone() }],
    }
}
