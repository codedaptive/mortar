//! Removes one step's throwaway repositories. Real file I/O: run_step (steps/run_step.rs) records it as a check on
//! every step that ran.

use std::io;

use super::World;

impl World {
    /// Removes the scratch folder and everything the step made in it.
    pub fn close(self) -> io::Result<()> {
        self.scratch.close()
    }
}
