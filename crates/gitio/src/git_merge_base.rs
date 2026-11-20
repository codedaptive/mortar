//! Git::merge_base: the best common ancestor of two commits. It only runs git, so it has no unit tests; land's
//! tests/squash_zit/squash_cases.rs (a_squash_keeps_origins_newer_change_and_adds_only_the_zones) covers it through
//! squash, which takes its base from it: the squash keeps origin's newer change and adds only the zone's commit.

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
