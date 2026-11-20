//! The `<lock>.takeover` guard path for a lock: the one place that names it.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// `<lock_path>.takeover`, e.g. `land.lock.takeover`. The takeover and the guard-removal op both use it.
pub(crate) fn takeover_guard_path(lock_path: &Path) -> PathBuf {
    let mut guard = OsString::from(lock_path.as_os_str());
    guard.push(".takeover");
    PathBuf::from(guard)
}
