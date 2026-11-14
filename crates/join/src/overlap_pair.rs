//! OverlapPair: two open requests that change at least one same path. A declaration only; pair_overlaps_tests
//! exercises it.

use std::path::PathBuf;

/// Two requests by name, `first` before `second`, and the paths both change: each once, in Path's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OverlapPair {
    pub first: String,
    pub second: String,
    pub paths: Vec<PathBuf>,
}
