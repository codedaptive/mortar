//! The verdict on one move of a protected ref (S-29): a fast-forward or a creation passes; a rewind, a
//! sideways move or a delete is refused.

use gitio::Oid;

use super::ProtectedRef;
use crate::store::{oid_is_zero, OplogError, RefMove};

/// `current` is what `m.refname` holds now (None: absent); `merge_base` is the caller's answer to
/// merge_base(current, m.new) (None: unrelated histories, or not asked). The caller pairs `guard` with the
/// move; its refname is not compared here.
///
/// A delete is refused even of an absent ref. Otherwise an absent ref is a creation and passes, an unchanged
/// value passes, and a move passes only when its merge base is `current`, so `current` is an ancestor of
/// `m.new`. A rewind (merge base `m.new`) or a sideways move (another merge base, or none) is refused.
pub(crate) fn protected_move_verdict(
    m: &RefMove,
    guard: &ProtectedRef,
    current: Option<&Oid>,
    merge_base: Option<&Oid>,
) -> Result<(), OplogError> {
    let refused = || OplogError::Protected {
        refname: m.refname.clone(),
        current: current.cloned(),
        new: m.new.clone(),
        by: guard.by.clone(),
    };
    if oid_is_zero(&m.new) {
        return Err(refused());
    }
    match current {
        None => Ok(()),
        Some(cur) if *cur == m.new || merge_base == Some(cur) => Ok(()),
        Some(_) => Err(refused()),
    }
}
