//! LandingStateError: why the landing state file could not be read, written or removed. A bare type (S-7);
//! landing_state_error_display_tests and landing_state_error_source_tests exercise each variant.

use std::io;
use std::path::PathBuf;

/// A failure on `<common>/mortar/landing.json` (or its temp file), naming the path. Debug only: io::Error is
/// neither Clone nor Eq.
#[derive(Debug)]
pub enum LandingStateError {
    /// The file exists but could not be read (a file that is not UTF-8 included).
    Read { path: PathBuf, source: io::Error },
    /// The file's text is not in the layout encode_landing_state writes; the reason says where.
    Corrupt { path: PathBuf, reason: String },
    /// The state folder, the temp file or the rename over the file failed; the path is the one that failed.
    Write { path: PathBuf, source: io::Error },
    /// The file exists but could not be removed.
    Remove { path: PathBuf, source: io::Error },
}
