//! decide_landing: whether land repeats, resumes, appends, starts or waits, from the tips and the landing state.
//! decide_landing_tests covers every branch.

use crate::landing::{LandingState, LandingStep};
use gitio::Oid;

/// `state` is already resolved against `integration` (resolve_landing_state). In this order: a member's squash is
/// `squash` and integration is at the batch tip (the newest member's integration): Landed with that member's
/// integration (a repeat; moves nothing). The newest member is `squash` and integration is where it stood before
/// that member (origin, with the state started at origin, for a batch of one; else the member before its
/// integration): Resume from there to its integration (a land killed between writing the state and its op lands the
/// same commit). Integration is at the batch tip and the tip is not origin: Append onto it (a published batch whose
/// state was not yet removed does not take appends). Integration is at origin: Fresh; a state left by a landing that
/// never moved integration, or by a publish killed before its clean-up, does not block, because the caller holds
/// lock `land`. Otherwise Busy with the newest member's change ID, or NoState without a state.
pub(crate) fn decide_landing(
    origin: &Oid,
    integration: &Oid,
    state: Option<&LandingState>,
    squash: &Oid,
) -> LandingStep {
    let Some(state) = state else {
        return if integration == origin { LandingStep::Fresh } else { LandingStep::NoState };
    };
    let at_tip = integration == &state.newest.integration;
    if let Some(member) = state.members().find(|member| &member.squash == squash).filter(|_| at_tip) {
        return LandingStep::Landed(member.integration.clone());
    }
    if &state.newest.squash == squash && before_newest(state, origin) == Some(integration) {
        return LandingStep::Resume { from: integration.clone(), to: state.newest.integration.clone() };
    }
    if at_tip && integration != origin {
        return LandingStep::Append { onto: integration.clone() };
    }
    if integration == origin {
        return LandingStep::Fresh;
    }
    LandingStep::Busy(state.newest.change_id.clone())
}

/// Where integration stood before the newest member landed: the member before its integration, or for a batch of
/// one origin, when the state started at it (a state from an older origin has no such point).
fn before_newest<'a>(state: &'a LandingState, origin: &'a Oid) -> Option<&'a Oid> {
    match state.earlier.last() {
        Some(member) => Some(&member.integration),
        None => (&state.origin_at_start == origin).then_some(origin),
    }
}
