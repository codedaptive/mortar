//! would_merge_from: would_merge's answer from a trial merge.

use crate::squash::WouldMerge;
use gitio::MergeTreeOutcome;

/// A clean merge is Yes; a conflicted one is No with its files. The tree is dropped: the answer is only
/// whether the squash merges.
pub(crate) fn would_merge_from(outcome: MergeTreeOutcome) -> WouldMerge {
    match outcome {
        MergeTreeOutcome::Clean(_) => WouldMerge::Yes,
        MergeTreeOutcome::Conflicted { files, .. } => WouldMerge::No { files },
    }
}
