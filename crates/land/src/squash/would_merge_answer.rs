//! WouldMerge: whether a squash merges cleanly into integration. A declaration only;
//! would_merge_from_tests exercises both variants.

use std::path::PathBuf;

/// The answer of would_merge; it keeps no commit and moves no ref.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WouldMerge {
    /// The squash merges into the integration tip without conflicts.
    Yes,
    /// The merge would conflict in these files, in git's order.
    No { files: Vec<PathBuf> },
}
