//! snapshot_parents: the parents of a new snapshot commit.

use gitio::Oid;

/// [head] for a ref's first snapshot, [head, previous] after that, HEAD first (S-19): the folder's branch is
/// the first-parent line, and the previous snapshot chains the ref's snapshots together.
pub(crate) fn snapshot_parents(head: &Oid, previous: Option<&Oid>) -> Vec<Oid> {
    std::iter::once(head).chain(previous).cloned().collect()
}
