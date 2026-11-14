//! with_join_lock: runs one join while holding the zone's join lock.
//! No unit test: it takes and releases a real lock file, runs `hostname`, and appends to the op log. Nagatha's
//! integration tests are to cover, in a throwaway repository: work runs with the zone's join lock file present
//! and the file is gone after; a stale lock left by a dead pid is taken over and a lock-takeover op is logged
//! before work runs; a dead same-host takeover guard removed during the acquire is logged as one
//! takeover-guard-removal op, both when the acquire wins the lock and when it refuses; a lock held by a live pid
//! refuses with JoinRunError::Lock and work does not run; a work error is returned and the lock is still released.
//! Its body repeats folders' with_folders_lock, a stopgap until oplog exports with_lock (WO-oplog-7); WO-join-2b
//! switches to oplog's and deletes the copy (S-44).

use crate::JoinRunError;
use gitio::Git;
use oplog::{
    acquire_lock, apply, guard_removal_op, host_name, lock_path, new_op_id, release_lock, takeover_op, unix_now,
    LockName, LockRecord, PidProbe,
};
use std::path::Path;

/// Takes oplog's join lock for `zone` (LockName::Join) under `common` for op `key`, logs each takeover guard the
/// acquire removed as an op (S-30), then a takeover of a stale holder (S-4), runs `work` with the lock's record
/// (its pid and host name the actor of join's ops), and releases the lock whatever work or the logging returned.
/// The guard ops come first because those removals happened before the lock was won; a failed apply skips the
/// rest of the logging and work. When the acquire refuses, its guard ops are still applied and the refusal is
/// returned as JoinRunError::Lock, unless applying a guard op failed: then that JoinRunError::Oplog is returned,
/// because the guard is already gone and a retry would not see it to log it again. work's error wins over a
/// release error; a release error is returned only when work succeeded.
pub(crate) fn with_join_lock<T>(
    git: &Git,
    common: &Path,
    zone: &str,
    key: &str,
    pids: &dyn PidProbe,
    work: impl FnOnce(&LockRecord) -> Result<T, JoinRunError>,
) -> Result<T, JoinRunError> {
    let name = LockName::Join(zone.to_string());
    let me = LockRecord {
        pid: std::process::id(),
        host: host_name().map_err(JoinRunError::Lock)?,
        key: key.to_string(),
        since: unix_now(),
    };
    let path = lock_path(common, &name).map_err(JoinRunError::Lock)?;
    let acquired = acquire_lock(common, &name, &me, pids);
    let guard_ops = match &acquired {
        Ok(lock) => guard_removal_op(&lock.path, &lock.record, &lock.removed_guards, &mut new_op_id, unix_now()),
        Err(e) => guard_removal_op(&path, &me, e.removed_guards(), &mut new_op_id, unix_now()),
    };
    let logged = guard_ops.iter().try_for_each(|op| apply(git, common, op).map(drop).map_err(JoinRunError::Oplog));
    let lock = match acquired {
        Ok(lock) => lock,
        Err(e) => return logged.and(Err(JoinRunError::Lock(e))),
    };
    let result = logged
        .and_then(|()| match takeover_op(&lock, new_op_id(), unix_now()) {
            Some(op) => apply(git, common, &op).map(drop).map_err(JoinRunError::Oplog),
            None => Ok(()),
        })
        .and_then(|()| work(&lock.record));
    let released = release_lock(&lock).map_err(JoinRunError::Lock);
    let value = result?;
    released.map(|()| value)
}
