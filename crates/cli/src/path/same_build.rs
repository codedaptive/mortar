//! same_build: whether the `mr` found on PATH is this Mortar (S-147). Pure: the caller gives both paths canonical and
//! has read the candidate's bytes for the marker.

use std::path::Path;

/// True when `candidate` and `this` are the same path, or when `carries_marker` (the candidate's bytes hold the
/// marker every Mortar binary carries: a copy of Mortar, such as cargo install's). Where the candidate sits and what
/// it is named say nothing: another program named `mr` may sit beside this binary.
pub fn same_build(candidate: &Path, this: &Path, carries_marker: bool) -> bool {
    candidate == this || carries_marker
}
