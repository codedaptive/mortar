//! is_saved: whether a folder's files equal the target's last snapshot.
//! No unit test: it only runs git and holds no decision of its own (S-5); decide_snapshot_tests covers the
//! comparison. Nagatha's integration tests cover it, in a throwaway repository:
//! - false when the request has no snapshot ref, and true right after a snapshot, without appending to the op log:
//!   tests/snapshot/lifecycle.rs::first_snapshot_saves_real_tree_parent_trailers_ref_and_attributed_op;
//! - false after a file in the folder changes:
//!   lifecycle.rs::unchanged_writes_nothing_then_changed_snapshot_chains_head_and_previous;
//! - true when the only change since the snapshot is a new file over the cap:
//!   tests/snapshot/caps.rs::refused_only_change_is_saved_and_unchanged_outcome_still_reports_the_refusal, and when
//!   that over-cap file was renamed since the snapshot:
//!   caps.rs::new_against_head_file_that_grows_or_is_renamed_over_cap_drops_from_next_snapshot.
//! Not yet covered by an integration test: is_saved reading an attempt's trysnap ref (Nagatha's).

use crate::decide_snapshot::decide_snapshot;
use crate::folder_tree::folder_tree;
use crate::read_previous_snapshot::read_previous_snapshot;
use crate::snapshot_decision::SnapshotDecision;
use crate::snapshot_ref_name::snapshot_ref_name;
use crate::{SnapshotError, SnapshotTarget};
use gitio::Git;
use std::path::Path;

/// True when `folder`'s tree, built as snapshot builds it, equals the tree of the target's snapshot ref
/// (snapshot_ref_name, as snapshot); false when it differs or there is no snapshot yet. `git` gives the binary and
/// environment; every run is in `folder`, which must be the folder's top. The caller passes the excludes and the
/// size cap it snapshots with, or the trees differ. The cap leaves new files over it out of the tree, so a folder
/// whose only change is such a file reads saved.
/// Writes tree objects (snapshot_tree does) but moves no ref and appends no op record.
pub fn is_saved(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<bool, SnapshotError> {
    let git = Git { repo: folder.to_path_buf(), ..git.clone() };
    let refname = snapshot_ref_name(target)?;
    let (_, tree, _) = folder_tree(&git, folder, extra_excludes, size_cap)?;
    let previous = read_previous_snapshot(&git, &refname)?;
    Ok(matches!(decide_snapshot(&tree, previous.as_ref()), SnapshotDecision::Unchanged(_)))
}
