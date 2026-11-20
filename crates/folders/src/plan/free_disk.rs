//! The question "how much disk is free here?", behind a trait so plans can be judged without a disk.

use std::path::Path;

/// Answers the bytes available to this user on the volume holding `path`, or why it cannot.
pub trait FreeDisk {
    fn free_bytes(&self, path: &Path) -> Result<u64, String>;
}
