//! save_unchanged: whether a save has a commit to write, and if not, the tree the folder's index is set to (S-41).

use gitio::Oid;

/// Some(`head_tree`) when the folder's saved `tree` equals HEAD's tree: no commit is written, and the folder's index
/// is set to HEAD's tree, which repairs an index left stale by an earlier IndexNotSet (S-41). None when the trees
/// differ, so a commit is to be written.
pub(crate) fn save_unchanged(tree: &Oid, head_tree: &Oid) -> Option<Oid> {
    (tree == head_tree).then(|| head_tree.clone())
}
