//! make_dirs: Makes a folder and the missing ones above it, saying which it made (spec 07 item 7: made folders are recorded).
//! No sibling test: it touches the disk. The install integration tests in install/integration/ cover it: uninstall.rs
//! (uninstall_restores_home_after_install, uninstall_keeps_preexisting_directory) and placement.rs.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Creates each missing folder from the top down, stopping at the first that cannot be made. Returns the folders it
/// made, in that order, also when it stopped early, so a caller records them either way; then the error, if any. A
/// path that already stands (a folder, a link or anything else) is not made and not returned.
pub fn make_dirs(dir: &Path) -> (Vec<PathBuf>, io::Result<()>) {
    let mut missing: Vec<PathBuf> = dir
        .ancestors()
        .take_while(|path| !path.as_os_str().is_empty() && fs::symlink_metadata(path).is_err())
        .map(Path::to_path_buf)
        .collect();
    missing.reverse();
    let mut made = Vec::with_capacity(missing.len());
    for path in missing {
        if let Err(error) = fs::create_dir(&path) {
            return (made, Err(error));
        }
        made.push(path);
    }
    (made, Ok(()))
}
