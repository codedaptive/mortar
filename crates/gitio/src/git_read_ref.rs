//! Git::read_ref: one full ref's id. It only runs git, so it has no unit tests; Nagatha's integration test
//! "read_ref finding a full ref, answering None for an absent one, and never resolving a short name" is to cover it.
//! A corrupt (unreadable) ref reads as absent (None): git answers it with the same not-a-valid-ref line as an absent
//! one, so the two cannot be told apart from its output.

use crate::read_ref_request::read_ref_request;
use crate::read_show_ref::read_show_ref;
use crate::{Git, GitError, Oid};

impl Git {
    /// The id the full ref `refname` points at (a symbolic ref's target's id), or None when it is absent.
    pub fn read_ref(&self, refname: &str) -> Result<Option<Oid>, GitError> {
        let (args, opts) = read_ref_request(refname)?;
        read_show_ref(&self.run(&args, &opts)?, refname)
    }
}
