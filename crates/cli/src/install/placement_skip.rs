//! placement_skip: Why install skips placing the binary, if it does (spec 07 item 1).

use std::path::{Path, PathBuf};

/// The reason placement is skipped, or None: --no-place, then `this` unknown, then `this` under a package-manager prefix
/// (whole path components, so `/usr/localx` is not under `/usr/local`).
pub fn placement_skip(this: Option<&Path>, prefixes: &[PathBuf], no_place: bool) -> Option<String> {
    if no_place {
        return Some("--no-place".to_string());
    }
    let Some(this) = this else {
        return Some("this binary's path is unknown".to_string());
    };
    prefixes.iter().find(|prefix| this.starts_with(prefix)).map(|prefix| {
        format!("{} is under {}, a package manager's prefix", this.display(), prefix.display())
    })
}
