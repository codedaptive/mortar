//! Git::maintenance: pack the repository's loose objects. It only runs git, so it has no unit tests; folders'
//! tests/folders/sweeps.rs
//! (sweep_deletes_closed_snapshot_joined_leftover_and_extra_but_keeps_open_unjoined_and_checked_out) checks only that
//! sweep_refs runs it, through a shim that records the call and then runs real git. No test checks that it packs loose
//! objects yet.

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
