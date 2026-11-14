//! PreviousSnapshot: the request's last snapshot. A declaration only; decide_snapshot_tests exercises it.

use gitio::Oid;

/// The commit the request's snapshot ref names and its tree.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PreviousSnapshot {
    pub commit: Oid,
    pub tree: Oid,
}
