//! save_with_snapshot: commit a folder's files on its branch, with the request's change ID, set its index to them,
//! and say whether the snapshot step moved the snapshot ref. save is this without that fact. The index step runs
//! with no GIT_INDEX_FILE from the caller's env (real_index_git), so it sets the folder's own index.
//! It runs no commit hooks; save_with_hooks is its twin that does. Its body is save_through with a hook step that
//! keeps the message: it reads the sentence and HEAD (save_message, save_head), takes the snapshot, and hands the
//! rest to save_snapshot_onto with HEAD as the commit the snapshot was built on.
//! No unit test: it runs git and holds no decision of its own; save_message and save_head are its decisions, and
//! save_snapshot_onto's (save_unchanged, save_change_id, save_trailers, save_op, save_from_apply,
//! saved_with_snapshot_from and real_index_git) are its decisions through it; all have their tests.
//! The commit's parents are always [HEAD], so there is no parent decision.
//! The integration tests in tests/snapshot, in a throwaway repository, run these cases through save, which calls
//! save_with_snapshot:
//! - a save leaves `git status` clean, in
//!   tests/snapshot/save_cases.rs::save_on_zone_tip_draws_a_new_id_leaves_status_clean_and_next_save_reuses_the_requests_id
//!   and, for the retry after IndexNotSet, in
//!   tests/snapshot/save_repair.rs::save_after_real_index_not_set_repairs_stale_real_index_with_no_commit_ref_or_log_movement;
//! - a folder unchanged from HEAD writes no commit, leaves the branch alone and keeps the index at HEAD's tree and
//!   entries: save_cases.rs::unchanged_save_keeps_index_at_heads_tree_and_entries_and_repeat_writes_no_history;
//! - a save after IndexNotSet sets the stale index to HEAD's tree, with no commit, ref or log movement:
//!   save_repair.rs::save_after_real_index_not_set_repairs_stale_real_index_with_no_commit_ref_or_log_movement;
//! - a first save on a zone tip draws a new change ID and a second save on the request reuses it:
//!   save_cases.rs::save_on_zone_tip_draws_a_new_id_leaves_status_clean_and_next_save_reuses_the_requests_id;
//! - a detached or unborn HEAD is refused with nothing written:
//!   save_cases.rs::detached_and_unborn_heads_are_named_before_any_snapshot_commit_ref_or_op_is_written;
//! - a branch moved under save gives BranchMoved:
//!   tests/snapshot/save_race.rs::branch_moved_during_save_is_named_without_overwriting_foreign_head_index_work_or_caller_lock;
//! - IndexNotSet from a locked index after the branch moved leaves the files and the old index alone:
//!   save_cases.rs::real_index_lock_produces_index_not_set_after_logged_branch_move_without_touching_files_or_old_index;
//! - the index step leaves a caller's GIT_INDEX_FILE untouched, after a written save in
//!   save_cases.rs::save_on_zone_tip_draws_a_new_id_leaves_status_clean_and_next_save_reuses_the_requests_id and after
//!   an unchanged one in save_repair.rs::save_after_real_index_not_set_repairs_stale_real_index_with_no_commit_ref_or_log_movement;
//! - a save with a refusing pre-commit installed still writes its commit and moves the branch:
//!   save_cases.rs::hooks_hookless_save_writes_its_commit_with_a_refusing_pre_commit_installed.
//!
//! The other hooks_ cases in save_cases.rs run save_with_hooks and are listed in save_with_hooks.rs.
//!
//! Not yet covered by an integration test: IndexNotSet from an unchanged save, OpRepeated or ChangeId through save,
//! and snapshot_written from save_with_snapshot (save_from_snapshot's false is covered in
//! tests/snapshot/save_from_snapshot_cases.rs).

use crate::save_through::save_through;
use crate::{SavedWithSnapshot, SnapshotError, SnapshotTarget};
use gitio::Git;
use std::path::Path;

/// Saves `folder`'s files as a commit on its branch: `sentence` is the subject, the parent is HEAD, and the
/// trailers are Mortar-Change-Id, Mortar-Request and Mortar-Zone. The change ID is HEAD's when HEAD's
/// Mortar-Request and Mortar-Zone are `target`'s, else a new one. The tree is snapshot's, with the same
/// built-in excludes, `extra_excludes` and `size_cap`; its refused list is passed through. `git` gives the binary
/// and environment; every run is in `folder`, which must be the folder's top.
///
/// The snapshot step is the first that writes, and moves the target's snapshot ref as snapshot does (when the
/// tree differs from the last snapshot), even when save then writes no commit; `snapshot_written` says whether it
/// did. A tree equal to HEAD's writes no commit and leaves the branch untouched: written false, HEAD as found,
/// change_id None, and the folder's index set to HEAD's tree, so a save after IndexNotSet repairs the index.
/// Otherwise the branch moves through one op log entry and the folder's index is set to the saved tree. Either
/// index step runs with every GIT_INDEX_FILE entry removed from `git`'s env, so it sets the folder's own index. The
/// folder's files are never touched. No commit hooks run.
///
/// Errors: BadSentence before anything is read or written; UnbornHead and DetachedHead before the snapshot runs;
/// snapshot's own errors; then, after the snapshot step, with no commit written and no branch moved: Trailers when
/// HEAD's trailers cannot be read, ChangeIdMissing when HEAD is this request's and zone's but has no change ID,
/// ChangeId when a new ID cannot be drawn; BranchMoved when the branch was not at the HEAD read, and OpRepeated when
/// the op key was already done (in both the commit was written and the branch did not move); IndexNotSet when the
/// index could not be set, after the branch moved (a written save) or with the branch at HEAD (an unchanged one);
/// Git when another git run fails, and Oplog when apply does. The caller holds the lock and runs oplog::recover.
pub fn save_with_snapshot(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<SavedWithSnapshot, SnapshotError> {
    save_through(git, folder, target, sentence, extra_excludes, size_cap, |_, _, message, _| Ok(message))
}
