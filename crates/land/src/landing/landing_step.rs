//! LandingStep: what land does next, as decide_landing chooses it. A bare type; decide_landing_tests exercises
//! each variant.

use gitio::Oid;
use oplog::ChangeId;

/// Landed: this squash is a member of the batch, landed at this commit, its own `integration`; nothing to do.
/// Busy: integration is off origin and off the batch tip; this change ID is the newest member's. NoState: integration
/// is off origin and no landing state explains it. Resume: the newest member is this squash, its state written but
/// integration still at `from`, where it stood before that member; move it to `to`. Append: integration is at the
/// batch tip `onto` and this squash is not a member; land it onto `onto` as the batch's next member. Fresh:
/// integration is at origin; land it as a new batch of one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum LandingStep {
    Landed(Oid),
    Busy(ChangeId),
    NoState,
    Resume { from: Oid, to: Oid },
    Append { onto: Oid },
    Fresh,
}
