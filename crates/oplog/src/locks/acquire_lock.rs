//! Takes a lock: creates its file by exclusive create (create_new temp + link(2), fails if the name exists), refuses a live holder, takes over a stale one.
//! Every stale takeover guard removed on the way, across all attempts, is returned on the lock or on the refusal,
//! so the caller logs each one.
//! No unit test: it creates directories and files and asks about real processes; the staleness
//! decision it applies is decide_lock_staleness, tested on its own. Covered by the crate's integration tests
//! (tests/oplog/locks.rs): acquire on a fresh dir (creates mortar/locks), refusal while held, stale takeover
//! (returns the previous holder), a holder on another host refused, two racing takers.

use std::path::Path;

use super::read_lock_file::read_lock_file;
use super::take_over_stale_lock::take_over_stale_lock;
use super::write_new_lock_file::write_new_lock_file;
use super::{
    decide_lock_staleness, lock_path, AcquiredLock, LockError, LockName, LockRecord, LockStaleness,
    PidProbe,
};

/// How many times the lock may vanish or change hands under us before acquire gives up as Contended.
const ATTEMPTS: usize = 3;

/// Takes lock `name` under the git common directory `common` for `me`. A stale holder taken over is
/// returned in `took_over`. Every stale takeover guard removed on the way, in order, is returned in
/// `removed_guards`: on the lock when acquire succeeds, on the refusal when it refuses. Both are for the
/// caller to log; this function writes no log.
pub fn acquire_lock(
    common: &Path,
    name: &LockName,
    me: &LockRecord,
    probe: &dyn PidProbe,
) -> Result<AcquiredLock, LockError> {
    let mut removed = Vec::new();
    match acquire_recording_removals(common, name, me, probe, &mut removed) {
        Ok(mut lock) => {
            lock.removed_guards = removed;
            Ok(lock)
        }
        Err(e) => Err(e.with_removed_guards(removed)),
    }
}

/// acquire_lock's attempts, pushing each takeover guard it removes onto `removed_guards`, so every return path,
/// `?` included, leaves the full list for acquire_lock to put on its result.
fn acquire_recording_removals(
    common: &Path,
    name: &LockName,
    me: &LockRecord,
    probe: &dyn PidProbe,
    removed_guards: &mut Vec<LockRecord>,
) -> Result<AcquiredLock, LockError> {
    let path = lock_path(common, name)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|source| LockError::Io {
            path: dir.to_path_buf(),
            source,
            removed_guards: Vec::new(),
        })?;
    }
    for _ in 0..ATTEMPTS {
        if write_new_lock_file(&path, me)? {
            return Ok(AcquiredLock {
                path,
                record: me.clone(),
                took_over: None,
                removed_guards: Vec::new(),
            });
        }
        let Some(holder) = read_lock_file(&path)? else {
            continue;
        };
        match decide_lock_staleness(&holder, &me.host, probe.pid_alive(holder.pid)) {
            LockStaleness::Held => {
                return Err(LockError::Held {
                    path,
                    holder,
                    removed_guards: Vec::new(),
                })
            }
            LockStaleness::Stale => {
                if let Some(lock) = take_over_stale_lock(&path, &holder, me, probe, removed_guards)? {
                    return Ok(lock);
                }
            }
        }
    }
    Err(LockError::Contended {
        path,
        removed_guards: Vec::new(),
    })
}
