//! restore_state_step: the restore step from the state file already present and the state a resolve's final op
//! recorded, the request ref aside. restore_state_step_tests covers each rule.

use crate::join_state::JoinState;
use crate::restore_step::RestoreStep;

/// The rules, in this order: no state file -> Write; a state file equal to the recorded state -> AlreadyRestored (a
/// repeat); a state file equal to the recorded state once its rebased_to and folder_tree are cleared -> Write; any
/// other state file -> StatePresent. It never answers Stale.
/// Why a file with rebased_to and folder_tree set is overwritten: it is the undone resolve's own state, left by a kill
/// after its final op and before the file's removal (S-87); kept, a resolve would read rebased_to and redo the move
/// the undo reversed. Any other paused join is never overwritten.
pub(crate) fn restore_state_step(existing: Option<&JoinState>, recorded: &JoinState) -> RestoreStep {
    let Some(existing) = existing else {
        return RestoreStep::Write;
    };
    if existing == recorded {
        return RestoreStep::AlreadyRestored;
    }
    let cleared = JoinState { rebased_to: None, folder_tree: None, ..existing.clone() };
    if &cleared == recorded {
        RestoreStep::Write
    } else {
        RestoreStep::StatePresent
    }
}
