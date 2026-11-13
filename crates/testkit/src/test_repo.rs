//! TestRepo owns a fresh disposable git repository. Its lifecycle is covered by tests/support.rs.

use std::path::PathBuf;
use tempfile::TempDir;

/// A real git repository, removed with its temporary directory when this value drops.
#[derive(Debug)]
pub struct TestRepo {
    pub path: PathBuf,
    pub(crate) directory: TempDir,
}
