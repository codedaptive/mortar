//! decide_landing: whether land repeats, waits, resumes or starts, from the tips and the landing state.
//! decide_landing_tests covers every branch.

use crate::landing::{LandingState, LandingStep};
use gitio::Oid;

/// In this order: the state names `squash` and integration is at the state's commit: Landed (a repeat; moves
/// nothing). Integration is off origin: Busy with the state's change ID, or NoState without a state. Integration is
/// at origin, the state names `squash` and started at this origin: Resume to the state's commit (a land killed
/// between writing the state and its op lands the same commit). Otherwise Fresh: a state left by a landing that
/// never moved integration does not block, because the caller holds lock `land`.
pub(crate) fn decide_landing(
    origin: &Oid,
    integration: &Oid,
    state: Option<&LandingState>,
    squash: &Oid,
) -> LandingStep {
    let ours = state.filter(|state| &state.squash == squash);
    if let Some(state) = ours.filter(|state| &state.integration == integration) {
        return LandingStep::Landed(state.integration.clone());
    }
    if integration != origin {
        return state.map_or(LandingStep::NoState, |state| LandingStep::Busy(state.change_id.clone()));
    }
    match ours.filter(|state| &state.origin_at_start == origin) {
        Some(state) => LandingStep::Resume(state.integration.clone()),
        None => LandingStep::Fresh,
    }
}
