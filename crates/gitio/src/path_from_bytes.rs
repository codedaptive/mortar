//! path_from_bytes: a path git printed, as a PathBuf.

use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;
use std::path::PathBuf;

/// The bytes as they are: git prints paths as raw unix bytes, and a non-UTF-8 name must still name the same file.
pub(crate) fn path_from_bytes(bytes: &[u8]) -> PathBuf {
    PathBuf::from(OsStr::from_bytes(bytes))
}
