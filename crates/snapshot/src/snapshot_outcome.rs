//! SnapshotOutcome: what a snapshot call left behind. A declaration only; snapshot_from_apply_tests exercises it.

use crate::{FormatterRun, RefusedFile};
use gitio::Oid;

/// The snapshot commit the ref names after the call, its tree, whether this call wrote that commit and moved the
/// ref to it (false: the tree equalled the last snapshot's and nothing was written), and each new file over the
/// size cap that was left out of the tree, with its size and the cap. `formatted` is the formatter's run (command,
/// paths, result), None when nothing ran (no formatter, or no file to format).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SnapshotOutcome {
    pub commit: Oid,
    pub tree: Oid,
    pub written: bool,
    pub refused: Vec<RefusedFile>,
    pub formatted: Option<FormatterRun>,
}
