//! SnapshotOutcome: what a snapshot call left behind. A declaration only; snapshot_from_apply_tests exercises it.

use crate::RefusedFile;
use gitio::Oid;

/// The snapshot commit the ref names after the call, its tree, whether this call wrote that commit and moved the
/// ref to it (false: the tree equalled the last snapshot's and nothing was written), and each new file over the
/// size cap that was left out of the tree, with its size and the cap.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotOutcome {
    pub commit: Oid,
    pub tree: Oid,
    pub written: bool,
    pub refused: Vec<RefusedFile>,
}
