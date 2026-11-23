//! would_merge_with: whether a squash and further squashes merge into integration in order, each onto the result
//! of the one before, by land's own merge step.
//! No unit test: it runs git; would_merge_step_tests covers its step answers.
//! tests/squash_zit/would_merge_with_cases.rs covers it, in a throwaway repository: ordered_squashes_merge_cleanly,
//! second_squash_conflicts_with_first_result and preview_leaves_refs_and_reachable_objects_unchanged.

use crate::landing::{landing_target_for, would_merge_step, LandError, WouldMergeWith};
use crate::squash::{read_tip, LandRefs, Squash};
use gitio::Git;
use std::ops::ControlFlow;

/// Merges `squash`, then each of `with` in order, onto the tip of `refs/heads/<integration_branch>` as it is now
/// (not origin: unlike land, it answers even when integration is off origin), as successive lands would: each step
/// fast-forwards when the squash's only parent is the tip, else merges it into the tip with the tip as attribute
/// source and writes the merge commit, which becomes the next step's tip. With `with` empty it merges the one
/// squash as land would. Clean when every step merges; Conflicted naming the first step that does not (numbered
/// from 1). It moves no ref; the merge commits it writes are reached by nothing. NoZone when a squash carries no
/// Mortar-Zone trailer.
pub fn would_merge_with(
    git: &Git,
    refs: &LandRefs,
    squash: &Squash,
    with: &[Squash],
) -> Result<WouldMergeWith, LandError> {
    let mut tip = read_tip(git, &format!("refs/heads/{}", refs.integration_branch)).map_err(LandError::Squash)?;
    for (step, sq) in std::iter::once(squash).chain(with).enumerate() {
        let (target, _zone) = landing_target_for(git, &tip, sq)?;
        match would_merge_step(step + 1, &sq.commit, target) {
            ControlFlow::Continue(next) => tip = next,
            ControlFlow::Break(answer) => return Ok(answer),
        }
    }
    Ok(WouldMergeWith::Clean)
}
