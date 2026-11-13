//! Git::range_diff: what changed between the reviewed and the new version of a series. It only runs git, so it has
//! no unit tests; Nagatha's integration test "range_diff showing an amended commit between a reviewed and a new
//! series, without colour codes" is to cover it.

use crate::range_diff_request::range_diff_request;
use crate::{Git, GitError, Oid};

impl Git {
    /// git's range-diff of `base..reviewed` against `base..new`, as text for a reviewer: bytes that are not UTF-8
    /// show as U+FFFD.
    pub fn range_diff(&self, base: &Oid, reviewed: &Oid, new: &Oid) -> Result<String, GitError> {
        let (args, opts) = range_diff_request(base, reviewed, new);
        Ok(String::from_utf8_lossy(&self.run(&args, &opts)?.stdout).into_owned())
    }
}
