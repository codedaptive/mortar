//! would_merge_step: one step of would_merge_with, from land's target for that step's squash.

use crate::landing::{LandingTarget, WouldMergeWith};
use gitio::Oid;
use std::ops::ControlFlow;

/// A Commit target continues with that commit as the tip the next step merges onto; a Conflicted target stops
/// with the answer naming `step`, the squash's `commit` and the conflicted files in git's order.
pub(crate) fn would_merge_step(step: usize, commit: &Oid, target: LandingTarget) -> ControlFlow<WouldMergeWith, Oid> {
    match target {
        LandingTarget::Commit(tip) => ControlFlow::Continue(tip),
        LandingTarget::Conflicted(files) => {
            ControlFlow::Break(WouldMergeWith::Conflicted { step, commit: commit.clone(), files })
        }
    }
}
