//! backup_op: the op a backup push runs inside, so MORTAR_OP rides on the push (SPEC-land 09, S-60).
//! backup_op_tests cover it.

use oplog::Op;

/// Kind `land-backup`, no moves: the push changes only remote refs, which the op log does not track. The key is
/// `land-backup:<host>:<op_id>`: it carries the op id so each backup run is its own op, because an earlier done
/// backup must never answer a later one. The reason names the host's backup namespace.
pub(crate) fn backup_op(host: &str, op_id: String, actor: &str, at: u64) -> Op {
    Op {
        // Before op_id, which the next field moves.
        key: format!("land-backup:{host}:{op_id}"),
        op_id,
        kind: "land-backup".to_string(),
        actor: actor.to_string(),
        reason: format!("backup pushes in-flight refs to refs/mortar-backup/{host}/ on the remote"),
        at,
        moves: Vec::new(),
    }
}
