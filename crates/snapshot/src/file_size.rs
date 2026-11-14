//! file_size: the size the cap is checked against for one changed path in a folder.
//! No unit test: it only reads the file system and holds no decision of its own (S-5); candidate_size_tests covers
//! what it makes of the metadata. Nagatha's integration group should cover, in a throwaway folder: a regular file
//! gives its length; a symlink gives its own length, not its target's; a deleted path and a directory give None.

use crate::candidate_size::candidate_size;
use crate::SnapshotError;
use std::path::Path;

/// The size of `path` in `folder`, read with symlink_metadata so a symlink is not followed: Some(len) for a file or
/// symlink, None for a directory (a gitlink) or a path no longer there. Any other read error is FileSize.
pub(crate) fn file_size(folder: &Path, path: &Path) -> Result<Option<u64>, SnapshotError> {
    candidate_size(std::fs::symlink_metadata(folder.join(path)).map(|m| (m.is_dir(), m.len())))
        .map_err(|error| SnapshotError::FileSize { path: path.to_path_buf(), error })
}
