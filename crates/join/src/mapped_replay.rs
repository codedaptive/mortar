//! MappedReplay: a replay's result with each commit's zone commit. A declaration only; map_steps_tests and the ZITs
//! in tests/replay_mapped.rs exercise it.

use crate::mapped_commit::MappedCommit;
use crate::ReplayResult;

/// What replay_mapped gave: the replay's result, and the commits of the range mapped to the zone commits standing
/// for them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappedReplay {
    /// The Clean or Conflicted outcome replay gives for the range (same tip tree, and for a conflict the same commit).
    pub result: ReplayResult,
    /// Oldest first. Clean: one entry per commit of the range, and the last entry's `replayed` is the Clean tip (onto
    /// when every commit dropped); an empty range has no entries. Conflicted: only the commits before the conflicted
    /// one; the conflicted commit and `remaining` do not appear.
    pub mapped: Vec<MappedCommit>,
}
