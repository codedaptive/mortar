//! What a ref holds after a move sets it to `new` (S-9: an all-zero new deletes the ref).

use gitio::Oid;

use crate::store::oid_is_zero;

/// None when `new` is all-zero (the ref is gone), else `new`.
pub(crate) fn value_after_move(new: &Oid) -> Option<Oid> {
    if oid_is_zero(new) {
        None
    } else {
        Some(new.clone())
    }
}
