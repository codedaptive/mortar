//! The folder move of a request's last join rebase: the request branch stands at its new tip; the folder stands at the
//! old tip if a kill stopped the move, or already at the new.
//! A bare type (S-7); find_pending_move_tests exercises it.

use gitio::Oid;

/// `request_ref` is the full ref name; `old` and `new` are the tips of the join-rebase op `op_id` that moved it.
/// `folder_tree` is the tree that op recorded under "folder_tree" (S-89(2)): after a resolve, the tree holding the
/// resolution that the folder moves from (S-87). None when the op carries none, as a plain join's rebase does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingMove {
    pub request_ref: String,
    pub old: Oid,
    pub new: Oid,
    pub op_id: String,
    pub folder_tree: Option<Oid>,
}
