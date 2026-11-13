//! Git::attr_check: the `merge` attribute of paths as a commit sets it. It only runs git, so it has no unit tests;
//! Nagatha's integration test "attr_check reading merge=union, -merge, merge and unspecified from a commit's
//! attributes, not the folder's" is to cover it.

use crate::attr_check_request::attr_check_request;
use crate::read_attr_check::read_attr_check;
use crate::{Git, GitError, Oid};
use std::path::PathBuf;

impl Git {
    /// Each of `paths` with its `merge` attribute as `commit`'s .gitattributes set it, not the folder's, in the order
    /// given: "set", "unset", "unspecified" or the value (e.g. "union"). The answer is unspecified for a missing
    /// commit: a commit id naming no object answers "unspecified" for every path (git 2.56 exits 0), so the caller
    /// passes a commit it has. An empty list, or an empty path or one holding a NUL, is refused.
    pub fn attr_check(&self, commit: &Oid, paths: &[PathBuf]) -> Result<Vec<(PathBuf, String)>, GitError> {
        let (args, opts) = attr_check_request(commit, paths)?;
        read_attr_check(&self.run(&args, &opts)?.stdout, paths)
    }
}
