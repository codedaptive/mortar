//! What remove does when the op log answers its delete op as a repeat, from where the branch is now.
//! A bare type; repeat_step_for_tests exercises each value.

/// `Removed`: the branch is absent, so the earlier op deleted it and nothing is left to do. `Again`: the branch is
/// back at the tip the op deletes it from (made again after that op), so a new op deletes it again. `Moved`: the
/// branch is at another commit, so nothing is deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepeatStep {
    Removed,
    Again,
    Moved,
}
