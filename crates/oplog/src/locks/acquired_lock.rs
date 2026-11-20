//! A lock this process now holds, the stale holder it replaced, and every stale takeover guard it removed.
//! A bare type with no behaviour of its own, so no unit test; acquire_lock and take_over_stale_lock build it.

use std::path::PathBuf;

use super::LockRecord;

/// A held lock. `took_over` is the previous holder of a stale lock, for the caller to log as an op.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquiredLock {
    pub path: PathBuf,
    pub record: LockRecord,
    pub took_over: Option<LockRecord>,
    /// The holders of every stale `<lock>.takeover` guard (dead pid on this host) this acquire removed, in order,
    /// across all its attempts, for the caller to log one op each; apply::guard_removal_op builds them.
    pub removed_guards: Vec<LockRecord>,
}
