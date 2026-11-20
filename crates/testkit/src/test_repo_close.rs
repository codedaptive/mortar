//! Optional explicit cleanup reports errors; TempDir also cleans up on drop.
//! Real-directory behavior is covered by tests/support.rs.

use crate::TestRepo;
use std::io;

impl TestRepo {
    /// Removes the repository now, reporting any cleanup failure instead of ignoring it on drop.
    pub fn close(self) -> io::Result<()> {
        self.directory.close()
    }
}
