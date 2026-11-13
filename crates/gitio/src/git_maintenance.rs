//! Git::maintenance: pack the repository's loose objects. It only runs git, so it has no unit tests; Nagatha's
//! integration test "maintenance packing a repository's loose objects" is to cover it.

use crate::maintenance_request::maintenance_request;
use crate::{Git, GitError};

impl Git {
    /// Runs git's gc task on this repository (from any of its worktrees). Ok also when another maintenance holds the
    /// repository's maintenance lock: git then skips the run silently and exits 0, so Ok does not prove objects were
    /// packed. Not a repository is git's error.
    pub fn maintenance(&self) -> Result<(), GitError> {
        let (args, opts) = maintenance_request();
        self.run(&args, &opts).map(drop)
    }
}
