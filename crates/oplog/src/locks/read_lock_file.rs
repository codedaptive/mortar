//! Reads a lock file and decodes its record; a missing file is no holder.
//! No unit test: it reads real files (S-5). Covered by Nagatha's integration group: refusal while
//! held (names the holder), a malformed lock file refused naming its path, release of a missing lock.

use std::io::ErrorKind;
use std::path::Path;

use super::{decode_lock_record, LockError, LockRecord};

/// The record in `path`, or None when there is no such file.
pub(crate) fn read_lock_file(path: &Path) -> Result<Option<LockRecord>, LockError> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(source) => {
            return Err(LockError::Io {
                path: path.to_path_buf(),
                source,
                removed_guards: Vec::new(),
            })
        }
    };
    match decode_lock_record(&text) {
        Ok(record) => Ok(Some(record)),
        Err(LockError::Malformed { reason, .. }) => Err(LockError::Malformed {
            path: Some(path.to_path_buf()),
            reason,
            removed_guards: Vec::new(),
        }),
        Err(other) => Err(other),
    }
}
