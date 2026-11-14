//! Runs one step of making or widening a folder while holding the folders lock.
//! No unit test: it takes and releases a real lock file, runs `hostname`, and appends to the op log (S-5).
//! Nagatha's integration group is to cover, in a throwaway repository: work runs with `mortar/locks/folders.lock`
//! present and the file is gone after; a stale lock left by a dead pid is taken over and a lock-takeover op is
//! logged before work runs; a lock held by a live pid refuses with FolderError::Lock and work does not run; a
//! work error is returned and the lock is still released.

use std::path::Path;

use gitio::Git;
use oplog::{
    acquire_lock, apply, host_name, new_op_id, release_lock, takeover_op, unix_now, LockName, LockRecord, PidProbe,
};

use super::FolderError;

/// Takes oplog's folders lock (LockName::Folders) under `common` for op `key`, logs a takeover of a stale
/// holder as an op (S-4), runs `work`, and releases the lock whatever work returned. work's error wins over a
/// release error; a release error is returned only when work succeeded.
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
    let lock = acquire_lock(common, &LockName::Folders, &me, pids).map_err(FolderError::Lock)?;
    let result = match takeover_op(&lock, new_op_id(), unix_now()) {
        Some(op) => apply(git, common, &op).map(drop).map_err(FolderError::Oplog),
        None => Ok(()),
    }
    .and_then(|()| work());
    let released = release_lock(&lock).map_err(FolderError::Lock);
    let value = result?;
    released.map(|()| value)
}
