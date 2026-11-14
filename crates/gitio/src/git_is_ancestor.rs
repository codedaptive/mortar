//! Git::is_ancestor: whether one commit is an ancestor of another. It only runs git, so it has no unit tests;
//! tests/land_ops/local_reads.rs covers it ("is_ancestor true, false, true for a == b, error for an unknown commit").

use crate::is_ancestor_request::is_ancestor_request;
use crate::read_is_ancestor::read_is_ancestor;
use crate::{Git, GitError, Oid};

impl Git {
    /// True when `a` is reachable from `b` through any parents, including `a == b`; false when it is not. An unknown
    /// commit is git's own error (exit 128), never false.
    pub fn is_ancestor(&self, a: &Oid, b: &Oid) -> Result<bool, GitError> {
        let (args, opts) = is_ancestor_request(a, b);
        read_is_ancestor(&self.run(&args, &opts)?)
    }
}
