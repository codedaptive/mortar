//! decide_back_out: whether back_out has nothing to do, reverts the landing, or refuses. decide_back_out_tests
//! covers every branch.

use crate::landing::{BackOutStep, LandingState};
use gitio::Oid;

/// Integration at origin: Nothing. Otherwise no state: NoState; integration at the state's commit: Revert with the
/// state; else Elsewhere naming the state's commit.
pub(crate) fn decide_back_out(origin: &Oid, integration: &Oid, state: Option<&LandingState>) -> BackOutStep {
    if integration == origin {
        return BackOutStep::Nothing;
    }
    match state {
        None => BackOutStep::NoState,
        Some(state) if &state.integration == integration => BackOutStep::Revert(state.clone()),
        Some(state) => BackOutStep::Elsewhere { expected: state.integration.clone() },
    }
}
