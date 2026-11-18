//! decide_hooks_path: whether core.hooksPath names one hooks folder every worktree shares.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

use super::HooksPathError;

/// Decides from core.hooksPath as `git config --type=path --get` printed it (None: the key is unset). Unset means
/// git's default hooks folder in the common dir, and an absolute path names one folder; either is shared by every
/// worktree, so one install guards them all. Any other value, the empty one included, git resolves against each
/// worktree's own top, so it is refused as Relative naming the value.
pub(crate) fn decide_hooks_path(value: Option<&[u8]>) -> Result<(), HooksPathError> {
    let Some(raw) = value else { return Ok(()) };
    // git config ends the value it prints with one newline.
    let path = Path::new(OsStr::from_bytes(raw.strip_suffix(b"\n").unwrap_or(raw)));
    if path.is_absolute() {
        return Ok(());
    }
    Err(HooksPathError::Relative { value: path.to_path_buf() })
}
