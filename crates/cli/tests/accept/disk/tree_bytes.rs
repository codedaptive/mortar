//! A folder's apparent bytes. Real files: the drive (steps/disk.rs) runs it through measure_zone on each zone folder
//! and request folder that was made, and on a request folder again when its zone's debug build was taken.

use std::io;
use std::path::Path;

/// The length of every entry under `path` that is not a directory, walked with symlink_metadata so a link counts
/// its own length and its target is never read; the top-level entries named in `leave_out` are skipped. With
/// [".git", "target"] it is the files measure the folders crate's measure_folder takes, so the numbers compare with
/// `mr disk`'s per-folder lines; with [".git"] it counts a folder's build output too.
pub fn tree_bytes(path: &Path, leave_out: &[&str]) -> io::Result<u64> {
    let mut bytes: u64 = 0;
    let mut dirs = vec![(path.to_path_buf(), true)];
    while let Some((dir, top)) = dirs.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            if top && entry.file_name().to_str().is_some_and(|name| leave_out.contains(&name)) {
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
