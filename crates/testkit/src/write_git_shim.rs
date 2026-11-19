//! Executable test-shim file I/O; tests/support.rs runs a shim that delegates to real git.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

/// Writes a caller-owned test shim and makes it executable by its owner only.
/// The supplied script is test code, not user input; this function does not execute it.
pub fn write_git_shim(path: &Path, script: &[u8]) -> io::Result<()> {
    fs::write(path, script)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))
}
