//! PreviousSnapshot: the last snapshot on a snapshot ref. A declaration only; decide_snapshot_tests exercises it.

use gitio::Oid;

/// The commit a snapshot ref names and its tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreviousSnapshot {
    pub commit: Oid,
    pub tree: Oid,
}
