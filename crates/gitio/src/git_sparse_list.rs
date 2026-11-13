//! Git::sparse_list: the folders a sparse folder holds. It only runs git, so it has no unit tests; Nagatha's
//! integration test "sparse_list reading back sparse_set and sparse_add folders with quoted names, and refusing a
//! non-cone folder" is to cover it.

use crate::read_sparse_list::read_sparse_list;
use crate::sparse_list_request::sparse_list_request;
use crate::{Git, GitError};
use std::path::{Path, PathBuf};

impl Git {
    /// The cone folders of `folder`, a sparse worktree of this repository, in git's (sorted) order; an empty cone is
    /// no paths. A folder that is not sparse is git's error; one whose patterns are not cone mode is refused.
    pub fn sparse_list(&self, folder: &Path) -> Result<Vec<PathBuf>, GitError> {
        let (args, opts) = sparse_list_request(folder)?;
        read_sparse_list(&self.run(&args, &opts)?)
    }
}
