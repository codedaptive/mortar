//! Removes the throwaway folder. Real file I/O: run_step (steps/run_step.rs) records it as a check through
//! World::close on every step that ran.

use std::fs;
use std::io;

use super::Scratch;

impl Scratch {
    /// Takes the path, so the drop that follows removes nothing more.
    pub fn close(mut self) -> io::Result<()> {
        fs::remove_dir_all(std::mem::take(&mut self.path))
    }
}
