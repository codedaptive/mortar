//! Mortar's folder under one of the user's XDG base directories.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// `<xdg>/mortar` when `xdg` is an absolute path (the XDG Base Directory spec ignores a relative or empty one);
/// else `<home>/<home_rel>/mortar` when `home` is non-empty; else None. Config passes XDG_CONFIG_HOME and
/// ".config", state XDG_STATE_HOME and ".local/state".
pub fn user_dir(xdg: Option<&OsStr>, home: Option<&OsStr>, home_rel: &str) -> Option<PathBuf> {
    if let Some(xdg) = xdg.map(Path::new).filter(|path| path.is_absolute()) {
        return Some(xdg.join("mortar"));
    }
    home.filter(|home| !home.is_empty()).map(|home| Path::new(home).join(home_rel).join("mortar"))
}
