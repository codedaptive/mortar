//! The ops that log the takeover guards an acquire of a lock removed, whether it won the lock or not.

use std::path::Path;

use super::guard_removal_op;
use crate::locks::{AcquiredLock, LockError, LockRecord};
use crate::store::Op;

/// One op per takeover guard `acquired` removed, in order. A won lock names its own path and holder; a
/// refusal names `lock_path` and `me`, the lock and the record this process tried to take it with. A refusal that
/// carries no guards (BadName, BadRecord, HostUnknown, NotOurs), or none removed, gives no ops.
pub(crate) fn guard_removal_ops(
    acquired: &Result<AcquiredLock, LockError>,
    lock_path: &Path,
    me: &LockRecord,
    next_op_id: &mut dyn FnMut() -> String,
    at: u64,
) -> Vec<Op> {
    match acquired {
        Ok(lock) => guard_removal_op(&lock.path, &lock.record, &lock.removed_guards, next_op_id, at),
        Err(e) => guard_removal_op(lock_path, me, e.removed_guards(), next_op_id, at),
    }
}
