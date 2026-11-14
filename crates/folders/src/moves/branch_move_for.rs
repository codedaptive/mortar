//! Where a folder's branch stands against a move's old and new tips.

use gitio::Oid;

use super::BranchMove;

/// `at` (the branch's commit, None when it has none) at `new` is AlreadyMoved; at `old` is Move; anything else is
/// Elsewhere. `new` is checked first, so a move whose tips are equal is AlreadyMoved and logs nothing.
pub(crate) fn branch_move_for(at: Option<&Oid>, old: &Oid, new: &Oid) -> BranchMove {
    match at {
        Some(at) if at == new => BranchMove::AlreadyMoved,
        Some(at) if at == old => BranchMove::Move,
        _ => BranchMove::Elsewhere,
    }
}
