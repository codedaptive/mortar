//! Gives a lock back: removes its file, only if the file holds our record when it is read.
//! No unit test: it reads and removes real files (S-5). Covered by Nagatha's integration group:
//! release after acquire removes the file, release of a lock taken over by another is refused naming
//! the new holder, release of a missing lock is refused.

use std::io::ErrorKind;

use super::read_lock_file::read_lock_file;
use super::{AcquiredLock, LockError};

/// Removes `lock`'s file. NotOurs when the file is gone or holds another record.
pub fn release_lock(lock: &AcquiredLock) -> Result<(), LockError> {
    let path = &lock.path;
    match read_lock_file(path)? {
        None => {
            return Err(LockError::NotOurs {
                path: path.clone(),
                holder: None,
            })
        }
        Some(current) if current != lock.record => {
            return Err(LockError::NotOurs {
                path: path.clone(),
                holder: Some(current),
            })
        }
        Some(_) => {}
    }
    std::fs::remove_file(path).map_err(|source| match source.kind() {
        ErrorKind::NotFound => LockError::NotOurs {
            path: path.clone(),
            holder: None,
        },
        _ => LockError::Io {
            path: path.clone(),
            source,
            removed_guards: Vec::new(),
        },
    })
}
