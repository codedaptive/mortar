//! Git::merge_base: the best common ancestor of two commits. It only runs git, so it has no unit tests; Nagatha's
//! integration test "merge_base finding the fork point, and None for unrelated histories" is to cover it.

use crate::merge_base_request::merge_base_request;
use crate::read_merge_base::read_merge_base;
use crate::{Git, GitError, Oid};

impl Git {
    /// The best common ancestor of `a` and `b`, or None when their histories share no commit.
    pub fn merge_base(&self, a: &Oid, b: &Oid) -> Result<Option<Oid>, GitError> {
        let (args, opts) = merge_base_request(a, b);
        read_merge_base(&self.run(&args, &opts)?)
    }
}
