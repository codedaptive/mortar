//! Removes a throwaway folder that was never closed, such as when World::new fails part way. Real file I/O with no
//! sibling check: the drive's shell step removes its world through scratch_close.rs, so this runs only on a failure.

use std::fs;

use super::Scratch;

impl Drop for Scratch {
    fn drop(&mut self) {
        // Empty once close has taken it; an error here has no one left to report to.
        if !self.path.as_os_str().is_empty() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
