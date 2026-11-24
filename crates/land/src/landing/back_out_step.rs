//! BackOutStep: what back_out does next, as decide_back_out chooses it. A bare type; decide_back_out_tests
//! exercises each variant, and the landing_zit tests batch::back_out_restores_origin_for_every_member and
//! kills::killed_second_member_recovers_then_back_out_discards_both a batch Revert.

use crate::landing::LandingState;
use gitio::Oid;

/// Nothing: integration is at origin. NoState: integration is off origin and no landing state explains it.
/// Revert: integration is at the batch's tip or at the point before its newest member; move it back to the state's
/// origin, every member with it. Elsewhere: integration is at none of those; `expected` is the batch's tip.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum BackOutStep {
    Nothing,
    NoState,
    Revert(LandingState),
    Elsewhere { expected: Oid },
}
