//! decide_restore: the restore step from the state file already present, the state a resolve's final op recorded
//! and where its request ref is now. decide_restore_tests covers each rule and the precedence between them.

use crate::join_state::JoinState;
use crate::restore_step::RestoreStep;
use gitio::Oid;

/// The rules, in this order: the request ref anywhere but the recorded state's request tip -> Stale; no state file
/// -> Write; a state file equal to the recorded state -> AlreadyRestored (a repeat); a state file equal to the
/// recorded state once its rebased_to and folder_tree are cleared -> Write; any other state file -> StatePresent.
/// Why a file with rebased_to and folder_tree set is overwritten: it is the undone resolve's own state, left by a kill
/// after its final op and before the file's removal (S-87); kept, a resolve would read rebased_to and redo the move
/// the undo reversed. Any other paused join is never overwritten.
pub(crate) fn decide_restore(
    existing: Option<&JoinState>,
    recorded: &JoinState,
    request_now: Option<&Oid>,
) -> RestoreStep {
    if request_now != Some(&recorded.request_tip) {
        return RestoreStep::Stale { expected: recorded.request_tip.clone(), actual: request_now.cloned() };
    }
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
