//! The recovery verdict on an open intent, from each planned move and the ref's actual value.

use gitio::Oid;

use super::RecoveryDecision;
use crate::store::{move_changes, oid_is_zero, RefMove};

/// Decided over the moves that change something (move_changes) only: Done when every changing move's ref is at new
/// (so an op with no changing move, or no moves at all, is Done); Failed when there is at least one changing
/// move and none is at new; Mixed when some are and some are not.
/// Failed and Mixed carry every move, changing or not, with its actual value.
/// update_refs is one all-or-nothing transaction, so a changing move not at new proves nothing moved.
/// An unchanged move (old == new, or a delete of an absent ref) sits at its new value whether or not the
/// transaction ran, and may since have been moved elsewhere by someone else, so it proves nothing either
/// way and never decides. A changing move at new beside a changing move not at new can only come from a
/// foreign move, so it is refused.
pub fn decide_recovery(refs: &[(RefMove, Option<Oid>)]) -> RecoveryDecision {
    let changing_at_new: Vec<bool> = refs
        .iter()
        .filter(|(m, _)| move_changes(m))
        .map(|(m, actual)| at_new(m, actual.as_ref()))
        .collect();
    if changing_at_new.iter().all(|&b| b) {
        return RecoveryDecision::Done;
    }
    if changing_at_new.contains(&true) {
        return RecoveryDecision::Mixed(refs.to_vec());
    }
    RecoveryDecision::Failed(refs.to_vec())
}

/// An all-zero `new` deletes the ref, so an absent ref is at it; otherwise the ref must hold `new`.
fn at_new(m: &RefMove, actual: Option<&Oid>) -> bool {
    match actual {
        None => oid_is_zero(&m.new),
        Some(oid) => *oid == m.new,
    }
}
