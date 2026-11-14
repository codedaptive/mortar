//! CapCandidate: a changed file the size cap is checked against. A declaration only; cap_refusals_tests and
//! cap_candidates_tests exercise it.

use std::path::PathBuf;

/// A changed path in the folder, its size in bytes, and whether HEAD already holds it (a file in HEAD is never
/// refused, whatever its size).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CapCandidate {
    pub(crate) path: PathBuf,
    pub(crate) size: u64,
    pub(crate) in_head: bool,
}
