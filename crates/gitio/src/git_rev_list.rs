//! Git::rev_list: the commits between two commits, oldest first. It only runs git, so it has no unit tests;
//! Nagatha's integration test "rev_list oldest first over a merge, empty for base == tip" is to cover it.

use crate::read_oid_lines::read_oid_lines;
use crate::rev_list_request::rev_list_request;
use crate::{Git, GitError, Oid};

impl Git {
    /// The commits reachable from `tip` and not from `base`, every parent before its children; empty when `base`
    /// is `tip` or already holds `tip`.
    pub fn rev_list(&self, base: &Oid, tip: &Oid) -> Result<Vec<Oid>, GitError> {
        let (args, opts) = rev_list_request(base, tip);
        read_oid_lines(&self.run(&args, &opts)?.stdout)
    }
}
