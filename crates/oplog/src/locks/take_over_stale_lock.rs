//! Replaces a stale lock with ours, one taker at a time: the `<lock>.takeover` guard file, made by
//! exclusive create (create_new temp + link(2), fails if the name exists), serializes takers (S-6). A guard that already exists is refused naming its holder; there is
//! no stale-guard rule here (S-6).
//! No unit test: it writes and removes real files (S-5). Covered by Nagatha's integration group: stale
//! takeover, two racing takers (one wins, the other starts again or is refused), a crash-left guard
//! refused as a takeover in progress, and the guard removed after every takeover.

use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::read_lock_file::read_lock_file;
use super::write_new_lock_file::write_new_lock_file;
use super::{AcquiredLock, LockError, LockRecord};

/// Takes over `path` from `stale`, the holder the caller judged stale. None means the lock or its
/// guard moved underneath (gone, or replaced and gone again): the caller starts again. A holder other than
/// `stale` now in the file is refused as Held.
pub(crate) fn take_over_stale_lock(
    path: &Path,
    stale: &LockRecord,
    me: &LockRecord,
) -> Result<Option<AcquiredLock>, LockError> {
    let guard_path = takeover_guard_path(path);
    if !write_new_lock_file(&guard_path, me)? {
        return match read_lock_file(&guard_path)? {
            Some(holder) => Err(LockError::TakeoverInProgress {
                path: guard_path,
                holder,
            }),
            None => Ok(None),
        };
    }
    let _guard = TakeoverGuard(guard_path);
    replace_stale_lock(path, stale, me)
}

/// `<path>.takeover`, e.g. `land.lock.takeover`.
fn takeover_guard_path(path: &Path) -> PathBuf {
    let mut guard = OsString::from(path.as_os_str());
    guard.push(".takeover");
    PathBuf::from(guard)
}

/// The guard this process created by exclusive create (create_new temp + link(2)). Nobody else removes guards (S-6), so it is removed on
/// every exit from the takeover, errors included.
struct TakeoverGuard(PathBuf);

impl Drop for TakeoverGuard {
    fn drop(&mut self) {
        // Nothing to report to: a guard that cannot be removed is refused, naming us, on the next takeover.
        let _ = std::fs::remove_file(&self.0);
    }
}

/// With the guard held: removes the lock only if it still holds `stale`, then creates ours.
fn replace_stale_lock(
    path: &Path,
    stale: &LockRecord,
    me: &LockRecord,
) -> Result<Option<AcquiredLock>, LockError> {
    match read_lock_file(path)? {
        None => return Ok(None),
        Some(current) if current != *stale => return Err(held(path, current)),
        Some(_) => {}
    }
    match std::fs::remove_file(path) {
        Err(e) if e.kind() != ErrorKind::NotFound => {
            return Err(LockError::Io {
                path: path.to_path_buf(),
                source: e,
            })
        }
        _ => {}
    }
    if write_new_lock_file(path, me)? {
        return Ok(Some(AcquiredLock {
            path: path.to_path_buf(),
            record: me.clone(),
            took_over: Some(stale.clone()),
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
    }
}
