//! The ops that log removing stale takeover guards, one per removed guard.

use std::path::Path;

use crate::locks::{takeover_guard_path, LockRecord};
use crate::store::Op;

/// One op per guard in `removed_guards`, in order, each with its op ID from one call of `next_op_id`; an empty
/// list gives no ops. The caller passes `(&lock.path, &lock.record, &lock.removed_guards)` after a successful
/// acquire, or `(the lock's path, me, err.removed_guards())` after a refusal, and logs each op.
/// Each op has no moves, kind `takeover-guard-removal`, actor `me` as `<pid>@<host>`, and key
/// `takeover-guard-removal:<guard path>:<old pid>@<old host>:<old since>`, where the guard path is
/// `takeover_guard_path(lock_path)` and "old" is the removed guard's holder, so a repeat of the same removal
/// gives the same key. The reason names the guard path and the old holder's pid, host and key.
pub(crate) fn guard_removal_op(
    lock_path: &Path,
    me: &LockRecord,
    removed_guards: &[LockRecord],
    next_op_id: &mut dyn FnMut() -> String,
    at: u64,
) -> Vec<Op> {
    let guard = takeover_guard_path(lock_path);
    let path = guard.display();
    removed_guards
        .iter()
        .map(|old| Op {
            op_id: next_op_id(),
            key: format!("takeover-guard-removal:{path}:{}@{}:{}", old.pid, old.host, old.since),
            kind: "takeover-guard-removal".to_string(),
            actor: format!("{}@{}", me.pid, me.host),
            reason: format!(
                "removed the stale takeover guard {path} left by pid {} on host {}, held for key {}",
                old.pid, old.host, old.key
            ),
            at,
            moves: Vec::new(),
        })
        .collect()
}
