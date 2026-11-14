//! The folder move of a request's last join rebase: the request branch stands at its new tip; the folder stands at the
//! old tip if a kill stopped the move, or already at the new.
//! A bare type (S-7); find_pending_move_tests exercises it.

use gitio::Oid;

/// `request_ref` is the full ref name; `old` and `new` are the tips of the join-rebase op `op_id` that moved it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingMove {
    pub request_ref: String,
    pub old: Oid,
    pub new: Oid,
    pub op_id: String,
}
