//! Clones a directory tree with clonefile(2), macOS's copy-on-write clone.
//! It holds the crate's only `unsafe`: the call into libSystem, which std does not wrap.
//! No unit test: it calls the OS; tests/folders/build_output.rs covers it through clone_build_dirs.

use std::path::Path;

/// Clones `source` to `target` (which must not exist) with clonefile(2): the clone shares `source`'s blocks
/// until a file in either changes. A symlink at `source` is cloned as the link, not followed. Never falls back
/// to a byte copy: on any failure nothing is copied and the error says why.
#[cfg(target_os = "macos")]
pub(crate) fn clonefile_dir(source: &Path, target: &Path) -> Result<(), String> {
    use std::ffi::{c_char, c_int, CString};
    use std::os::unix::ffi::OsStrExt;

    extern "C" {
        fn clonefile(src: *const c_char, dst: *const c_char, flags: u32) -> c_int;
    }
    // CLONE_NOFOLLOW from <sys/clonefile.h>: a symlink at `source` is cloned as the link, never followed.
    const CLONE_NOFOLLOW: u32 = 0x0001;

    let src = CString::new(source.as_os_str().as_bytes())
        .map_err(|_| format!("{} holds a NUL byte; nothing copied", source.display()))?;
    let dst = CString::new(target.as_os_str().as_bytes())
        .map_err(|_| format!("{} holds a NUL byte; nothing copied", target.display()))?;
    // SAFETY: both pointers come from CStrings that are valid and NUL-terminated and outlive the call;
    // clonefile only reads them.
    let status = unsafe { clonefile(src.as_ptr(), dst.as_ptr(), CLONE_NOFOLLOW) };
    if status != 0 {
        return Err(format!("clonefile refused: {}", std::io::Error::last_os_error()));
    }
    Ok(())
}

/// clonefile(2) exists on macOS only, so elsewhere every clone is refused and nothing is copied.
#[cfg(not(target_os = "macos"))]
pub(crate) fn clonefile_dir(_source: &Path, _target: &Path) -> Result<(), String> {
    Err("clonefile(2) is macOS only; nothing copied".to_string())
}
