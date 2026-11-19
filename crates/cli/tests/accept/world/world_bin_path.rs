//! The world's PATH: cargo's own build output folder, which holds the `mr` and `mortar` files it built, first, then
//! the drive's PATH. The drive runs `mr` as cargo builds it, with no links. Pure: World::new passes it the
//! built paths and the drive's PATH.

use std::ffi::{OsStr, OsString};
use std::path::Path;

/// `mr`'s folder, then every entry of `drive_path` in order. An error, never a panic, when `mortar` is not in that
/// same folder (the guard hooks Mortar installs run `mortar` by name, so it must be found where `mr` is), when `mr`
/// has no folder, or when the folder cannot be a PATH entry (it holds the separator).
pub fn world_bin_path(mr: &Path, mortar: &Path, drive_path: &OsStr) -> Result<OsString, String> {
    let bin = mr.parent().ok_or_else(|| format!("the built mr has no folder: {}", mr.display()))?;
    if mortar.parent() != Some(bin) {
        return Err(format!("the built mortar {} is not beside the built mr {}", mortar.display(), mr.display()));
    }
    std::env::join_paths(std::iter::once(bin.to_path_buf()).chain(std::env::split_paths(drive_path)))
        .map_err(|error| format!("{} cannot be a PATH entry: {error}", bin.display()))
}
