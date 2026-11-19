//! check_restore_join_state: the read-only pre-check of restore_join_state's refusals that the cli runs
//! before the cli's pending folder move and oplog undo, which run in that order after it, so a refused undo of a
//! resolve is refused with no ref moved by the undo.
//! restore_join_state keeps its own checks for a change between this check and the restore; the caller holds
//! join-<zone>.
//! No unit test: it runs git and reads a real file. restore_refusal_tests (its rules, with fakes),
//! recorded_join_state_tests and restore_state_step_tests (the decisions it passes in) cover it. No ZIT: any test
//! through it calls gitio for real, which makes it a RIT.
//! RITs (tests/join_rit), each in a throwaway repository: tests/join_rit/precheck.rs covers
//! - the pre-check, then oplog undo of the resolve's final op, then restore_join_state: Restored
//!   (precheck_then_real_undo_and_restore_recreates_the_exact_pause_and_resolves_the_same_tree);
//! - the pre-check while another paused join's state is present: StatePresent, and no ref moved and no file written
//!   (precheck_preserves_another_real_paused_join_and_refuses_before_undo);
//! - the pre-check on a plain join's join-rebase op: NotAResolveOp
//!   (precheck_rejects_a_real_plain_join_record_without_writing_or_moving_anything);
//! - the pre-check with the undone resolve's own state already restored: Ok
//!   (precheck_accepts_its_own_already_restored_pause_and_keeps_every_byte);
//! - the pre-check with the request ref at the op's new tip, before the undo: Ok, never RestoreStale
//!   (precheck_at_the_new_request_tip_is_ok_not_restore_stale_before_any_undo);
//! - the pre-check with a mortar-join.json that does not decode: StateCorrupt, and the file kept
//!   (precheck_keeps_a_corrupt_real_state_file_and_reports_state_corrupt_without_undo);
//!
//! and tests/join_rit/precheck_crash.rs covers
//! - the pre-check with the undone resolve's own finished state left by a kill: Ok, then undo and restore: Restored
//!   (precheck_of_a_killed_resolves_own_finished_state_is_ok_then_undo_and_restore_succeed).

use crate::join_state_path::join_state_path;
use crate::read_join_state::read_join_state;
use crate::recorded_join_state::recorded_join_state;
use crate::restore_refusal::restore_refusal;
use crate::restore_state_step::restore_state_step;
use crate::JoinRunError;
use gitio::Git;
use oplog::ValuedRecord;
use std::path::Path;

/// Ok, or the refusal restore_join_state would give for `op` in `folder` that a ref read cannot change:
/// NotAResolveOp or RecordedStateCorrupt from the record, checked before anything is read; Git when `folder`'s git
/// dir cannot be found; StateRead or StateCorrupt when mortar-join.json cannot be read or decoded; StatePresent when
/// it holds another paused join. Never RestoreStale: before the undo the request ref is at the op's new tip. It
/// writes nothing and reads no ref.
pub fn check_restore_join_state(git: &Git, folder: &Path, op: &ValuedRecord) -> Result<(), JoinRunError> {
    let read_state = || {
        let git_dir = git.git_dir(folder).map_err(JoinRunError::Git)?;
        let path = join_state_path(&git_dir);
        let existing = read_join_state(&path)?;
        Ok((path, existing))
    };
    restore_refusal(op, recorded_join_state, read_state, restore_state_step)
}
