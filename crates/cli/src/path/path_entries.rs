//! path_entries: the directories a PATH value names, in lookup order, as the shell reads them (S-133 (1), S-147).
//! Pure: it reads no file system, and the current directory is passed in.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

/// The entries of `path` (a PATH value) in order: an absolute entry as given, an empty entry `cwd` and a relative
/// one `cwd.join(entry)`. A set but empty PATH is one empty entry, so `[cwd]`. An unset PATH (None) is no entries:
/// it searches nothing, neither the cwd nor any default path a shell might assume.
pub fn path_entries(path: Option<&OsStr>, cwd: &Path) -> Vec<PathBuf> {
    // Path::join keeps an absolute entry as it is and puts an empty or relative one under cwd.
    path.map_or_else(Vec::new, |path| std::env::split_paths(path).map(|entry| cwd.join(entry)).collect())
}
