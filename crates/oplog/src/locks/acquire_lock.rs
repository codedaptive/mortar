//! Takes a lock: creates its file by exclusive create (create_new temp + link(2), fails if the name exists), refuses a live holder, takes over a stale one.
//! No unit test: it creates directories and files and asks about real processes (S-5); the staleness
//! decision it applies is decide_lock_staleness, tested on its own. Covered by Nagatha's integration
//! group: acquire on a fresh dir (creates mortar/locks), refusal while held, stale takeover (returns
//! the previous holder), a holder on another host refused, two racing takers.

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
/// returned in `took_over` for the caller to log (S-4); this function writes no log.
pub fn acquire_lock(
    common: &Path,
    name: &LockName,
    me: &LockRecord,
    probe: &dyn PidProbe,
) -> Result<AcquiredLock, LockError> {
    let path = lock_path(common, name)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|source| LockError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
    }
    for _ in 0..ATTEMPTS {
        if write_new_lock_file(&path, me)? {
            return Ok(AcquiredLock {
                path,
                record: me.clone(),
                took_over: None,
            });
        }
        let Some(holder) = read_lock_file(&path)? else {
            continue;
        };
        match decide_lock_staleness(&holder, &me.host, probe.pid_alive(holder.pid)) {
            LockStaleness::Held => return Err(LockError::Held { path, holder }),
            LockStaleness::Stale => {
                if let Some(lock) = take_over_stale_lock(&path, &holder, me)? {
                    return Ok(lock);
                }
            }
        }
    }
    Err(LockError::Contended { path })
}
