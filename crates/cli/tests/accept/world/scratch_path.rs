//! Where the throwaway folder is. Covered by the shell step (steps/shell.rs) through World::new on every drive.

use std::path::Path;

use super::Scratch;

impl Scratch {
    pub fn path(&self) -> &Path {
        &self.path
    }
}
