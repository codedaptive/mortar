//! What remove does with a folder, from its record, its HEAD and where its branch stands against its zone.
//! A bare type; remove_step_for_tests exercises each value.

/// `Go`: a request folder on its own branch, contained in its zone branch, so it is removed. `ZoneFolder`: a
/// zone's standing folder, never removed. `OffBranch`: HEAD is on another branch than the folder's request branch.
/// `NotJoined`: the branch is not contained in its zone branch, so its work has not joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RemoveStep {
    Go,
    ZoneFolder,
    OffBranch,
    NotJoined,
}
