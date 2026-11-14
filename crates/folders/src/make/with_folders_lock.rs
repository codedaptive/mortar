//! Runs one step of making or widening a folder while holding the folders lock.
//! No unit test: it takes and releases a real lock file, runs `hostname`, and appends to the op log (S-5).
//! Nagatha's integration group is to cover, in a throwaway repository: work runs with `mortar/locks/folders.lock`
//! present and the file is gone after; a stale lock left by a dead pid is taken over and a lock-takeover op is
//! logged before work runs; a dead same-host takeover guard removed during the acquire is logged as one
//! takeover-guard-removal op, both when the acquire wins the lock and when it refuses; a lock held by a live pid
//! refuses with FolderError::Lock and work does not run; a work error is returned and the lock is still released.

use std::path::Path;

use gitio::Git;
use oplog::{
    acquire_lock, apply, host_name, lock_path, new_op_id, release_lock, takeover_op, unix_now, LockName,
    LockRecord, PidProbe,
};

use super::guard_removal_ops::guard_removal_ops;
use super::FolderError;

/// Takes oplog's folders lock (LockName::Folders) under `common` for op `key`, logs each takeover guard the
/// acquire removed as an op (S-30), then a takeover of a stale holder (S-4), runs `work`, and releases the lock
/// whatever work or the logging returned. The guard ops come first because those removals happened before the
/// lock was won; a failed apply skips the rest of the logging and work. When the acquire refuses, its guard ops
/// are still applied and the refusal is returned as FolderError::Lock, unless applying a guard op failed: then that
/// FolderError::Oplog is returned, because the guard is already gone and a retry would not see it to log it again.
/// work's error wins over a release error; a release error is returned only when work succeeded.
pub(crate) fn with_folders_lock<T>(
    git: &Git,
    common: &Path,
    key: &str,
    pids: &dyn PidProbe,
    work: impl FnOnce() -> Result<T, FolderError>,
) -> Result<T, FolderError> {
    let me = LockRecord {
        pid: std::process::id(),
        host: host_name().map_err(FolderError::Lock)?,
        key: key.to_string(),
        since: unix_now(),
    };
    let path = lock_path(common, &LockName::Folders).map_err(FolderError::Lock)?;
    let acquired = acquire_lock(common, &LockName::Folders, &me, pids);
    let logged = guard_removal_ops(&acquired, &path, &me, &mut new_op_id, unix_now())
        .iter()
        .try_for_each(|op| apply(git, common, op).map(drop).map_err(FolderError::Oplog));
    let lock = match acquired {
        Ok(lock) => lock,
        Err(e) => return logged.and(Err(FolderError::Lock(e))),
    };
    let result = logged
        .and_then(|()| match takeover_op(&lock, new_op_id(), unix_now()) {
            Some(op) => apply(git, common, &op).map(drop).map_err(FolderError::Oplog),
            None => Ok(()),
        })
        .and_then(|()| work());
    let released = release_lock(&lock).map_err(FolderError::Lock);
    let value = result?;
    released.map(|()| value)
}
