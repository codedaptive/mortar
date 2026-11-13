//! What recovery makes of an open intent, from where its refs stand.
//! A bare type; decide_recovery_tests and recovery_close_tests exercise each variant.

use gitio::Oid;

use crate::store::RefMove;

/// The verdict on one open intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryDecision {
    /// Every ref whose move changes something is at its new value, or no move changes anything: nothing is
    /// left to do.
    Done,
    /// No ref whose move changes something is at its new value, so nothing moved: each planned move with
    /// the ref's actual value.
    Failed(Vec<(RefMove, Option<Oid>)>),
    /// A ref whose move changes something is at its new value beside another such ref that is not: each
    /// planned move with the ref's actual value.
    Mixed(Vec<(RefMove, Option<Oid>)>),
}
