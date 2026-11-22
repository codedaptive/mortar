//! SnapshotError: why a snapshot or a save did not happen. A declaration only: its Display is snapshot_error_display,
//! snapshot_error_display_tests exercises the text that shows no gitio or oplog Display, and
//! snapshot_error_display_inner_tests the text that shows one (an Oid, or a gitio or oplog error).
//! snapshot_ref_name_tests exercises BadTarget, snapshot_from_apply_tests OpRepeated and RefMoved,
//! capped_excludes_tests PathNotUtf8, save_head_tests UnbornHead and DetachedHead, save_message_tests BadSentence,
//! save_change_id_tests ChangeIdMissing, and
//! save_from_apply_tests OpRepeated and BranchMoved. The integration tests get, through snapshot: RefMoved in
//! tests/snapshot/race.rs::foreign_logged_move_after_previous_read_is_refused_without_overwriting_ref_or_files;
//! FileSize (an ELOOP) in
//! tests/snapshot/sizing_edges.rs::real_metadata_symlink_loop_is_file_size_error_before_commit_ref_or_log_and_retry_preserves_work;
//! CapMismatch, also through saved_state, in
//! sizing_edges.rs::caller_noglob_environment_produces_exact_cap_mismatch_without_losing_previous_snapshot;
//! UnbornHead, also through saved_state, in
//! sizing_edges.rs::unborn_head_is_named_before_any_snapshot_commit_ref_or_op.
//! Through save, in tests/snapshot/save_cases.rs: BadSentence (blank_and_multiline_sentences_refuse_before_snapshot_or_any_history_write),
//! DetachedHead and UnbornHead (detached_and_unborn_heads_are_named_before_any_snapshot_commit_ref_or_op_is_written),
//! ChangeIdMissing (this_requests_head_without_change_id_refuses_after_snapshot_without_saving_a_branch_commit),
//! Trailers from a malformed Mortar-Change-Id (malformed_change_id_in_real_head_trailers_is_typed_and_preserves_branch_index_and_work),
//! IndexNotSet (real_index_lock_produces_index_not_set_after_logged_branch_move_without_touching_files_or_old_index and
//! tests/snapshot/save_repair.rs::save_after_real_index_not_set_repairs_stale_real_index_with_no_commit_ref_or_log_movement);
//! BranchMoved in
//! tests/snapshot/save_race.rs::branch_moved_during_save_is_named_without_overwriting_foreign_head_index_work_or_caller_lock.
//! Through save_from_snapshot, BranchMoved is in tests/snapshot/save_from_snapshot_cases.rs:
//! moved_branch_refuses_changed_snapshot_and_preserves_foreign_commit_and_files and
//! moved_branch_refuses_unchanged_snapshot_without_resetting_index.
//! ChangeId comes only from new_change_id's I/O in save and no integration test reaches it there (its Display is in
//! snapshot_error_display_inner_tests.rs::change_id_variant_keeps_oplogs_quoted_bad_text_and_reason_after_snapshot_prefix);
//! PathNotUtf8 and OpRepeated have no integration test.
//! BadTarget (attempt 0, through snapshot and saved_state) is in
//! tests/snapshot/saved_state_cases.rs::attempt_zero_is_bad_target_for_snapshot_and_saved_state_before_any_ref_log_or_file_write.

use gitio::Oid;
use std::path::PathBuf;

/// Derives Debug only, since oplog's OplogError is neither Clone nor Eq; its Display is in snapshot_error_display.rs.
#[derive(Debug)]
pub enum SnapshotError {
    /// A git run failed.
    Git(gitio::GitError),
    /// The op log refused.
    Oplog(oplog::OplogError),
    /// The folder's HEAD has no commit yet, so there is nothing to start the tree from or to save onto.
    UnbornHead { folder: PathBuf },
    /// The folder's HEAD names no branch, so save has no branch to move.
    DetachedHead { folder: PathBuf },
    /// A zone, request or attempt that cannot form the ref name; the reason names it.
    BadTarget { reason: String },
    /// The sentence cannot be a commit subject; the reason says why.
    BadSentence { reason: String },
    /// HEAD's Mortar-Request and Mortar-Zone are the target's but it carries no Mortar-Change-Id; save writes
    /// no commit and moves no branch rather than make a second ID for the request. Its snapshot step has already
    /// run, so the target's snapshot ref may have moved.
    ChangeIdMissing { head: Oid },
    /// HEAD's message trailers could not be read.
    Trailers(oplog::MortarTrailerError),
    /// A new change ID could not be drawn.
    ChangeId(oplog::ChangeIdError),
    /// The snapshot ref was not at the value read before the move, so nothing moved; the failed record names the
    /// ref and its actual value.
    RefMoved(oplog::OpRecord),
    /// Save's branch was not at the HEAD commit read, so nothing moved; the failed record names the ref and its
    /// actual value.
    BranchMoved(oplog::OpRecord),
    /// The op log already held a done record for this op's key, so nothing moved and the ref was not checked; the
    /// record is that earlier done record. With a key made from a fresh op id this means an op id was reused.
    OpRepeated(oplog::OpRecord),
    /// Setting the folder's index failed; `commit` is the branch's commit (the new commit on a written save, HEAD on
    /// an unchanged one). The files are untouched. The index is the old one, unless gitio's set_index failed at its
    /// sparse-checkout reapply, after read-tree had already set it: then it holds the commit's tree without its
    /// skip-worktree bits. Either way a save again finds the tree equal to HEAD's and sets the index to it.
    IndexNotSet { commit: Oid, error: gitio::GitError },
    /// A changed file's size could not be read, so the cap could not be checked; no commit was written.
    FileSize { path: PathBuf, error: std::io::Error },
    /// A refused path is not UTF-8, so it cannot be passed to git as an exclude; no commit was written.
    PathNotUtf8 { path: PathBuf },
    /// The capped tree did not differ from the full tree by exactly the refused files: git matched an exclude
    /// differently (e.g. GIT_NOGLOB_PATHSPECS or GIT_LITERAL_PATHSPECS set), or the folder changed between the two
    /// tree builds; no commit was written. The paths are
    /// the refused files still in the tree, then the other files left out of it.
    CapMismatch { paths: Vec<PathBuf> },
}
