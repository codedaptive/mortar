//! WouldMergeWith: whether a squash and further squashes merge into integration in order. A declaration only;
//! would_merge_step_tests exercises Conflicted and the squash_zit would_merge_with cases exercise both.

use gitio::Oid;
use std::path::PathBuf;

/// The answer of would_merge_with; it moves no ref, and the merge commits of clean steps are reached by nothing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WouldMergeWith {
    /// Every squash merges in order, each onto the result of the one before.
    Clean,
    /// Step `step` conflicts (1 is the squash, 2 the first further squash, and so on); `commit` is that squash's
    /// commit; `files` in git's order.
    Conflicted { step: usize, commit: Oid, files: Vec<PathBuf> },
}
