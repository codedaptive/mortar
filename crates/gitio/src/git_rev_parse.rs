//! Git::rev_parse: resolve a rev to a commit id. It only runs git, so it has no unit tests; tests/typed/
//! commits_and_refs.rs `rev_parse_peels_annotated_tag_and_list_refs_respects_prefix` covers it: a branch's commit,
//! an annotated tag peeled to its commit, and None for a missing ref and for `--help`.

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
