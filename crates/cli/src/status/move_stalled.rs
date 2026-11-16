//! move_stalled: whether a pending folder move is one a kill left undone (S-85, S-97, S-116).

use gitio::Oid;

/// True only when the folder's index tree is known, is not the new tip's tree, and is still where the move started:
/// the old tip's tree `old_tree`, or the op's recorded `folder_tree` when the move carried one (S-87, S-116). An index
/// at the new tip's tree is a finished move; an index anywhere else holds staged edits (after a finished move, or after
/// a kill before the move, which the next join still finishes, S-86), which show as unsaved work, not a stall (S-116).
/// None (the index has unmerged entries, so it has no tree) is not a stall: an index mid-conflict is a paused join's,
/// which status reports on its own line (S-97).
pub fn move_stalled(index_tree: Option<&Oid>, old_tree: &Oid, folder_tree: Option<&Oid>, new_tree: &Oid) -> bool {
    index_tree.is_some_and(|tree| tree != new_tree && (tree == old_tree || Some(tree) == folder_tree))
}
