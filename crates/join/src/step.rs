//! Step: what replaying one commit gave. A declaration only; replay_commit_tests exercises it.

use gitio::Oid;
use std::path::PathBuf;

/// What replaying one commit onto the running tip gave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    /// The commit applied; this is the new commit, written on the running tip.
    Clean(Oid),
    /// The commit became empty on the running tip; no commit was written and the running tip stands.
    Dropped,
    /// The commit did not apply cleanly: the merged tree holding the conflicts and its conflicted files. No commit
    /// was written.
    Conflicted { tree: Oid, files: Vec<PathBuf> },
}
