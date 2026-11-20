//! Git::sparse_add: widen a folder's sparse paths. It only runs git, so it has no unit tests; folders'
//! tests/folders/need.rs (need_widens_real_cone_and_record_repeat_is_inert_and_repeated_make_keeps_needed_paths) covers
//! it: the sparse-checkout list widens to the added folder, and that folder is filled.

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
