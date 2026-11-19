//! Git::commit_info: one commit's tree, parents, author and message. It only runs git, so it has no unit tests;
//! tests/typed/commit_info.rs covers a root, a one-parent and a merge commit, and an empty author date passed
//! through; tests/typed/author_round_trip.rs covers its author passed to commit_tree rebuilding the same commit id.

use crate::commit_info_request::commit_info_request;
use crate::read_commit_info::read_commit_info;
use crate::{CommitInfo, Git, GitError, Oid};

impl Git {
    /// The tree, parents, author and message of `commit`. An id that names no commit (such as a tree) is refused.
    pub fn commit_info(&self, commit: &Oid) -> Result<CommitInfo, GitError> {
        let (args, opts) = commit_info_request(commit);
        read_commit_info(&self.run(&args, &opts)?.stdout, commit)
    }
}
