//! write_replacing: put a file's whole new content in place in one step. It writes files, so it has no unit test (S-5).
//! tests/land_rit/guard_codec.rs and guard_installation.rs check the guard.json bytes it writes;
//! guard_installation.rs also checks the hook contents and exec bits.

use std::ffi::OsString;
use std::fs::{self, Permissions};
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::GuardError;

/// Writes `bytes` to `<path>.mortar-tmp` in the same folder, sets `mode` on it when given, and renames it over
/// `path`. A reader (git running a hook, or the hook reading guard.json) sees the old file or the whole new one,
/// never a part, and a hook is executable from the moment it appears.
pub(crate) fn write_replacing(path: &Path, bytes: &[u8], mode: Option<u32>) -> Result<(), GuardError> {
    let mut tmp = OsString::from(path);
    tmp.push(".mortar-tmp");
    let tmp = PathBuf::from(tmp);
    fs::write(&tmp, bytes).map_err(io_error(&tmp))?;
    if let Some(mode) = mode {
        fs::set_permissions(&tmp, Permissions::from_mode(mode)).map_err(io_error(&tmp))?;
    }
    fs::rename(&tmp, path).map_err(io_error(path))
}

/// Io naming `path`, for map_err.
fn io_error(path: &Path) -> impl FnOnce(io::Error) -> GuardError {
    let path = path.to_path_buf();
    move |error| GuardError::Io { path, error }
}
