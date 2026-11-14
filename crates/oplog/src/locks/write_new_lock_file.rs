//! Publishes a lock file holding a record, only if no file of that name exists; an existing one is
//! left untouched. The record is written and synced to a private temp file in the same directory,
//! then hard-linked to the lock's name: a reader never sees an empty or partial record, and
//! hard_link refuses an existing name just as O_EXCL does. Used for lock files and takeover guards.
//! No unit test: it writes real files (S-5). Covered by Nagatha's integration group: acquire on a
//! fresh dir, refusal while held, two racing takers (exactly one creates the file), a reader racing
//! a writer never sees a partial record.

use std::ffi::OsString;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{encode_lock_record, LockError, LockRecord};

/// Makes each temp name unique per call within this process; pid and time separate processes.
static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// True when this call published `path` holding `record`; false when `path` already exists.
pub(crate) fn write_new_lock_file(path: &Path, record: &LockRecord) -> Result<bool, LockError> {
    let text = encode_lock_record(record)?;
    let temp = temp_path(path);
    write_temp(&temp, text.as_bytes()).map_err(|source| io(&temp, source))?;
    let linked = std::fs::hard_link(&temp, path);
    // Once linked the lock is held through `path`; a temp left behind is litter, not a failure.
    remove_temp(&temp);
    match linked {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == ErrorKind::AlreadyExists => Ok(false),
        Err(source) => Err(io(path, source)),
    }
}

/// `<lock file name>.<pid>.<unix nanos>.<n>.tmp`, beside `path`.
fn temp_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let mut name = OsString::from(path.as_os_str());
    name.push(format!(
        ".{}.{}.{}.tmp",
        std::process::id(),
        nanos,
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    PathBuf::from(name)
}

/// create_new refuses an existing file or a planted symlink, so the bytes go only to a file we made.
/// A failed write removes that file; a refused open removes nothing, since the name is not ours.
fn write_temp(temp: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(temp)?;
    let written = file.write_all(bytes).and_then(|()| file.sync_all());
    if written.is_err() {
        drop(file);
        remove_temp(temp);
    }
    written
}

fn remove_temp(temp: &Path) {
    let _ = std::fs::remove_file(temp);
}

fn io(path: &Path, source: std::io::Error) -> LockError {
    LockError::Io {
        path: path.to_path_buf(),
        source,
        removed_guards: Vec::new(),
    }
}
