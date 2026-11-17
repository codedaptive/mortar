//! Every way installing, reading or checking the guard can fail.
//! A bare type (S-7); guard_error_display_tests exercises the Io, ForeignHook and BadConfig texts,
//! guard_error_source_tests each variant's source; the Git text is checked in tests/land_rit/owed_displays.rs.

use std::path::PathBuf;

use gitio::GitError;

/// A guard step that git, the file system or the guard itself stopped.
#[derive(Debug)]
pub enum GuardError {
    /// git could not name the hooks folder or the common dir.
    Git(GitError),
    /// Reading or writing `path` failed.
    Io { path: PathBuf, error: std::io::Error },
    /// A hook at `path` that Mortar did not write; the guard never overwrites one.
    ForeignHook { path: PathBuf },
    /// guard.json at `path` could not be read as a guard config, for the reason `why`.
    BadConfig { path: PathBuf, why: String },
}
