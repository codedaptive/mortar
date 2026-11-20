//! dir_position: Where a folder stands among PATH's entries.

use std::path::{Path, PathBuf};

/// The index of the first entry equal to `dir` as a Path, or None when no entry is.
pub fn dir_position(dir: &Path, entries: &[PathBuf]) -> Option<usize> {
    entries.iter().position(|entry| entry == dir)
}
