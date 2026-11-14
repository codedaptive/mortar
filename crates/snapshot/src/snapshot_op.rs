//! snapshot_op: the op log entry that moves a request's snapshot ref.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind and reason "snapshot"; one move of `refname` from `previous` (None: the ref must not exist yet) to `commit`.
/// The key is `snapshot <refname> <op_id>`, so with a fresh op id oplog apply finds no done record for it and runs
/// the ref transaction: a ref swept or moved back is moved again, even to a commit built before. A snapshot's repeat-safety comes from
/// decide_snapshot (an unchanged tree writes nothing), not from this key; the key only has to be unique per move.
pub(crate) fn snapshot_op(
    refname: &str,
    previous: Option<&Oid>,
    commit: &Oid,
    actor: &str,
    op_id: String,
    at: u64,
) -> Op {
    Op {
        key: format!("snapshot {refname} {op_id}"),
        op_id,
        kind: "snapshot".to_string(),
        actor: actor.to_string(),
        reason: "snapshot".to_string(),
        at,
        moves: vec![RefMove { refname: refname.to_string(), old: previous.cloned(), new: commit.clone() }],
    }
}
