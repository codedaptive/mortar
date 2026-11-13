//! The recovery verdict on an open intent, from each planned move and the ref's actual value.

use gitio::Oid;

use super::RecoveryDecision;
use crate::store::RefMove;

/// Decided over the moves that change something only: Done when every changing move's ref is at new (so
/// an op with no changing move, or no moves at all, is Done); Failed when none is; Mixed when some are.
/// Failed and Mixed carry every move, changing or not, with its actual value.
/// update_refs is one all-or-nothing transaction (S-1), so a changing move not at new proves nothing moved.
/// An unchanged move (old == new, or a delete of an absent ref) sits at its new value whether or not the
/// transaction ran, and may since have been moved elsewhere by someone else, so it proves nothing either
/// way and never decides. A changing move at new beside a changing move not at new can only come from a
/// foreign move, so it is refused (S-13).
pub fn decide_recovery(refs: &[(RefMove, Option<Oid>)]) -> RecoveryDecision {
    let changing_at_new: Vec<bool> = refs
        .iter()
        .filter(|(m, _)| changes_something(m))
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
        None => is_zero(&m.new),
        Some(oid) => *oid == m.new,
    }
}

/// False when the move's old value already is its new one: old == new, or a delete of an absent ref.
fn changes_something(m: &RefMove) -> bool {
    match &m.old {
        None => !is_zero(&m.new),
        Some(old) => *old != m.new,
    }
}

fn is_zero(oid: &Oid) -> bool {
    oid.as_str().bytes().all(|b| b == b'0')
}
