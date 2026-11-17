//! Runs the built binary as `mr` in the world. Real processes: the shell step (steps/shell.rs) runs `mr init` and
//! `mr log` through it on every drive.

use std::path::Path;

use super::{Ran, World};

impl World {
    /// `mr <args>` in `cwd`, run to the end. `mr` is found by name on the world's PATH (cargo's build output
    /// folder), as an agent's shell finds it.
    pub fn mortar(&self, cwd: &Path, args: &[&str]) -> Ran {
        self.command(Path::new("mr"), cwd)
            .args(args)
            .output()
            .map_or_else(|error| Ran::from_error("mr", error), Ran::from_output)
    }
}
