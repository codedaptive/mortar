//! path_warning_for: whether `mr guard install` warns about the first `mr` on PATH, and with which
//! line. Pure: the caller finds both paths and gives the canonicalize and the marker read to use, so tests pass fakes.
//! The is-this-build test itself lives in is_this_build.rs.

use std::path::{Path, PathBuf};

use super::{is_this_build, mr_path_warning};

/// None when `this` (this binary's path) is None: there is no `mortar` path to name. Else mr_path_warning's line
/// for `first` (the first `mr` on PATH), which is None only when is_this_build says `first` is this build (both
/// canonicalize, and same_build holds of the canonical paths; `has_marker` asked only when they differ). The line
/// names `first` and `this` as given, not canonicalized.
pub fn path_warning_for(
    this: Option<&Path>,
    first: Option<&Path>,
    canonical: &dyn Fn(&Path) -> Option<PathBuf>,
    has_marker: &dyn Fn(&Path) -> bool,
) -> Option<String> {
    let this = this?;
    let same = first.is_some_and(|first| is_this_build(first, this, canonical, has_marker));
    mr_path_warning(first, this, same)
}
