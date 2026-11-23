//! symlinked_ancestor: the disk check that no folder on the way to a named path is a symlink.
//! No unit test: it only reads the file system. The integration test
//! tests/snapshot/commit_paths_cases.rs::commit_paths_refuses_symlink_ancestor covers it through commit_paths.

use crate::CommitPathsError;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

/// Looks (lstat, not following links) at each proper ancestor of `path` under `folder`, shallowest first:
/// `folder/a`, `folder/a/b`, ... A symlink is ThroughSymlink naming that ancestor relative to the folder; one not
/// found ends the check, as nothing deeper exists; any other error is Ancestor. The path's own last part is not
/// looked at: it may be a symlink, which is committed as one.
pub(crate) fn symlinked_ancestor(folder: &Path, path: &Path) -> Result<(), CommitPathsError> {
    let parts: Vec<_> = path.components().collect();
    let mut link = PathBuf::new();
    for part in &parts[..parts.len().saturating_sub(1)] {
        link.push(part);
        match fs::symlink_metadata(folder.join(&link)) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(CommitPathsError::ThroughSymlink { path: path.to_path_buf(), link });
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(CommitPathsError::Ancestor { path: path.to_path_buf(), error }),
        }
    }
    Ok(())
}
