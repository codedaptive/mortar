//! What with_lock can return: a lock failure, an op log failure, or the error its work returned.
//! Its own type, with one arm for each, so each caller (folders, join) maps every arm into its own error type
//! without needing From impls, and knows which of the three it was.
//! A declaration only; with_lock is what produces each arm.

use crate::locks::LockError;
use crate::store::OplogError;

/// The error from running work under a held lock.
#[derive(Debug)]
pub enum HeldLockError<E> {
    /// Taking, naming or releasing the lock failed: host_name, lock_path, the acquire's refusal, or the release.
    Lock(LockError),
    /// Applying a takeover-guard-removal op or a lock-takeover op failed.
    Oplog(OplogError),
    /// The error `work` returned, untouched.
    Work(E),
}
