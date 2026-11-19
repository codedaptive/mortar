//! How many bytes a folder tree holds and when it was last touched.
//! No unit test: it reads the file system. The integration group is to cover, in a throwaway
//! directory: a tree of files gives the sum of their lengths and the newest mtime; a top-level entry named in skip
//! is left out, and the same name deeper down is counted; a symlink counts its own length, its target is never
//! read; a missing path gives zero bytes and no time.

use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use super::TreeSize;
use crate::make::FolderError;

/// Walks `path` with symlink_metadata, never following a symlink, and sums the length of every entry that is not a
/// directory. The length is the apparent size: blocks a clonefile copy shares with another tree are counted in full
/// here. `newest` is the newest mtime in unix seconds over every entry under `path` (not `path` itself), None when
/// there is none; an mtime before the epoch is left out. A top-level entry whose name is in `skip` is not walked.
/// A `path` that is not a directory is one entry. A missing `path` is zero bytes and no time; any other io failure
/// is Io naming the path it failed on.
pub(crate) fn measure_tree(path: &Path, skip: &[&str]) -> Result<TreeSize, FolderError> {
    let io = |at: &Path| {
        let at = at.to_path_buf();
        move |source| FolderError::Io { path: at, source }
    };
    let root = match std::fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(TreeSize { bytes: 0, newest: None }),
        Err(source) => return Err(io(path)(source)),
    };
    let mut size = TreeSize { bytes: 0, newest: None };
    if !root.is_dir() {
        add(&mut size, &root);
        return Ok(size);
    }
    let mut dirs: Vec<(PathBuf, bool)> = vec![(path.to_path_buf(), true)];
    while let Some((dir, top)) = dirs.pop() {
        for entry in std::fs::read_dir(&dir).map_err(io(&dir))? {
            let entry = entry.map_err(io(&dir))?;
            if top && skip.iter().any(|name| entry.file_name() == **name) {
                continue;
            }
            let child = entry.path();
            let meta = std::fs::symlink_metadata(&child).map_err(io(&child))?;
            add(&mut size, &meta);
            if meta.is_dir() {
                dirs.push((child, false));
            }
        }
    }
    Ok(size)
}

fn add(size: &mut TreeSize, meta: &std::fs::Metadata) {
    if !meta.is_dir() {
        size.bytes = size.bytes.saturating_add(meta.len());
    }
    let secs = meta.modified().ok().and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_secs());
    size.newest = size.newest.max(secs);
}
