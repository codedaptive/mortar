//! Git::worktree_list: the repository's worktrees. It only runs git with a fixed argv, so it has no unit tests;
//! read_worktree_list_tests cover the reading, and Nagatha's integration test "worktree_list showing the main
//! folder, a detached locked worktree and a prunable one" is to cover the run.

use crate::read_worktree_list::read_worktree_list;
use crate::{Git, GitError, RunOpts, Worktree};

impl Git {
    /// Every worktree of the repository, the main one first, as git lists them.
    pub fn worktree_list(&self) -> Result<Vec<Worktree>, GitError> {
        let output = self.run(
            &["worktree", "list", "--porcelain", "-z"],
            &RunOpts {
                read_only: true,
                ..RunOpts::default()
            },
        )?;
        read_worktree_list(&output.stdout)
    }
}
