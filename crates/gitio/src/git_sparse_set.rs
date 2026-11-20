//! Git::sparse_set: set a folder's sparse paths and fill them. It only runs git, so it has no unit tests;
//! tests/typed/worktrees.rs covers it: a worktree added by worktree_add filled with the cone and the root files only.

use crate::sparse_set_requests::sparse_set_requests;
use crate::{Git, GitError};
use std::path::Path;

impl Git {
    /// Makes `folder` (a worktree of this repository) hold only `paths` (cone mode: whole folders, plus the files at
    /// the top), then checks out what that set covers.
    pub fn sparse_set(&self, folder: &Path, paths: &[String]) -> Result<(), GitError> {
        let [set, checkout] = sparse_set_requests(folder, paths)?;
        self.run(&set.0, &set.1)?;
        self.run(&checkout.0, &checkout.1).map(drop)
    }
}
