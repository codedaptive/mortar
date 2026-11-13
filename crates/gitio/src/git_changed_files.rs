//! Git::changed_files: the paths that differ between two commits. It only runs git, so it has no unit tests;
//! Nagatha's integration test "changed_files listing both sides of a rename and paths with spaces and newlines" is
//! to cover it.

use crate::changed_files_request::changed_files_request;
use crate::read_nul_paths::read_nul_paths;
use crate::{Git, GitError, Oid};
use std::path::PathBuf;

impl Git {
    /// Every path added, deleted or changed between `a` and `b`, in git's order; a rename shows as its old and its
    /// new path.
    pub fn changed_files(&self, a: &Oid, b: &Oid) -> Result<Vec<PathBuf>, GitError> {
        let (args, opts) = changed_files_request(a, b);
        read_nul_paths(&self.run(&args, &opts)?.stdout)
    }
}
