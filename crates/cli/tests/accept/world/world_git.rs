//! Runs plain git in the world. Real processes: the shell step (steps/shell.rs) reads the repositories' refs
//! through it on every drive.

use std::path::Path;

use super::{Ran, World};

impl World {
    /// `git <args>` in `cwd`, run to the end: only to read state, or to act as someone other than Mortar.
    pub fn git(&self, cwd: &Path, args: &[&str]) -> Ran {
        self.command(Path::new("git"), cwd)
            .args(args)
            .output()
            .map_or_else(|error| Ran::from_error("git", error), Ran::from_output)
    }
}
