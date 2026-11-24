//! land: lands a zone's squash on integration as a landing of one, answering in one landing's terms; cli reads it.
//! land_batch is the batch: land never makes a batch of two. It takes no lock: the caller holds `land` and has run
//! recover.
//! No unit test: it reads git and the landing state, and lands through land_batch, which runs git. Covered by the
//! landing_zit tests a_squash_on_origin_fast_forwards_integration_to_the_squash,
//! a_squash_behind_a_moved_origin_lands_as_a_merge_commit, a_conflicting_squash_moves_nothing_and_writes_no_state,
//! a_second_squash_of_the_same_zone_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged,
//! old_land_of_another_zone_at_the_tip_is_busy_and_writes_nothing,
//! duplicate_zone_reports_its_member_and_changes_nothing,
//! killed_between_state_and_op_recovers_and_the_repeat_lands_the_states_commit and
//! killed_after_the_op_recovers_done_and_the_repeat_is_landed_with_no_new_record.

use crate::landing::{
    decide_landing, land_batch, read_landing_state, resolve_landing_state, BatchLandAnswer, LandAnswer, LandError,
    LandingStep,
};
use crate::squash::{read_tip, LandRefs, Squash};
use gitio::Git;

/// Reads the origin and integration tips and the landing state, resolved against the integration tip, as land_batch
/// does. Where decide_landing answers Append (integration is at a landing's tip, off origin, and the squash is not a
/// member), the answer is Busy with the newest member's change ID, and nothing more is read or written. Otherwise
/// land_batch(git, refs, squash, actor), its answer as a LandAnswer: Landed, Conflicted and Busy as they are.
pub fn land(git: &Git, refs: &LandRefs, squash: &Squash, actor: &str) -> Result<LandAnswer, LandError> {
    let origin = read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(LandError::Squash)?;
    let integration =
        read_tip(git, &format!("refs/heads/{}", refs.integration_branch)).map_err(LandError::Squash)?;
    let state = read_landing_state(&git.common_dir().map_err(LandError::Git)?)
        .map_err(LandError::State)?
        .map(|state| resolve_landing_state(state, &integration));
    let step = decide_landing(&origin, &integration, state.as_ref(), &squash.commit);
    if let (LandingStep::Append { .. }, Some(state)) = (step, state) {
        return Ok(LandAnswer::Busy { change_id: state.newest.change_id });
    }
    Ok(match land_batch(git, refs, squash, actor)? {
        BatchLandAnswer::Landed { integration } => LandAnswer::Landed { integration },
        BatchLandAnswer::Conflicted { files } => LandAnswer::Conflicted { files },
        // land_batch answers ZoneInBatch only when it appends, and land answered Busy above for every Append; the
        // caller holds lock `land`, so nothing moves integration or the state between the two reads.
        BatchLandAnswer::Busy { change_id } | BatchLandAnswer::ZoneInBatch { change_id, .. } => {
            LandAnswer::Busy { change_id }
        }
    })
}
