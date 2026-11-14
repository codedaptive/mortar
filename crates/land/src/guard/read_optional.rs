//! read_optional: a file's bytes, or None when it is not there. It only does I/O, so it has no unit test (S-5);
//! tests/guard_zit.rs covers a file present and absent (every guard_status and read_guard_config case); no test
//! reaches the Io arm.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use super::GuardError;

/// The bytes at `path`; None when there is no file; any other failure is Io naming the path.
pub(crate) fn read_optional(path: &Path) -> Result<Option<Vec<u8>>, GuardError> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(GuardError::Io { path: path.to_path_buf(), error }),
    }
}
