//! A folder's apparent bytes. Real files: the drive (steps/disk.rs) runs it through run_disk on the zone and
//! request folders on every drive where both were made.

use std::io;
use std::path::Path;

/// The length of every entry under `path` that is not a directory, walked with symlink_metadata so a link counts
/// its own length and its target is never read; the top-level `.git` and `target` are left out. The same measure
/// as folders' measure_folder files bytes, so the numbers compare with `mr disk`'s per-folder lines.
pub fn tree_bytes(path: &Path) -> io::Result<u64> {
    let mut bytes: u64 = 0;
    let mut dirs = vec![(path.to_path_buf(), true)];
    while let Some((dir, top)) = dirs.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            if top && matches!(entry.file_name().to_str(), Some(".git" | "target")) {
                continue;
            }
            let meta = std::fs::symlink_metadata(entry.path())?;
            if meta.is_dir() {
                dirs.push((entry.path(), false));
            } else {
                bytes = bytes.saturating_add(meta.len());
            }
        }
    }
    Ok(bytes)
}
