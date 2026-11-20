//! integration_move_stopped: whether the integration folder is where a stopped run left it.

use gitio::Oid;

/// True only when integration has moved past `old` (`tip` is not `old`) and the folder's index tree is still
/// `old_tree`, the tree of `old`: the op moved the branch and the folder never followed. An index at any other tree
/// (the tip's after a finished move, or one holding staged edits) is left alone: staged edits are someone's work, not
/// a stall. None (the index has unmerged entries, so it has no tree) is never a stall.
pub fn integration_move_stopped(index_tree: Option<&Oid>, old: &Oid, old_tree: &Oid, tip: &Oid) -> bool {
    tip != old && index_tree == Some(old_tree)
}
