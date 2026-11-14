//! restore_op: the one op that creates the local refs a restore_backup fetched. restore_op_tests cover it.

use gitio::Oid;
use oplog::{Op, RefMove};

/// Kind `land-restore-backup`, one move per `created` (local refname, backup id), in order, each from absent
/// (None: the ref must not exist yet) to its id. The key is `land-restore-backup:<host>:<op_id>`: it carries the op
/// id so each restore attempt is its own op, because an earlier done restore (one later undone, say) must never
/// answer this one. The reason names the host and how many refs the op creates.
pub(crate) fn restore_op(host: &str, created: &[(String, Oid)], op_id: String, actor: &str, at: u64) -> Op {
    Op {
        // Before op_id, which the next field moves.
        key: format!("land-restore-backup:{host}:{op_id}"),
        op_id,
        kind: "land-restore-backup".to_string(),
        actor: actor.to_string(),
        reason: match created.len() {
            1 => format!("restore_backup creates 1 local ref from host {host}'s backup"),
            n => format!("restore_backup creates {n} local refs from host {host}'s backup"),
        },
        at,
        moves: created
            .iter()
            .map(|(refname, oid)| RefMove { refname: refname.clone(), old: None, new: oid.clone() })
            .collect(),
    }
}
