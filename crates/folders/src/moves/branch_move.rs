//! What moving a folder does with its branch, from where the branch is against the move's two tips.
//! A bare type; branch_move_for_tests exercises each value.

/// `Move`: the branch is at the old tip, so one op moves it to the new tip. `AlreadyMoved`: the branch is at the
/// new tip already (join's rebase moved it), so no op is logged. `Elsewhere`: the branch is at neither tip, or has
/// no commit, so nothing is moved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BranchMove {
    Move,
    AlreadyMoved,
    Elsewhere,
}
