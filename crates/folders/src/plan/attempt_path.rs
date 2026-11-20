//! Where make_attempts puts an attempt's folder.

use std::path::{Path, PathBuf};

/// `dir`/`try-<k>`: attempt k's folder inside the directory the caller names.
pub fn attempt_path(dir: &Path, k: u32) -> PathBuf {
    dir.join(format!("try-{k}"))
}
