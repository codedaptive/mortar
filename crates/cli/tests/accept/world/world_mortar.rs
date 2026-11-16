//! Runs the built mortar binary in the world. Real processes: the shell step (steps/shell.rs) runs `mortar init`
//! and `mortar log` through it on every drive.

use std::path::Path;

use super::{Ran, World};

impl World {
    /// `mortar <args>` in `cwd`, run to the end.
    pub fn mortar(&self, cwd: &Path, args: &[&str]) -> Ran {
        self.command(World::mortar_bin(), cwd)
            .args(args)
            .output()
            .map_or_else(|error| Ran::from_error("mortar", error), Ran::from_output)
    }
}
