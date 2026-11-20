//! save_from_formatted: the one snapshot and save of a folder, with the formatter when one is given, shared by mr join
//! and mr save (save calls it only when a formatter is configured).
//! No unit test: it runs git and the formatter and appends to the op log, and its own decisions (HEAD with no commit is
//! UnbornHead; the flag of whether the formatter's snapshot moved the ref) are covered by the integration tests in
//! crates/cli/tests/save_join_runs, with the mortar binary as a child process in a throwaway repository:
//! - a formatter that commits on the request branch, then join saves on that tip and fast-forwards the zone:
//!   joins.rs::join_saves_on_branch_moved_by_formatter_then_fast_forwards_zone;
//! - the same through save, the formatter's commit the save's parent:
//!   saves.rs::save_uses_request_branch_tip_moved_by_formatter_as_parent;
//! - a detached HEAD with a formatter keeps the edit in the snapshot, is refused naming the moved snapshot ref and
//!   moves no request branch:
//!   saves.rs::detached_head_save_with_formatter_preserves_edit_in_snapshot_without_moving_request_branch;
//! - HEAD with no commit is refused and moves no ref:
//!   joins.rs::unborn_head_join_is_refused_without_moving_any_refs.

use std::path::Path;

use gitio::Git;
use snapshot::{
    save_from_snapshot, snapshot_formatted, Formatter, FormatterRun, SavedWithSnapshot, SnapshotError,
    SnapshotTarget,
};

/// Runs `formatter` (None runs none) with a snapshot (snapshot_formatted), reads HEAD, then saves from that snapshot
/// (save_from_snapshot, which takes no second one) with `sentence`, `excludes` and `cap`. HEAD is read after the
/// formatter because the formatter may move the branch and snapshot_formatted builds its tree on HEAD as the
/// formatter left it; the caller holds the lock, so nothing runs between the snapshot and the read.
///
/// Answers the save, the formatter's run (None when nothing ran) and whether the snapshot moved the snapshot ref.
/// An error carries that same flag for the steps before it: false when the snapshot itself fails. HEAD with no
/// commit is UnbornHead; a git error reading HEAD is Git.
pub(crate) fn save_from_formatted(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    excludes: &[String],
    cap: u64,
    formatter: Option<&Formatter>,
) -> Result<(SavedWithSnapshot, Option<FormatterRun>, bool), (SnapshotError, bool)> {
    let mut taken = snapshot_formatted(git, folder, target, excludes, cap, formatter).map_err(|error| (error, false))?;
    let written = taken.written;
    let formatted = taken.formatted.take();
    let (_, head) = git.head(folder).map_err(|error| (SnapshotError::Git(error), written))?;
    let Some(built_on) = head else {
        return Err((SnapshotError::UnbornHead { folder: folder.to_path_buf() }, written));
    };
    let saved =
        save_from_snapshot(git, folder, target, sentence, &built_on, taken).map_err(|error| (error, written))?;
    Ok((saved, formatted, written))
}
