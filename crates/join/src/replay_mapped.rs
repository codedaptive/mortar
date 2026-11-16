//! replay_mapped: replay a request's commits onto a new base and say which zone commit stands for each, moving no
//! ref.
//! No unit test: it only joins replay_mapped_in to gitio::Git; replay_mapped_in_tests cover the decisions and the
//! ZITs in tests/replay_mapped.rs cover it.

use crate::replay_mapped_in::replay_mapped_in;
use crate::{JoinError, MappedReplay};
use gitio::{Git, Oid};

/// Replays the commits of `base..tip` onto `onto` commit by commit, as replay's commit-by-commit path does, and
/// maps each commit, oldest first, to the running tip after it: a commit dropped as empty maps to the running tip
/// before it, or to `onto` when nothing before it was kept. `tip` is a full ref name (such as `refs/heads/topic`),
/// read by full name only. The result has the same outcome as replay's for the range (same trees, messages, authors
/// and chain; commit ids may differ). Conflicted: only the commits before the conflicted one are mapped; the
/// conflicted commit and the remaining commits are not. Checks first that `onto` exists. Writes objects; moves no
/// ref.
pub fn replay_mapped(git: &Git, base: &Oid, tip: &str, onto: &Oid) -> Result<MappedReplay, JoinError> {
    replay_mapped_in(git, base, tip, onto)
}
