//! Git::first_parent_list: a tip's first-parent line back to a base, oldest first. It only runs git, so it has no
//! unit tests; tests/land_ops/local_reads.rs covers it ("first_parent_list over a merge (first parents only, oldest
//! first), empty for base == tip").

use crate::first_parent_list_request::first_parent_list_request;
use crate::read_oid_lines::read_oid_lines;
use crate::{Git, GitError, Oid};

impl Git {
    /// The commits on `tip`'s first-parent line that `base` cannot reach, oldest first; a merge's other parents and
    /// what only they reach are left out. Empty when `base` is `tip` or holds it. When `base` is not an ancestor of
    /// `tip` this is still an answer, not a refusal: the part of `tip`'s line that `base` cannot reach. An unknown
    /// commit is git's own error (exit 128).
    pub fn first_parent_list(&self, base: &Oid, tip: &Oid) -> Result<Vec<Oid>, GitError> {
        let (args, opts) = first_parent_list_request(base, tip);
        read_oid_lines(&self.run(&args, &opts)?.stdout)
    }
}
