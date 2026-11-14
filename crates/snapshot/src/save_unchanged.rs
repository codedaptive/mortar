//! save_unchanged: whether a save has nothing to write.

use gitio::Oid;

/// True when the folder's saved `tree` equals HEAD's tree, so a save writes nothing.
pub(crate) fn save_unchanged(tree: &Oid, head_tree: &Oid) -> bool {
    tree == head_tree
}
