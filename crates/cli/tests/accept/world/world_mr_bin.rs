//! Where the built `mr` binary is. Covered by the shell step (steps/shell.rs), which runs it by name through
//! World::mortar on every drive, from the bin folder world_bin_dir links it into.

use std::path::Path;

use super::World;

impl World {
    /// The `mr` binary cargo built for this test target.
    pub fn mr_bin() -> &'static Path {
        Path::new(env!("CARGO_BIN_EXE_mr"))
    }
}
