//! land: lands a zone's squash on integration. It takes no lock: the caller holds `land` and has run recover. The
//! reviewed squash commit is never rewritten.
//! No unit test: it runs git, reads and writes the landing state and writes the op log; decide_landing_tests,
//! is_fast_forward_tests and landing_applied_tests cover its decisions.
//! Covered by the landing_zit tests a_squash_on_origin_fast_forwards_integration_to_the_squash,
//! a_squash_behind_a_moved_origin_lands_as_a_merge_commit, a_conflicting_squash_moves_nothing_and_writes_no_state,
//! a_second_landing_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged,
//! killed_between_state_and_op_recovers_and_the_repeat_lands_the_states_commit and
//! killed_after_the_op_recovers_done_and_the_repeat_is_landed_with_no_new_record.

use crate::landing::{
    decide_landing, integration_op, landing_applied, landing_target_for, read_landing_state, write_landing_state,
    LandAnswer, LandError, LandingState, LandingStep, LandingTarget,
};
use crate::squash::{read_tip, LandRefs, Squash};
use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now};
use std::path::Path;

/// Reads the origin and integration tips and the landing state, then: this squash already landed at integration
/// answers Landed and moves nothing; integration off origin answers Busy with the landing state's change ID, or is
/// refused as NoLandingState without one; a state for this squash written but never applied lands the commit it
/// names. Otherwise the target is built (the squash for a fast-forward, else a merge commit named after the zone
/// in the squash's Mortar-Zone trailer, NoZone without one): conflicted answers Conflicted and writes no state and
/// no op; else the landing state is written, then one op moves integration from origin to the target: Landed.
pub fn land(git: &Git, refs: &LandRefs, squash: &Squash, actor: &str) -> Result<LandAnswer, LandError> {
    let origin = read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(LandError::Squash)?;
    let integration_ref = format!("refs/heads/{}", refs.integration_branch);
    let integration = read_tip(git, &integration_ref).map_err(LandError::Squash)?;
    let common = git.common_dir().map_err(LandError::Git)?;
    let state = read_landing_state(&common).map_err(LandError::State)?;
    let target = match decide_landing(&origin, &integration, state.as_ref(), &squash.commit) {
        LandingStep::Landed(integration) => return Ok(LandAnswer::Landed { integration }),
        LandingStep::Busy(change_id) => return Ok(LandAnswer::Busy { change_id }),
        LandingStep::NoState => return Err(LandError::NoLandingState { origin, integration }),
        LandingStep::Resume(target) => target,
        LandingStep::Fresh => match fresh_target(git, &common, &origin, squash)? {
            LandingTarget::Commit(target) => target,
            LandingTarget::Conflicted(files) => return Ok(LandAnswer::Conflicted { files }),
        },
    };
    let op = integration_op(
        "land",
        &integration_ref,
        &origin,
        &target,
        &squash.change_id,
        new_op_id(),
        actor,
        unix_now(),
    );
    landing_applied(apply(git, &common, &op))?;
    Ok(LandAnswer::Landed { integration: target })
}

/// The target of a fresh landing onto `origin` (where integration stands), from landing_target_for. A Commit target
/// has its landing state written under `common` before it is returned; a Conflicted one writes no state.
fn fresh_target(git: &Git, common: &Path, origin: &Oid, squash: &Squash) -> Result<LandingTarget, LandError> {
    let (target, zone) = landing_target_for(git, origin, squash)?;
    if let LandingTarget::Commit(commit) = &target {
        let state = LandingState {
            change_id: squash.change_id.clone(),
            zone,
            squash: squash.commit.clone(),
            origin_at_start: origin.clone(),
            integration: commit.clone(),
        };
        write_landing_state(common, &state).map_err(LandError::State)?;
    }
    Ok(target)
}
