//! Formatter: the program a snapshot runs over its changed files before it saves them. A declaration only;
//! formatter_argv_tests exercises it.

use std::ffi::OsString;
use std::path::PathBuf;

/// Run as `program`, then `args`, then the paths to format, with no shell, in the folder (the child's current
/// directory). `program` is a bare name, looked up on PATH, or an absolute path; a relative path holding a separator
/// is resolved by the OS against the folder. The paths are appended as they are, so a formatter that takes `--`
/// gets it last in `args`: without it a path starting with `-` is read as an option. The child inherits the process
/// environment; `git.env` is not applied, because the formatter is not git.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Formatter {
    pub program: PathBuf,
    pub args: Vec<OsString>,
}
