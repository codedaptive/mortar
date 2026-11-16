//! save: commit a folder's files on its branch, with the request's change ID, and set its index to them. The index
//! step runs with no GIT_INDEX_FILE from the caller's env (real_index_git), so it sets the folder's own index.
//! No unit test: it runs git and appends to the op log, and holds no decision of its own (S-5); save_message,
//! save_head, save_unchanged, save_change_id, save_trailers, save_op, save_from_apply and real_index_git are its
//! decisions and have their tests. The commit's parents are always [HEAD], so there is no parent decision.
//! Nagatha's integration tests, in a throwaway repository, cover these cases:
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
//!   an unchanged one in save_repair.rs::save_after_real_index_not_set_repairs_stale_real_index_with_no_commit_ref_or_log_movement.
//! Not yet covered by an integration test: IndexNotSet from an unchanged save, and OpRepeated or ChangeId through save.

use crate::real_index_git::real_index_git;
use crate::save_change_id::save_change_id;
use crate::save_from_apply::save_from_apply;
use crate::save_head::save_head;
use crate::save_message::save_message;
use crate::save_op::save_op;
use crate::save_trailers::save_trailers;
use crate::save_unchanged::save_unchanged;
use crate::snapshot::snapshot;
use crate::{SaveOutcome, SnapshotError, SnapshotTarget};
use gitio::Git;
use oplog::{apply, new_change_id, new_op_id, read_mortar_trailers, unix_now};
use std::path::Path;

/// Saves `folder`'s files as a commit on its branch: `sentence` is the subject, the parent is HEAD, and the
/// trailers are Mortar-Change-Id, Mortar-Request and Mortar-Zone. The change ID is HEAD's when HEAD's
/// Mortar-Request and Mortar-Zone are `target`'s (S-34), else a new one. The tree is snapshot's, with the same
/// built-in excludes, `extra_excludes` and `size_cap`; its refused list is passed through. `git` gives the binary
/// and environment; every run is in `folder`, which must be the folder's top.
///
/// The snapshot step is the first that writes, and moves the target's snapshot ref as snapshot does (when the
/// tree differs from the last snapshot), even when save then writes no commit. A tree equal to HEAD's writes no
/// commit and leaves the branch untouched: written false, HEAD as found, change_id None, and the folder's index set
/// to HEAD's tree (S-41), so a save after IndexNotSet repairs the index. Otherwise the branch moves through one op
/// log entry and the folder's index is set to the saved tree. Either index step runs with every GIT_INDEX_FILE
/// entry removed from `git`'s env, so it sets the folder's own index. The folder's files are never touched.
///
/// Errors: BadSentence before anything is read or written; UnbornHead and DetachedHead before the snapshot runs;
/// snapshot's own errors; then, after the snapshot step, with no commit written and no branch moved: Trailers when
/// HEAD's trailers cannot be read, ChangeIdMissing when HEAD is this request's and zone's but has no change ID,
/// ChangeId when a new ID cannot be drawn; BranchMoved when the branch was not at the HEAD read, and OpRepeated when
/// the op key was already done (in both the commit was written and the branch did not move); IndexNotSet when the
/// index could not be set, after the branch moved (a written save) or with the branch at HEAD (an unchanged one);
/// Git when another git run fails, and Oplog when apply does. The caller holds the lock and runs oplog::recover
/// (S-25).
pub fn save(
    git: &Git,
    folder: &Path,
    target: &SnapshotTarget,
    sentence: &str,
    extra_excludes: &[String],
    size_cap: u64,
) -> Result<SaveOutcome, SnapshotError> {
    let git = Git { repo: folder.to_path_buf(), ..git.clone() };
    let subject = save_message(sentence)?;
    let (branch, head) = save_head(folder, git.head(folder).map_err(SnapshotError::Git)?)?;
    let snap = snapshot(&git, folder, target, extra_excludes, size_cap)?;
    let info = git.commit_info(&head).map_err(SnapshotError::Git)?;
    if let Some(index_tree) = save_unchanged(&snap.tree, &info.tree) {
        real_index_git(&git)
            .set_index(folder, &index_tree)
            .map_err(|error| SnapshotError::IndexNotSet { commit: head.clone(), error })?;
        return Ok(SaveOutcome {
            commit: head,
            tree: snap.tree,
            written: false,
            change_id: None,
            refused: snap.refused,
        });
    }
    let trailers = read_mortar_trailers(&git, &info.message).map_err(SnapshotError::Trailers)?;
    let change_id = match save_change_id(&head, &trailers, target)? {
        Some(id) => id,
        None => new_change_id().map_err(SnapshotError::ChangeId)?,
    };
    let message = git
        .with_trailers(&subject, &save_trailers(&change_id, target))
        .map_err(SnapshotError::Git)?;
    let commit = git
        .commit_tree(&snap.tree, std::slice::from_ref(&head), &message, None, None)
        .map_err(SnapshotError::Git)?;
    let common = git.common_dir().map_err(SnapshotError::Git)?;
    let op = save_op(&branch, &head, &commit, &target.actor, new_op_id(), unix_now());
    save_from_apply(apply(&git, &common, &op).map_err(SnapshotError::Oplog)?)?;
    real_index_git(&git)
        .set_index(folder, &snap.tree)
        .map_err(|error| SnapshotError::IndexNotSet { commit: commit.clone(), error })?;
    Ok(SaveOutcome { commit, tree: snap.tree, written: true, change_id: Some(change_id), refused: snap.refused })
}
