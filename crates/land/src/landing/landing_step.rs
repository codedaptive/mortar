//! LandingStep: what land does next, as decide_landing chooses it. A bare type; decide_landing_tests exercises
//! each variant.

use gitio::Oid;
use oplog::ChangeId;

/// Landed: this squash is already landed at this commit; nothing to do. Busy: another landing, this change ID's,
/// is in flight. NoState: integration is off origin and no landing state explains it. Resume: the state for this
/// squash was written but integration never moved; move it to this commit. Fresh: build the landing commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LandingStep {
    Landed(Oid),
    Busy(ChangeId),
    NoState,
    Resume(Oid),
    Fresh,
}
