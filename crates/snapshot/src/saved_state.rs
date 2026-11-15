//! saved_state: whether a folder's files equal the target's last snapshot, with the folder's tree (built without a
//! formatter) and the files the size cap refused.
//! No unit test: it only runs git, reads file sizes and composes decide_snapshot and folder_tree's cap decisions
//! (cap_candidates, cap_refusals, capped_excludes, cap_mismatch), which have their own tests (S-5). is_saved's
//! integration tests reach it through is_saved (is_saved.rs names them), and directly in
//! tests/snapshot/saved_state_cases.rs::saved_state_tracks_the_snapshot_ref_and_lists_the_refused_new_file_without_moving_ref_or_log:
//! - no snapshot yet: saved false, and tree is the tree the next snapshot of the unchanged folder records;
//! - saved true after a snapshot, and tree is the snapshot ref's tree;
//! - a new over-cap file: saved true, refused names that file with its size and the cap, and tree is still the
//!   ref's tree, without that file;
//! - a change: saved false, with refused still listed, and tree differs from the ref's; a fresh snapshot then
//!   records that tree.

use crate::decide_snapshot::decide_snapshot;
use crate::folder_tree::folder_tree;
use crate::read_previous_snapshot::read_previous_snapshot;
use crate::snapshot_decision::SnapshotDecision;
use crate::snapshot_ref_name::snapshot_ref_name;
use crate::{SavedState, SnapshotError, SnapshotTarget};
use gitio::Git;
use std::path::Path;

/// Whether `folder`'s tree, built as snapshot builds it, equals the tree of the target's snapshot ref
/// (snapshot_ref_name, as snapshot), and each new file the size cap left out of that tree. `saved` is false when
/// the trees differ or there is no snapshot yet; `refused` is reported whether or not the folder is saved. `git`
/// gives the binary and environment; every run is in `folder`, which must be the folder's top. The caller passes
/// the excludes and the size cap it snapshots with, or the trees differ. It returns the tree it built as `tree`,
/// built without a formatter (snapshot_formatted runs one first, so with a formatter its tree can differ).
/// Writes tree objects (folder_tree does, through gitio's snapshot_tree) but moves no ref and appends no op record.
/// The cli's status and the caller of folders' remove are to read it: remove takes `saved` and the refused paths
/// and refuses unless `saved` is true and no file was refused (S-32). That rule is not applied here.
pub fn saved_state(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<SavedState, SnapshotError> {
    let git = Git { repo: folder.to_path_buf(), ..git.clone() };
    let refname = snapshot_ref_name(target)?;
    let (_, tree, refused) = folder_tree(&git, folder, extra_excludes, size_cap)?;
    let previous = read_previous_snapshot(&git, &refname)?;
    let saved = matches!(decide_snapshot(&tree, previous.as_ref()), SnapshotDecision::Unchanged(_));
    Ok(SavedState { saved, tree, refused })
}
