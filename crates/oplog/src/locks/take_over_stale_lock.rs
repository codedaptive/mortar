//! Replaces a stale lock with ours, one taker at a time: the `<lock>.takeover` guard file, made by exclusive create
//! (create_new temp + link(2), fails if the name exists), serializes takers, and a per-holder reap guard
//! serializes removal of a dead guard. A guard whose holder is stale by decide_lock_staleness (same host, dead pid)
//! is removed if it still holds that holder, pushed onto the caller's `removed_guards`, and the guard is taken
//! again. A live or other-host holder, or an existing reap guard, is refused with TakeoverInProgress.
//! Errors and the lock built here carry an empty `removed_guards`: acquire_lock owns the list and puts it on its
//! result.
//! No unit test: it writes and removes real files and asks about real processes; the decision it applies is
//! decide_lock_staleness, tested on its own. Covered by the crate's integration tests: stale takeover
//! (tests/oplog/locks.rs); two racing takers (one wins, the other starts again or is refused), a live or
//! other-host guard refused as a takeover in progress, a dead same-host guard removed and returned in
//! `removed_guards`, two takers racing to remove the same dead guard (one removes it)
//! (tests/oplog/takeover_guards.rs); and the guard removed after every takeover (tests/oplog/locks.rs and
//! tests/oplog/takeover_guards.rs).

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::read_lock_file::read_lock_file;
use super::takeover_guard_path;
use super::write_new_lock_file::write_new_lock_file;
use super::{decide_lock_staleness, AcquiredLock, LockError, LockRecord, LockStaleness, PidProbe};

/// Takes over `path` from `stale`, the holder the caller judged stale. None means the lock or its guard moved
/// underneath (gone, or replaced and gone again): the caller starts again. A holder other than `stale` now in the
/// file is refused as Held. Each stale guard this call removes is pushed onto `removed_guards`, in order.
pub(crate) fn take_over_stale_lock(
    path: &Path,
    stale: &LockRecord,
    me: &LockRecord,
    probe: &dyn PidProbe,
    removed_guards: &mut Vec<LockRecord>,
) -> Result<Option<AcquiredLock>, LockError> {
    let guard_path = takeover_guard_path(path);
    if !write_new_lock_file(&guard_path, me)? {
        let Some(holder) = read_lock_file(&guard_path)? else {
            return Ok(None);
        };
        if decide_lock_staleness(&holder, &me.host, probe.pid_alive(holder.pid)) == LockStaleness::Held {
            return Err(LockError::TakeoverInProgress { path: guard_path, holder, removed_guards: Vec::new() });
        }
        if !remove_guard_if_held_by(&guard_path, &holder, me)? {
            return Ok(None);
        }
        removed_guards.push(holder);
        if !write_new_lock_file(&guard_path, me)? {
            return match read_lock_file(&guard_path)? {
                Some(holder) => {
                    Err(LockError::TakeoverInProgress { path: guard_path, holder, removed_guards: Vec::new() })
                }
                None => Ok(None),
            };
        }
    }
    let _guard = TakeoverGuard(guard_path);
    replace_stale_lock(path, stale, me)
}

/// Removes the guard only if it still holds `holder`. True only when this call removed it; a guard that changed
/// or is already gone is false, so the caller starts again rather than claim another taker's removal.
/// Sound because only reapers of `holder` unlink a guard holding it, they are serialized by its reap guard
/// `<guard>.reap-<pid>-<since>`, and `holder` never reappears once gone: no taker can split the compare and the
/// unlink. An existing reap guard (crash-left included) is refused with TakeoverInProgress naming it.
fn remove_guard_if_held_by(guard_path: &Path, holder: &LockRecord, me: &LockRecord) -> Result<bool, LockError> {
    let mut reap = OsString::from(guard_path.as_os_str());
    reap.push(format!(".reap-{}-{}", holder.pid, holder.since));
    let reap = PathBuf::from(reap);
    if !write_new_lock_file(&reap, me)? {
        return match read_lock_file(&reap)? {
            Some(h) => Err(LockError::TakeoverInProgress { path: reap, holder: h, removed_guards: Vec::new() }),
            None => Ok(false),
        };
    }
    let _reap = TakeoverGuard(reap);
    if read_lock_file(guard_path)?.as_ref() != Some(holder) {
        return Ok(false);
    }
    match std::fs::remove_file(guard_path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(false),
        Err(source) => Err(LockError::Io { path: guard_path.to_path_buf(), source, removed_guards: Vec::new() }),
    }
}

/// A guard file (takeover or reap) this process created by exclusive create (create_new temp + link(2)). This
/// process removes its own on every exit, errors included; another taker removes a takeover guard only when its
/// holder is stale.
struct TakeoverGuard(PathBuf);

impl Drop for TakeoverGuard {
    fn drop(&mut self) {
        // Nothing to report to: a guard that cannot be removed stays. A takeover guard left so is removed by the
        // next taker if its holder is stale, else refused naming us; a reap guard is refused naming us until
        // removed by hand.
        let _ = std::fs::remove_file(&self.0);
    }
}

/// With the guard held: removes the lock only if it still holds `stale`, then creates ours.
fn replace_stale_lock(path: &Path, stale: &LockRecord, me: &LockRecord) -> Result<Option<AcquiredLock>, LockError> {
    match read_lock_file(path)? {
        None => return Ok(None),
        Some(current) if current != *stale => return Err(held(path, current)),
        Some(_) => {}
    }
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != ErrorKind::NotFound => {
            return Err(LockError::Io { path: path.to_path_buf(), source: e, removed_guards: Vec::new() })
        }
        _ => {}
    }
    if write_new_lock_file(path, me)? {
        return Ok(Some(AcquiredLock {
            path: path.to_path_buf(),
            record: me.clone(),
            took_over: Some(stale.clone()),
            removed_guards: Vec::new(),
        }));
    }
    match read_lock_file(path)? {
        Some(current) => Err(held(path, current)),
        None => Ok(None),
    }
}

fn held(path: &Path, holder: LockRecord) -> LockError {
    LockError::Held {
        path: path.to_path_buf(),
        holder,
        removed_guards: Vec::new(),
    }
}
