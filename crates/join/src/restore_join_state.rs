//! restore_join_state: puts back the paused join a resolve's final op finished, so a later resolve continues from it
//! (S-142(1)). It takes no lock and runs no op: the caller holds join-<zone> (S-47, S-16).
//! No unit test: it reads a ref and reads and writes a real file (S-5); decide_restore_tests and
//! recorded_join_state_tests cover its decisions.
//! RITs owed (Nagatha's join_rit, Orc #2414, S-135), in a throwaway repository:
//! - a resolve to a clean finish, oplog undo of its final op, then restore: Restored, mortar-join.json byte-identical
//!   to the text before that final resolve (plus the newline encode_join_state writes), and resolve again with the
//!   same tree answers Rebased with the same folder_tree;
//! - a repeat restore answers AlreadyRestored and leaves the file unchanged;
//! - a restore while another paused join's state is present is StatePresent, the file unchanged;
//! - a restore before the op is undone (request ref at the op's new tip) is RestoreStale, no file written;
//! - a kill after the final op and before the state's removal, then undo, then restore: Restored over the leftover,
//!   and resolve again finishes;
//! - a plain join's join-rebase op is NotAResolveOp, no file written.

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
/// order (S-142(1)): undo the op's ref move through oplog, then this, then `mr resolve` continues the join; a
/// repeat of either answers done (AlreadyRestored here). The op must be one (NotAResolveOp) and its state must decode
/// and match it (RecordedStateCorrupt); the request ref must be back at the state's request tip (RestoreStale); a
/// state file holding another paused join is kept (StatePresent), and one holding the undone resolve's own finished
/// state is overwritten (decide_restore says why). A state file that cannot be read is StateRead or StateCorrupt.
/// Every refusal writes nothing.
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
