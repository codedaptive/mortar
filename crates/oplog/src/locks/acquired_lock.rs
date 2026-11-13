//! A lock this process now holds, and the stale holder it replaced, if any.
//! A bare type with no behaviour of its own, so no unit test; acquire_lock and take_over_stale_lock build it.

use std::path::PathBuf;

use super::LockRecord;

/// A held lock. `took_over` is the previous holder of a stale lock, for the caller to log as an op (S-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcquiredLock {
    pub path: PathBuf,
    pub record: LockRecord,
    pub took_over: Option<LockRecord>,
}
