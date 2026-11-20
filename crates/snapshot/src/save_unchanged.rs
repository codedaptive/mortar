//! save_unchanged: whether a save has a commit to write, and if not, the tree the folder's index is set to.

use gitio::Oid;

/// Some(`head_tree`) when the folder's saved `tree` equals the tree of the commit the save is built on and HEAD has
/// not moved off that commit (`head_moved` false): no commit is written, and the folder's index is set to that tree,
/// which repairs an index left stale by an earlier IndexNotSet. None when the trees differ, so a commit is to be
/// written, and None when HEAD has moved: the save then takes the commit path, whose op apply refuses with
/// BranchMoved, so the index is never set to a tree HEAD no longer has.
pub(crate) fn save_unchanged(tree: &Oid, head_tree: &Oid, head_moved: bool) -> Option<Oid> {
    (tree == head_tree && !head_moved).then(|| head_tree.clone())
}
