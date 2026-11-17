//! What left a request's folder move pending: a join rebase, or an undo of a plain join rebase that put the branch
//! back (S-143). An undo of a resolve's final join step leaves no pending move (S-148).
//! A bare type (S-7); find_pending_move_tests and finished_move_part_tests exercise it.

/// Rebase: a join-rebase op moved the request branch to its rebased tip (join and resolve write it). Undo: an undo
/// op reversed a plain join rebase, not a resolve step (is_resolve_step), and put the branch back at the tip before
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCause {
    Rebase,
    Undo,
}
