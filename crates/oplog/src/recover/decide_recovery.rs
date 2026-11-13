//! The recovery verdict on an open intent, from each planned move and the ref's actual value.

use gitio::Oid;

use super::RecoveryDecision;
use crate::store::RefMove;

/// Done when every ref is at new (checked first, so an op with no moves, or with old == new, is Done);
/// Failed when every ref is at old; else Mixed with every move and its actual value.
pub fn decide_recovery(refs: &[(RefMove, Option<Oid>)]) -> RecoveryDecision {
    if refs.iter().all(|(m, actual)| at_new(m, actual.as_ref())) {
        return RecoveryDecision::Done;
    }
    if refs.iter().all(|(m, actual)| m.old.as_ref() == actual.as_ref()) {
        return RecoveryDecision::Failed;
    }
    RecoveryDecision::Mixed(refs.to_vec())
}

/// An all-zero `new` deletes the ref, so an absent ref is at it; otherwise the ref must hold `new`.
fn at_new(m: &RefMove, actual: Option<&Oid>) -> bool {
    match actual {
        None => m.new.as_str().bytes().all(|b| b == b'0'),
        Some(oid) => *oid == m.new,
    }
}
