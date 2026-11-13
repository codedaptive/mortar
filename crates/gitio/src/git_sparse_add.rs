//! Git::sparse_add: widen a folder's sparse paths. It only runs git, so it has no unit tests; Nagatha's integration
//! test "sparse_add widening a sparse_set folder and filling the added folders" is to cover it.

use crate::sparse_add_request::sparse_add_request;
use crate::{Git, GitError};
use std::path::Path;

impl Git {
    /// Adds `paths` (whole folders) to the cone of `folder`, a sparse worktree of this repository, and fills them. A
    /// folder that is not sparse, or whose patterns are not cone mode, is git's error.
    pub fn sparse_add(&self, folder: &Path, paths: &[String]) -> Result<(), GitError> {
        let (args, opts) = sparse_add_request(folder, paths)?;
        self.run(&args, &opts).map(drop)
    }
}
