//! What left a request's folder move pending: a join rebase, a joined partner request, or an undo of a plain join
//! rebase that put the branch back. An undo of a resolve's final join step leaves no pending move.
//! A bare type; find_pending_move_tests and finished_move_part_tests exercise it.

/// Rebase: a join-rebase op moved the request branch to its rebased tip (join and resolve write it), or a
/// join-zone-request op moved it to the joined commit (join writes it). Undo: an undo
/// op reversed a plain join rebase, not a resolve step (is_resolve_step), and put the branch back at the tip before
/// it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveCause {
    Rebase,
    Undo,
}
