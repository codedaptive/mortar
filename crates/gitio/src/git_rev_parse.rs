//! Git::rev_parse: resolve a rev to a commit id. It only runs git, so it has no unit tests; Nagatha's integration
//! test "rev_parse finding a commit, peeling a tag and answering None for a missing rev" is to cover it.

use crate::read_rev_parse::read_rev_parse;
use crate::rev_parse_request::rev_parse_request;
use crate::{Git, GitError, Oid};

impl Git {
    /// The commit `rev` names, or None when it names no commit.
    pub fn rev_parse(&self, rev: &str) -> Result<Option<Oid>, GitError> {
        let (args, opts) = rev_parse_request(rev)?;
        read_rev_parse(&self.run(&args, &opts)?)
    }
}
