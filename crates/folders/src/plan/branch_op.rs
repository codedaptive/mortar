//! The oplog op that makes a folder's branch at a commit.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind "folder-branch", key "folders:branch:<branch>@<commit hex>", one move creating
/// `refs/heads/<branch>` at `commit` (old None: the ref must not exist yet). The key holds the
/// commit, so a repeat at the same commit is answered Repeated by oplog::apply. The caller supplies
/// `op_id` and `at` (unix seconds), so this stays pure.
pub fn branch_op(op_id: String, actor: String, at: u64, branch: &str, commit: &Oid) -> Op {
    Op {
        op_id,
        key: format!("folders:branch:{branch}@{}", commit.as_str()),
        kind: "folder-branch".to_string(),
        actor,
        reason: format!("make folder branch {branch}"),
        at,
        moves: vec![RefMove {
            refname: format!("refs/heads/{branch}"),
            old: None,
            new: commit.clone(),
        }],
    }
}
