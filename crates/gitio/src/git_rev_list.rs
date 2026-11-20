//! Git::rev_list: the commits between two commits, oldest first. It only runs git, so it has no unit tests; land's
//! tests/squash_zit/squash_cases.rs (a_squash_keeps_origins_newer_change_and_adds_only_the_zones) covers it through
//! squash: the squash message lists the zone's one commit.

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
