//! move_stalled: whether a pending folder move is one a kill or a refused move left undone.

use gitio::Oid;

/// True only when the folder's index tree is known, is not the new tip's tree, and is still where the move started:
/// the old tip's tree `old_tree`, or the op's recorded `folder_tree` when the move carried one. An index
/// at the new tip's tree is a finished move; an index anywhere else holds staged edits (after a finished move, or after
/// a kill before the move, which the next join still finishes), which show as unsaved work, not a stall.
/// None (the index has unmerged entries, so it has no tree) is not a stall: unmerged entries come only from git run by
/// hand in the folder (Mortar's join and resolve leave none), and status shows them on its `unmerged:` line.
pub fn move_stalled(index_tree: Option<&Oid>, old_tree: &Oid, folder_tree: Option<&Oid>, new_tree: &Oid) -> bool {
    index_tree.is_some_and(|tree| tree != new_tree && (tree == old_tree || Some(tree) == folder_tree))
}
