//! Whether a move changes its ref at all; recover and undo both decide by it.

use super::{oid_is_zero, RefMove};

/// False when the move's old value already is its new one: old == new, or a delete of an absent ref.
pub(crate) fn move_changes(m: &RefMove) -> bool {
    match &m.old {
        None => !oid_is_zero(&m.new),
        Some(old) => *old != m.new,
    }
}
