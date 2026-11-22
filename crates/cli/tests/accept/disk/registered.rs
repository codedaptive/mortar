//! One team repository a team manifest registers. Data only: registered_repos makes it, copy_refusal reads it,
//! and copy_refusal_tests.rs covers that reading.

use std::path::PathBuf;

/// `path` is the manifest's repo path, canonical when it exists; `common` is its git common dir, canonical, or None
/// when the path does not exist.
#[derive(Debug, Clone, PartialEq)]
pub struct Registered {
    pub path: PathBuf,
    pub common: Option<PathBuf>,
}
