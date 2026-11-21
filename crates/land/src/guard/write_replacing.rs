//! write_replacing: put a file's whole new content in place in one step. It writes files, so it has no unit test.
//! tests/land_rit/guard_codec.rs and guard_installation.rs check the guard.json bytes it writes;
//! guard_installation.rs also checks the hook contents and exec bits.

use std::fs::{self, File, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use super::create_with_redraw::create_with_redraw;
use super::draw_temp_path::draw_temp_path;
use super::GuardError;

/// Writes `bytes` to a temp file in `path`'s folder named by `draw_temp_path` (`<name>.mortar-tmp-<16 hex>`),
/// sets `mode` on it when given, and renames it over `path`. A reader (git running a hook, or the hook reading
/// guard.json) sees the old file or the whole new one, never a part, and a hook is executable from the moment it
/// appears.
///
/// The temp is opened with `create_new`, so it never follows or reuses anything already at its name (a planted
/// symlink, a crash-left temp); a name already taken is drawn again by `create_with_redraw`, up to its
/// `CREATE_ATTEMPTS` names in all. Failures are `GuardError::Io`: creating, writing or setting the mode names the
/// temp path, the rename names `path`. A failure after the temp exists removes it first, best effort.
pub(crate) fn write_replacing(path: &Path, bytes: &[u8], mode: Option<u32>) -> Result<(), GuardError> {
    let (tmp, file) =
        create_with_redraw(|| draw_temp_path(path), |tmp| OpenOptions::new().write(true).create_new(true).open(tmp))?;
    let result = fill(file, bytes, mode)
        .map_err(io_error(&tmp))
        .and_then(|()| fs::rename(&tmp, path).map_err(io_error(path)));
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// Writes `bytes` and sets `mode` through the open handle; the file closes when this returns.
fn fill(mut file: File, bytes: &[u8], mode: Option<u32>) -> io::Result<()> {
    file.write_all(bytes)?;
    if let Some(mode) = mode {
        file.set_permissions(Permissions::from_mode(mode))?;
    }
    Ok(())
}

/// Io naming `path`, for map_err.
fn io_error(path: &Path) -> impl FnOnce(io::Error) -> GuardError {
    let path = path.to_path_buf();
    move |error| GuardError::Io { path, error }
}
