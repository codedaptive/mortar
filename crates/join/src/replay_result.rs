//! ReplayResult: what a replay gave. A declaration only; replay_fallback_tests and replay_in_tests exercise it.

use gitio::Oid;
use std::path::PathBuf;

/// The replayed tip, or the state a conflicted replay stopped in (what a resolve needs to continue it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReplayResult {
    /// Every commit applied; this is the new tip (onto itself when the range was empty). No ref has moved.
    Clean(Oid),
    /// A commit did not apply cleanly; nothing after it was replayed and no ref has moved.
    Conflicted {
        /// The commit that did not apply.
        commit: Oid,
        /// The tip replayed so far: onto when the conflicted commit was the first.
        replayed_tip: Oid,
        /// The commits after `commit` still to replay, oldest first.
        remaining: Vec<Oid>,
        /// The merged tree holding the conflicts, with conflict markers.
        tree: Oid,
        /// Each conflicted path once, in git's order.
        files: Vec<PathBuf>,
    },
}
