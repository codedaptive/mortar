//! is_saved: whether a folder's files equal the target's last snapshot.
//! No unit test: it only calls saved_state and holds no decision of its own. The integration tests in tests/snapshot
//! cover it, in a throwaway repository:
//! - false when the request has no snapshot ref, and true right after a snapshot, without appending to the op log:
//!   tests/snapshot/lifecycle.rs::first_snapshot_saves_real_tree_parent_trailers_ref_and_attributed_op;
//! - false after a file in the folder changes:
//!   lifecycle.rs::unchanged_writes_nothing_then_changed_snapshot_chains_head_and_previous;
//! - true when the only change since the snapshot is a new file over the cap:
//!   tests/snapshot/caps.rs::refused_only_change_is_saved_and_unchanged_outcome_still_reports_the_refusal, and when
//!   that over-cap file was renamed since the snapshot:
//!   caps.rs::new_against_head_file_that_grows_or_is_renamed_over_cap_drops_from_next_snapshot.
//! - false for an attempt that has no snapshot, true after a snapshot of that attempt, reading only that attempt's
//!   trysnap ref: tests/snapshot/attempts.rs::attempts_have_independent_snapshot_chains_and_saved_reads_only_the_selected_ref.

use crate::saved_state::saved_state;
use crate::{SnapshotError, SnapshotTarget};
use gitio::Git;
use std::path::Path;

/// saved_state's `saved`: true when `folder`'s tree, built as snapshot builds it, equals the tree of the target's
/// snapshot ref; false when it differs or there is no snapshot yet. The caller passes the excludes and the size cap
/// it snapshots with; any other pair can make the trees differ.
/// The cap leaves new files over it out of the tree, so a folder whose only
/// change is such a file reads saved. The bool alone does not show the cap's refusals: a caller that must know them
/// (such as the cli's folder remove, which passes them to folders' remove) reads saved_state.
/// Writes tree objects (folder_tree does, through gitio's snapshot_tree) but moves no ref and appends no op record.
pub fn is_saved(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<bool, SnapshotError> {
    saved_state(git, folder, target, extra_excludes, size_cap).map(|s| s.saved)
}
