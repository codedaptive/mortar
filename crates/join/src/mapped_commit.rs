//! MappedCommit: one range commit and the zone commit standing for it. A declaration only; map_steps_tests and the
//! ZITs in tests/replay_mapped.rs exercise it.

use gitio::Oid;

/// A commit of the replayed range and the zone commit standing for it after the replay: the running tip after it.
/// A commit dropped as empty maps to the running tip before it (onto when nothing before it was kept).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MappedCommit {
    /// The commit as it was in the range.
    pub source: Oid,
    /// The running tip after replaying `source`.
    pub replayed: Oid,
}
