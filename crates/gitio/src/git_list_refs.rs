//! Git::list_refs: the refs under a prefix with their ids. It only runs git, so it has no unit tests;
//! tests/typed/commits_and_refs.rs covers it: the refs under a prefix, none under a missing one, every ref for an
//! empty prefix, and an annotated tag left unpeeled.

use crate::list_refs_request::list_refs_request;
use crate::read_ref_list::read_ref_list;
use crate::{Git, GitError, Oid};

impl Git {
    /// Each ref matching `prefix` (every ref when it is empty) as (id, full refname), in git's order.
    pub fn list_refs(&self, prefix: &str) -> Result<Vec<(Oid, String)>, GitError> {
        let (args, opts) = list_refs_request(prefix)?;
        read_ref_list(&self.run(&args, &opts)?.stdout)
    }
}
