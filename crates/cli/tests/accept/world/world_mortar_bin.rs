//! Where the built `mortar` binary is. Covered by the shell step (steps/shell.rs): world_bin_dir links it into the
//! bin folder first on every child's PATH, where the guard hooks Mortar installs find it by name.

use std::path::Path;

use super::World;

impl World {
    /// The `mortar` binary cargo built for this test target.
    pub fn mortar_bin() -> &'static Path {
        Path::new(env!("CARGO_BIN_EXE_mortar"))
    }
}
