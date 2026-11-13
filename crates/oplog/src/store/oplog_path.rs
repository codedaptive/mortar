//! Where the op log lives inside a repository's git common dir.

use std::path::{Path, PathBuf};

/// `<common>/mortar/oplog.jsonl`.
pub fn oplog_path(common: &Path) -> PathBuf {
    common.join("mortar").join("oplog.jsonl")
}
