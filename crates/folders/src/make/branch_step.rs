//! What making a folder does next after the op that makes its branch.
//! A bare type; branch_step_after_tests exercises each value.

/// `Ready`: the branch is at the tip, go on. `Remake`: the op log says the branch was made but the ref is gone,
/// so make it again. `Exists`: the branch is at another commit, or was not made by mortar at this one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BranchStep {
    Ready,
    Remake,
    Exists,
}
