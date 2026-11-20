//! restore_join_state: puts back the paused join a resolve's final op finished, so a later resolve continues from it.
//! It takes no lock and runs no op: the caller holds join-<zone>.
//! No unit test: it reads a ref and reads and writes a real file; decide_restore_tests,
//! restore_state_step_tests and recorded_join_state_tests cover its decisions.
//! RITs (tests/join_rit), each in a throwaway repository: tests/join_rit/restore.rs covers
//! - a resolve to a clean finish, oplog undo of its final op, then restore: Restored, mortar-join.json byte-identical
//!   to the text before that final resolve (plus the newline encode_join_state writes), and resolve again with the
//!   same tree answers Rebased with the same folder_tree
//!   (restore_after_real_resolve_and_oplog_undo_restores_exact_pause_then_resolves_the_same_tree);
//! - a repeat restore answers AlreadyRestored and leaves the file unchanged
//!   (restore_repeat_answers_already_restored_without_changing_state_refs_log_index_or_files);
//! - a restore while another paused join's state is present is StatePresent, the file unchanged
//!   (restore_preserves_another_real_paused_join_and_returns_state_present);
//! - a restore before the op is undone (request ref at the op's new tip) is RestoreStale, no file written
//!   (restore_before_undo_refuses_at_the_new_tip_without_creating_state_or_changing_anything);
//! - a plain join's join-rebase op is NotAResolveOp, no file written
//!   (restore_rejects_a_real_plain_join_rebase_done_record_without_writing_state);
//!
//! and tests/join_rit/restore_crash.rs covers
//! - a kill after the final op and before the state's removal, then undo, then restore: Restored over the leftover,
//!   and resolve again finishes
//!   (restore_after_killed_resolve_and_real_undo_overwrites_only_that_resolves_finished_state).
//!
//! and tests/join_rit/precheck.rs covers
//! - a check_restore_join_state that answered Ok, then oplog undo of the op, then restore: Restored
//!   (precheck_then_real_undo_and_restore_recreates_the_exact_pause_and_resolves_the_same_tree).

use crate::decide_restore::decide_restore;
use crate::join_state_path::join_state_path;
use crate::read_join_state::read_join_state;
use crate::recorded_join_state::recorded_join_state;
use crate::restore_step::RestoreStep;
use crate::save_join_state::save_join_state;
use crate::{JoinRunError, PausedJoin, RestoredJoin};
use gitio::Git;
use oplog::ValuedRecord;
use std::path::Path;

/// Writes mortar-join.json in `folder`'s git dir with the state `op`, a resolve's final join-rebase op as read back
/// from the op log, recorded under JOIN_STATE: the state that file held before that resolve's final run. The cli's
/// order: check_restore_join_state, then finish the pending folder move (which snapshots the folder first), then
/// undo the op's ref move through oplog, then this, then `mr resolve` continues the join; a repeat of the undo or of
/// this restore answers done (AlreadyRestored here). The op must be one (NotAResolveOp) and its state must decode and
/// match it (RecordedStateCorrupt); the request ref must be back at the state's request tip (RestoreStale); a state
/// file holding another paused join is kept (StatePresent), and one holding the undone resolve's own finished state is
/// overwritten (restore_state_step says why). A state file that cannot be read is StateRead or StateCorrupt. Every
/// refusal before the write step (save_join_state) writes nothing; a StateWrite refusal, raised during the write, can
/// leave `mortar-join.json.tmp` in the git dir.
pub fn restore_join_state(git: &Git, folder: &Path, op: &ValuedRecord) -> Result<RestoredJoin, JoinRunError> {
    let recorded = recorded_join_state(op)?;
    let git_dir = git.git_dir(folder).map_err(JoinRunError::Git)?;
    let path = join_state_path(&git_dir);
    let request_now = git.read_ref(&recorded.request_ref).map_err(JoinRunError::Git)?;
    let existing = read_join_state(&path)?;
    let paused = PausedJoin { commit: recorded.commit.clone(), files: recorded.files.clone() };
    match decide_restore(existing.as_ref(), &recorded, request_now.as_ref()) {
        RestoreStep::Write => {
            save_join_state(&git_dir, &recorded)?;
            Ok(RestoredJoin::Restored { state: path, paused })
        }
        RestoreStep::AlreadyRestored => Ok(RestoredJoin::AlreadyRestored { state: path, paused }),
        RestoreStep::Stale { expected, actual } => {
            Err(JoinRunError::RestoreStale { request_ref: recorded.request_ref, expected, actual })
        }
        RestoreStep::StatePresent => Err(JoinRunError::StatePresent { state: path }),
    }
}
