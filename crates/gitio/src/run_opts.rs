//! RunOpts: what one git run adds to the Git value's settings.

use std::ffi::OsString;
use std::path::PathBuf;

/// Options for one run; the default is no stdin, no extra env, the repo dir, only exit 0 accepted, a write command.
#[derive(Clone, Debug, Default)]
pub struct RunOpts {
    /// Bytes written to git's stdin; None gives git a null stdin.
    pub stdin: Option<Vec<u8>>,
    /// Extra environment for this run, applied after the Git value's env.
    pub env: Vec<(OsString, OsString)>,
    /// Directory passed as `-C` in place of the Git value's repo.
    pub cwd: Option<PathBuf>,
    /// Non-zero exit codes that are answers rather than errors (merge-tree's 1 is a conflict answer).
    pub answer_codes: Vec<i32>,
    /// A read command: git takes no optional locks (such as status's index refresh) and `git diff` does not refresh
    /// the index's stat data; those are the index writes these settings stop.
    pub read_only: bool,
}
