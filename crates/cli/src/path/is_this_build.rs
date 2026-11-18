//! is_this_build: whether a candidate binary found on PATH is this Mortar once both paths are canonical (S-147).
//! Pure: the caller gives the canonicalize and the marker read to use, so tests pass fakes.

use std::path::{Path, PathBuf};

use super::same_build;

/// True when `candidate` and `this` both canonicalize and same_build says the two are one build: the same canonical
/// path, or `has_marker` true of the canonical candidate. `has_marker` is asked only when both canonicalize and
/// differ. A canonicalize that gives None, of either path, counts as not this build.
pub fn is_this_build(
    candidate: &Path,
    this: &Path,
    canonical: &dyn Fn(&Path) -> Option<PathBuf>,
    has_marker: &dyn Fn(&Path) -> bool,
) -> bool {
    match (canonical(candidate), canonical(this)) {
        (Some(candidate), Some(this)) => same_build(&candidate, &this, candidate != this && has_marker(&candidate)),
        (None, _) | (_, None) => false,
    }
}
