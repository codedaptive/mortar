//! The folder move a request's last mover left: the request branch stands at the move's new tip; the folder stands at
//! the old tip if a kill or a refusal stopped the move, or already at the new.
//! A bare type; find_pending_move_tests exercises it.

use gitio::Oid;

use super::MoveCause;

/// `request_ref` is the full ref name; `old` and `new` are the tips of the move of op `op_id`: for cause Rebase the
/// join-rebase op's old and new, for cause Undo the undo op's (the rebased tip back to the tip before the rebase).
/// `folder_tree` is the tree a join-rebase op recorded under "folder_tree": after a resolve, the tree
/// holding the resolution that the folder moves from. None when the op carries none, as a plain join's rebase
/// does, and always None for an undo's move.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingMove {
    pub request_ref: String,
    pub old: Oid,
    pub new: Oid,
    pub op_id: String,
    pub folder_tree: Option<Oid>,
    pub cause: MoveCause,
}
