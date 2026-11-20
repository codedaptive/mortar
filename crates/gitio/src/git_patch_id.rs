//! Git::patch_id: the stable patch ID of one commit's change. It only runs git, so it has no unit tests;
//! tests/typed/patch_id.rs covers it: one change on two bases with different messages, authors and dates, two
//! different changes, an unchanged tree, a root commit, a merge, an unknown commit, hostile diff config, a hostile
//! environment, uncommitted worktree attributes, and body lines that look like patch headers.

use crate::patch_id_diff_request::patch_id_diff_request;
use crate::patch_id_hash_request::patch_id_hash_request;
use crate::read_patch_id::read_patch_id;
use crate::{Git, GitError, Oid, PatchId};

impl Git {
    /// The stable patch ID of `commit`'s change against its first parent: a root commit against the empty tree, a
    /// merge against its first parent. None when that change is empty. The same change on any parent gives the same
    /// ID; the message, author and dates never change it. The diff is held in memory once, between the two runs. A
    /// git failure, such as an unknown commit, is an error, never None.
    pub fn patch_id(&self, commit: &Oid) -> Result<Option<PatchId>, GitError> {
        let (args, opts) = patch_id_diff_request(commit);
        let (args, opts) = patch_id_hash_request(self.run(&args, &opts)?.stdout);
        read_patch_id(&self.run(&args, &opts)?.stdout, commit)
    }
}
