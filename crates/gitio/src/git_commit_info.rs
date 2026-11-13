//! Git::commit_info: one commit's tree, parents and message. It only runs git, so it has no unit tests; Nagatha's
//! integration test "commit_info on a root, a one-parent and a merge commit" is to cover it.

use crate::commit_info_request::commit_info_request;
use crate::read_commit_info::read_commit_info;
use crate::{CommitInfo, Git, GitError, Oid};

impl Git {
    /// The tree, parents and message of `commit`. An id that names no commit (such as a tree) is refused.
    pub fn commit_info(&self, commit: &Oid) -> Result<CommitInfo, GitError> {
        let (args, opts) = commit_info_request(commit);
        read_commit_info(&self.run(&args, &opts)?.stdout, commit)
    }
}
