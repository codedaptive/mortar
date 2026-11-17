//! check_restore_join_state: the read-only pre-check of restore_join_state's refusals (S-148) that the cli runs
//! before oplog undo moves anything, so a refused undo of a resolve is refused with nothing moved (S-142(1)).
//! restore_join_state keeps its own checks for a change between this check and the restore (S-148 (1)); the caller
//! holds join-<zone> (S-16).
//! No unit test: it runs git and reads a real file (S-5, S-104). restore_refusal_tests (its rules, with fakes),
//! recorded_join_state_tests and restore_state_step_tests (the decisions it passes in) cover it. No ZIT: any test
//! through it calls gitio for real, which makes it a RIT (S-135).
//! RITs owed (Nagatha's join_rit, S-148 (2)), in a throwaway repository:
//! - the pre-check, then oplog undo of the resolve's final op, then restore_join_state: Restored;
//! - the pre-check while another paused join's state is present: StatePresent, and no ref moved and no file written;
//! - the pre-check on a plain join's join-rebase op: NotAResolveOp;
//! - the pre-check with the undone resolve's own state already restored: Ok;
//! - the pre-check with the request ref at the op's new tip, before the undo: Ok, never RestoreStale;
//! - the pre-check with the undone resolve's own finished state left by a kill: Ok, then undo and restore: Restored;
//! - the pre-check with a mortar-join.json that does not decode: StateCorrupt, and the file kept.

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
