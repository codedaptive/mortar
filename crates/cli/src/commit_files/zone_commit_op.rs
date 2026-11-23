//! zone_commit_op: the op that moves a zone branch from its tip to the commit `mr commit-files` wrote.
//! zone_commit_op_tests covers it.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind "commit-files", one move of refs/heads/zone/<zone> from `tip` to `commit`, a reason naming both, and the
/// given key, actor, op id and time.
pub fn zone_commit_op(zone: &str, tip: &Oid, commit: &Oid, key: &str, actor: &str, op_id: String, at: u64) -> Op {
    let refname = format!("refs/heads/zone/{zone}");
    Op {
        op_id,
        key: key.to_string(),
        kind: "commit-files".to_string(),
        actor: actor.to_string(),
        reason: format!("commit-files moves {refname} from {tip} to {commit}"),
        at,
        moves: vec![RefMove { refname, old: Some(tip.clone()), new: commit.clone() }],
    }
}
