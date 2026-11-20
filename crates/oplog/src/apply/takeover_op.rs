//! The op that logs taking over a stale lock.

use crate::locks::AcquiredLock;
use crate::store::Op;

/// None when the lock was not taken over. Otherwise an op with no moves, kind `lock-takeover`, actor the
/// new holder as `<pid>@<host>`, and key `lock-takeover:<lock path>:<old pid>@<old host>:<old since>`, so
/// a repeat of the same takeover is the same op. The reason names the lock path and the old holder's pid,
/// host and key.
pub fn takeover_op(lock: &AcquiredLock, op_id: String, at: u64) -> Option<Op> {
    let old = lock.took_over.as_ref()?;
    let path = lock.path.display();
    Some(Op {
        op_id,
        key: format!("lock-takeover:{path}:{}@{}:{}", old.pid, old.host, old.since),
        kind: "lock-takeover".to_string(),
        actor: format!("{}@{}", lock.record.pid, lock.record.host),
        reason: format!(
            "took over the stale lock {path} from pid {} on host {}, held for key {}",
            old.pid, old.host, old.key
        ),
        at,
        moves: Vec::new(),
    })
}
