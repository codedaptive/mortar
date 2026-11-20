//! BackOutStep: what back_out does next, as decide_back_out chooses it. A bare type; decide_back_out_tests
//! exercises each variant.

use crate::landing::LandingState;
use gitio::Oid;

/// Nothing: integration is at origin. NoState: integration is off origin and no landing state explains it.
/// Revert: integration is at the state's commit; move it back to the state's origin. Elsewhere: integration is at
/// neither origin nor `expected`, the state's commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BackOutStep {
    Nothing,
    NoState,
    Revert(LandingState),
    Elsewhere { expected: Oid },
}
