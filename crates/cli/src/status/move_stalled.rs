//! move_stalled: whether a pending folder move is one a kill left undone (S-85, S-97).

use gitio::Oid;

/// True only when the folder's index tree is known and differs from the tree of the request branch's new tip: the
/// branch moved and the folder did not follow. None (the index has unmerged entries, so it has no tree) is not a
/// stall: an index mid-conflict is a paused join's, which status reports on its own line (S-97).
pub fn move_stalled(index_tree: Option<&Oid>, new_tree: &Oid) -> bool {
    index_tree.is_some_and(|tree| tree != new_tree)
}
