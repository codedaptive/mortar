//! file_size: the size the cap is checked against for one changed path in a folder.
//! No unit test: it only reads the file system and holds no decision of its own; candidate_size_tests covers
//! what it makes of the metadata. The integration tests cover, through snapshot in a throwaway repository, a
//! regular file giving its length, as the size a refused file is listed with: 6 MiB in
//! tests/snapshot/caps.rs::new_six_mib_file_is_reported_but_large_changed_head_file_is_kept, 8 bytes in
//! caps.rs::caller_cap_is_inclusive_and_refused_glob_name_excludes_only_that_path, 4 bytes in
//! caps.rs::new_against_head_file_that_grows_or_is_renamed_over_cap_drops_from_next_snapshot.
//! A deleted path, a path under a file and a plain directory give None, shown on new candidates (not in HEAD) at a
//! zero cap, where each is written with nothing refused, in
//! tests/snapshot/sizing_none.rs::new_candidates_deleted_under_a_replaced_parent_or_now_plain_directories_do_not_become_size_refusals.
//! Paths in HEAD are covered only as a snapshot that succeeds with one (no FileSize), not as None itself: a deleted
//! path in
//! tests/snapshot/formatter_paths.rs::formatter_gets_only_regular_saved_nonconflicted_changes_not_deleted_refused_excluded_links_or_submodules
//! and, at a zero cap with the path gone from the tree, in
//! tests/snapshot/sizing_edges.rs::deleted_changed_path_is_not_a_size_refusal_even_at_zero_cap; a path under a file (a
//! HEAD directory's old children once a file of its name replaced it) in
//! sizing_edges.rs::over_cap_file_replacing_head_directory_is_refused_without_resurrecting_old_children.
//! A symlink gives its own length, not its target's: sizing_edges.rs::symlink_cap_uses_its_own_eleven_byte_length_not_sixty_four_byte_target
//! (its 11 bytes are refused at a cap of 10 and kept at 11, beside a 64-byte target).
//! A new gitlink directory gives None (it is not refused at a zero cap):
//! sizing_edges.rs::new_gitlink_directory_is_not_treated_as_a_file_or_refused_at_zero_cap.
//! Another read error gives FileSize: sizing_edges.rs::real_metadata_symlink_loop_is_file_size_error_before_commit_ref_or_log_and_retry_preserves_work
//! (an ELOOP).

use crate::candidate_size::candidate_size;
use crate::SnapshotError;
use std::path::Path;

/// The size of `path` in `folder`, read with symlink_metadata so a symlink is not followed: Some(len) for a file or
/// symlink, None for a directory (a gitlink or a plain one), a path no longer there or a path under a file. Any
/// other read error is FileSize.
pub(crate) fn file_size(folder: &Path, path: &Path) -> Result<Option<u64>, SnapshotError> {
    candidate_size(std::fs::symlink_metadata(folder.join(path)).map(|m| (m.is_dir(), m.len())))
        .map_err(|error| SnapshotError::FileSize { path: path.to_path_buf(), error })
}
