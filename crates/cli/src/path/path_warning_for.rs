//! path_warning_for: whether `mr guard install` warns about the first `mr` on PATH (S-133 (1), S-147), and with which
//! line. Pure: the caller finds both paths and gives the canonicalize and the marker read to use, so tests pass fakes.

use std::path::{Path, PathBuf};

use super::{mr_path_warning, same_build};

/// None when `this` (this binary's path) is None: there is no `mortar` path to name. Else mr_path_warning's line
/// for `first` (the first `mr` on PATH), which is None only when `first` and `this` both canonicalize and
/// same_build says the two are one build: the same canonical path, or `has_marker` true of the canonical candidate.
/// `has_marker` is asked only when both canonicalize and differ. A canonicalize that gives None counts as not this
/// build. The line names `first` and `this` as given, not canonicalized.
pub fn path_warning_for(
    this: Option<&Path>,
    first: Option<&Path>,
    canonical: &dyn Fn(&Path) -> Option<PathBuf>,
    has_marker: &dyn Fn(&Path) -> bool,
) -> Option<String> {
    let this = this?;
    let same = first.is_some_and(|first| match (canonical(first), canonical(this)) {
        (Some(first), Some(this)) => same_build(&first, &this, first != this && has_marker(&first)),
        (None, _) | (_, None) => false,
    });
    mr_path_warning(first, this, same)
}
