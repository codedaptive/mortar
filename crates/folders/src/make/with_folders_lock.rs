//! Runs one step of making or widening a folder while holding the folders lock.
//! No unit test: it takes and releases a real lock file, runs `hostname`, and appends to the op log.
//! The integration group is to cover, in a throwaway repository: work runs with `mortar/locks/folders.lock`
//! present and the file is gone after; a stale lock left by a dead pid is taken over and a lock-takeover op is
//! logged before work runs; a dead same-host takeover guard removed during the acquire is logged as one
//! takeover-guard-removal op, both when the acquire wins the lock and when it refuses; a lock held by a live pid
//! refuses with FolderError::Lock and work does not run; a work error is returned and the lock is still released.

use std::path::Path;

use gitio::Git;
use oplog::{with_lock, HeldLockError, LockName, PidProbe};

use super::FolderError;

/// Takes oplog's folders lock (LockName::Folders) under `common` for holder key `key` and runs `work` through
/// oplog::with_lock, which logs the takeover-guard-removal ops and the lock-takeover op in the order its
/// doc gives, runs work under the lock and releases the lock whatever work returned. HeldLockError::Lock becomes
/// FolderError::Lock, HeldLockError::Oplog becomes FolderError::Oplog, and work's own FolderError is returned
/// untouched.
pub(crate) fn with_folders_lock<T>(
    git: &Git,
    common: &Path,
    key: &str,
    pids: &dyn PidProbe,
    work: impl FnOnce() -> Result<T, FolderError>,
) -> Result<T, FolderError> {
    with_lock(git, common, &LockName::Folders, key, pids, work).map_err(|e| match e {
        HeldLockError::Lock(e) => FolderError::Lock(e),
        HeldLockError::Oplog(e) => FolderError::Oplog(e),
        HeldLockError::Work(e) => e,
    })
}
