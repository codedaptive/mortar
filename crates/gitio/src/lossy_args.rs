//! lossy_args: git arguments as text for errors.

use std::ffi::OsString;

/// The arguments as text for errors; a non-UTF-8 argument shows with U+FFFD.
pub(crate) fn lossy_args(args: &[OsString]) -> Vec<String> {
    args.iter().map(|arg| arg.to_string_lossy().into_owned()).collect()
}
