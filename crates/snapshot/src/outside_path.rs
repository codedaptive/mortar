//! outside_path: the pure check that a named path stays inside the folder by its text alone. Its tests are in
//! outside_path_tests.rs; the integration tests in tests/snapshot/commit_paths_cases.rs (commit_paths_refuses_absolute,
//! commit_paths_refuses_parent_part) get both refusals through commit_paths.

use crate::CommitPathsError;
use std::path::{Component, Path};

/// Absolute when `path` has a root, else ParentPart when any of its parts is `..`, else None. A symlinked folder on
/// the way is not seen here: symlinked_ancestor checks the disk.
pub(crate) fn outside_path(path: &Path) -> Option<CommitPathsError> {
    if path.has_root() {
        return Some(CommitPathsError::Absolute { path: path.to_path_buf() });
    }
    if path.components().any(|part| part == Component::ParentDir) {
        return Some(CommitPathsError::ParentPart { path: path.to_path_buf() });
    }
    None
}
