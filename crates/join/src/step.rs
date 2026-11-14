//! Step: what replaying one commit gave. A declaration only; replay_commit_tests exercises it.

use gitio::Oid;
use std::path::PathBuf;

/// The new commit, or the conflicted tree and its files (no commit written).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Step {
    Clean(Oid),
    Conflicted { tree: Oid, files: Vec<PathBuf> },
}
