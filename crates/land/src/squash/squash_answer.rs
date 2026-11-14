//! SquashAnswer: what a squash did. A declaration only (S-7); Nagatha's squash integration tests of SPEC-land
//! 01-squash.md §Tests are to exercise each variant (they do not exist yet).

use crate::squash::Squash;
use gitio::Oid;
use std::path::PathBuf;

/// The answer of one squash. No variant moves the zone branch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SquashAnswer {
    /// A new commit was written and one op moved the squash ref to it.
    Made(Squash),
    /// The commit already at the squash ref has the same tree, parent and message: nothing written, no op.
    Same(Squash),
    /// The zone tip is the merge base with origin: nothing to squash.
    Nothing { source: Oid },
    /// The zone's changes conflict with origin in these files: no commit written, no ref moved.
    Conflicted { files: Vec<PathBuf> },
}
