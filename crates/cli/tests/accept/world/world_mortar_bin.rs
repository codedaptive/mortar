//! Where the built mortar binary is. Covered by the shell step (steps/shell.rs), which runs it through
//! World::mortar on every drive.

use std::path::Path;

use super::World;

impl World {
    /// The binary cargo built for this test target.
    pub fn mortar_bin() -> &'static Path {
        Path::new(env!("CARGO_BIN_EXE_mortar"))
    }
}
