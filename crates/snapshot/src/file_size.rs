//! file_size: the size the cap is checked against for one changed path in a folder.
//! No unit test: it only reads the file system and holds no decision of its own (S-5); candidate_size_tests covers
//! what it makes of the metadata. Nagatha's integration tests cover, through snapshot in a throwaway repository, a
//! regular file giving its length, as the size a refused file is listed with: 6 MiB in
//! tests/snapshot/caps.rs::new_six_mib_file_is_reported_but_large_changed_head_file_is_kept, 8 bytes in
//! caps.rs::caller_cap_is_inclusive_and_refused_glob_name_excludes_only_that_path, 4 bytes in
//! caps.rs::new_against_head_file_that_grows_or_is_renamed_over_cap_drops_from_next_snapshot.
//! Not yet covered by an integration test: a symlink gives its own length, not its target's; a deleted path and a
//! directory give None.

use crate::candidate_size::candidate_size;
use crate::SnapshotError;
use std::path::Path;

/// The size of `path` in `folder`, read with symlink_metadata so a symlink is not followed: Some(len) for a file or
/// symlink, None for a directory (a gitlink) or a path no longer there. Any other read error is FileSize.
pub(crate) fn file_size(folder: &Path, path: &Path) -> Result<Option<u64>, SnapshotError> {
    candidate_size(std::fs::symlink_metadata(folder.join(path)).map(|m| (m.is_dir(), m.len())))
        .map_err(|error| SnapshotError::FileSize { path: path.to_path_buf(), error })
}
