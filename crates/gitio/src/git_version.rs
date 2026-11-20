//! Git::version: the installed git's release, checked against mortar's floor.

use crate::{Git, GitError, RunOpts, Version};

impl Git {
    /// Runs `git version` as a read; the parsed release when it is at least mortar's floor, else a refusal.
    pub fn version(&self) -> Result<Version, GitError> {
        let opts = RunOpts {
            read_only: true,
            ..RunOpts::default()
        };
        Version::parse(&self.run(&["version"], &opts)?.stdout)?.require_floor()
    }
}
