//! is_fast_forward: whether a squash lands by moving integration onto it. is_fast_forward_tests covers it.

use gitio::Oid;

/// True only when the squash has exactly one parent and that parent is `integration`.
pub(crate) fn is_fast_forward(squash_parents: &[Oid], integration: &Oid) -> bool {
    matches!(squash_parents, [parent] if parent == integration)
}
