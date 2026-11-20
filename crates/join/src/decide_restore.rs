//! decide_restore: the restore step from the state file already present, the state a resolve's final op recorded
//! and where its request ref is now. It decides the ref step and hands the rest to restore_state_step.
//! decide_restore_tests covers each rule and the precedence between them.

use crate::join_state::JoinState;
use crate::restore_state_step::restore_state_step;
use crate::restore_step::RestoreStep;
use gitio::Oid;

/// The request ref anywhere but the recorded state's request tip -> Stale, whatever the state file holds; else the
/// step restore_state_step chooses from the state file (Write, AlreadyRestored or StatePresent; it says why the undone
/// resolve's own finished state is overwritten).
pub(crate) fn decide_restore(
    existing: Option<&JoinState>,
    recorded: &JoinState,
    request_now: Option<&Oid>,
) -> RestoreStep {
    if request_now != Some(&recorded.request_tip) {
        return RestoreStep::Stale { expected: recorded.request_tip.clone(), actual: request_now.cloned() };
    }
    restore_state_step(existing, recorded)
}
