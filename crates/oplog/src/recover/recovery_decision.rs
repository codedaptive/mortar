//! What recovery makes of an open intent, from where its refs stand.
//! A bare type; decide_recovery_tests and recovery_close_tests exercise each variant.

use gitio::Oid;

use crate::store::RefMove;

/// The verdict on one open intent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryDecision {
    /// Every ref is at its new value: the transaction ran.
    Done,
    /// Every ref is at its old value: nothing moved.
    Failed,
    /// The refs are neither all at new nor all at old: each planned move with the ref's actual value.
    Mixed(Vec<(RefMove, Option<Oid>)>),
}
