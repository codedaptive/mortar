//! decide_back_out: whether back_out has nothing to do, reverts the landing batch, or refuses. decide_back_out_tests
//! covers every branch; the landing_zit tests batch::back_out_restores_origin_for_every_member and
//! kills::killed_second_member_recovers_then_back_out_discards_both cover it through back_out_batch.

use crate::landing::{BackOutStep, LandingState};
use gitio::Oid;

/// Integration at origin: Nothing. Otherwise no state: NoState. Integration at the batch's tip (its newest member's
/// integration), or at the point before the newest member (the last earlier member's integration: a land of the
/// newest member killed between its state write and its op): Revert with the state, the whole batch. Else Elsewhere
/// naming the batch's tip.
pub(crate) fn decide_back_out(origin: &Oid, integration: &Oid, state: Option<&LandingState>) -> BackOutStep {
    if integration == origin {
        return BackOutStep::Nothing;
    }
    let Some(state) = state else {
        return BackOutStep::NoState;
    };
    let before_newest = state.earlier.last().map(|member| &member.integration);
    if &state.newest.integration == integration || before_newest == Some(integration) {
        return BackOutStep::Revert(state.clone());
    }
    BackOutStep::Elsewhere { expected: state.newest.integration.clone() }
}
