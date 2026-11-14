//! One try at a folder's two-tree move.
//! A bare type; move_attempts_tests exercises the values move_attempts builds.

use gitio::Oid;

/// One try at the two-tree move: when `set_index` is Some, the folder's index is set to that tree first; then the
/// folder moves from `from` to the new tip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MoveAttempt<'a> {
    pub set_index: Option<&'a Oid>,
    pub from: &'a Oid,
}
