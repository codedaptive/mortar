//! Worktree: one entry of a repository's worktree list. A declaration only; read_worktree_list_tests exercises it.

use crate::Oid;
use std::path::PathBuf;

/// A worktree as `git worktree list --porcelain` describes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Worktree {
    /// The worktree's folder, as git printed it (non-UTF-8 names kept).
    pub path: PathBuf,
    /// The commit checked out; None for a bare repository, which has no HEAD line.
    pub head: Option<Oid>,
    /// The full branch name checked out, such as `refs/heads/main`; None when detached or bare.
    pub branch: Option<String>,
    pub bare: bool,
    pub detached: bool,
    /// Locked against removal and pruning (the reason git may give is not kept).
    pub locked: bool,
    /// Git would prune it: its folder is gone (the reason git gives is not kept).
    pub prunable: bool,
}
