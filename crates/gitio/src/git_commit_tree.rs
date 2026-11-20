//! Git::commit_tree: write a commit object. It only runs git, so it has no unit tests; tests/typed/commits_and_refs.rs
//! and tests/typed/author_round_trip.rs cover it.

use crate::commit_tree_request::commit_tree_request;
use crate::oid_line::oid_line;
use crate::{Git, GitError, Oid, Person};

impl Git {
    /// Writes a commit of `tree` with `parents` in order and `message` as given, and returns its id. A None author
    /// or committer leaves that identity to git's own settings. No ref moves.
    pub fn commit_tree(
        &self,
        tree: &Oid,
        parents: &[Oid],
        message: &str,
        author: Option<&Person>,
        committer: Option<&Person>,
    ) -> Result<Oid, GitError> {
        let (args, opts) = commit_tree_request(tree, parents, message, author, committer)?;
        oid_line(&self.run(&args, &opts)?.stdout)
    }
}
