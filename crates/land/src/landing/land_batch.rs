//! land_batch: lands a zone's squash on integration as a member of the landing batch, answering in the batch's
//! terms. It takes no lock: the caller holds `land` and has run recover. The reviewed squash commit is never
//! rewritten.
//! No unit test: it runs git, reads and writes the landing state and writes the op log; decide_landing_tests,
//! zone_in_batch_tests, is_fast_forward_tests and landing_applied_tests cover its decisions.
//! Covered by the landing_zit tests a_squash_on_origin_fast_forwards_integration_to_the_squash,
//! a_squash_behind_a_moved_origin_lands_as_a_merge_commit, a_conflicting_squash_moves_nothing_and_writes_no_state,
//! a_second_squash_of_the_same_zone_is_busy_and_the_first_ones_repeat_is_landed_with_the_log_unchanged (through land),
//! three_zones_append_in_order, conflict_keeps_batch_and_next_zone_can_append,
//! duplicate_zone_reports_its_member_and_changes_nothing, off_tip_is_busy_with_newest_member,
//! repeat_earlier_member_returns_its_own_commit,
//! killed_between_state_and_op_recovers_and_the_repeat_lands_the_states_commit,
//! killed_after_the_op_recovers_done_and_the_repeat_is_landed_with_no_new_record,
//! killed_second_member_recovers_and_repeat_lands_once and killed_second_member_recovers_then_back_out_discards_both,
//! and by the commit_files_zit test append_after_commit_files_uses_the_named_commit_as_parent.

use crate::landing::{
    decide_landing, integration_op, landing_applied, landing_target_for, read_landing_state, resolve_landing_state,
    squash_zone, write_landing_state, zone_in_batch, BatchLandAnswer, LandError, LandingMember, LandingState,
    LandingStep, LandingTarget,
};
use crate::squash::{read_tip, LandRefs, Squash};
use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now};
use std::ops::ControlFlow;
use std::path::Path;

/// Reads the origin and integration tips and the landing state, resolved against the integration tip
/// (resolve_landing_state), and acts on decide_landing: Landed (a repeat, answering that member's integration) and
/// Busy move nothing; NoState is refused as NoLandingState; Resume moves integration from the point before the
/// newest member to the state's commit. Fresh lands the squash onto origin as a new batch of one. Append answers
/// ZoneInBatch, with nothing written, when a member already holds the zone in the squash's Mortar-Zone trailer
/// (NoZone without one); else it lands the squash onto the batch tip as the batch's newest member. Landing onto a
/// commit builds the target (the squash for a fast-forward, else a merge commit named after the zone): conflicted
/// answers Conflicted and writes no state and no op, so the batch is untouched; else the landing state with the new
/// member is written, then one `land` op moves integration from the commit it was decided on to the target: Landed.
pub fn land_batch(git: &Git, refs: &LandRefs, squash: &Squash, actor: &str) -> Result<BatchLandAnswer, LandError> {
    let origin = read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(LandError::Squash)?;
    let integration_ref = format!("refs/heads/{}", refs.integration_branch);
    let integration = read_tip(git, &integration_ref).map_err(LandError::Squash)?;
    let common = git.common_dir().map_err(LandError::Git)?;
    let state = read_landing_state(&common)
        .map_err(LandError::State)?
        .map(|state| resolve_landing_state(state, &integration));
    let step = decide_landing(&origin, &integration, state.as_ref(), &squash.commit);
    let (from, joined) = match (step, state) {
        (LandingStep::Landed(integration), _) => return Ok(BatchLandAnswer::Landed { integration }),
        (LandingStep::Busy(change_id), _) => return Ok(BatchLandAnswer::Busy { change_id }),
        (LandingStep::Resume { from, to }, _) => (from, ControlFlow::Continue(to)),
        (LandingStep::Fresh, _) => (origin.clone(), join(git, &common, &origin, origin.clone(), Vec::new(), squash)?),
        (LandingStep::Append { onto }, Some(state)) => {
            let joined = append(git, &common, &onto, state, squash)?;
            (onto, joined)
        }
        // decide_landing answers Append only with a state.
        (LandingStep::NoState | LandingStep::Append { .. }, _) => {
            return Err(LandError::NoLandingState { origin, integration })
        }
    };
    let to = match joined {
        ControlFlow::Continue(to) => to,
        ControlFlow::Break(answer) => return Ok(answer),
    };
    let op = integration_op("land", &integration_ref, &from, &to, &squash.change_id, new_op_id(), actor, unix_now());
    landing_applied(apply(git, &common, &op))?;
    Ok(BatchLandAnswer::Landed { integration: to })
}

/// Appends the squash to `state`'s batch onto its tip `onto`: Break(ZoneInBatch), with nothing written, when a
/// member already holds the squash's zone; else as join, with the batch's members kept before the new one.
fn append(
    git: &Git,
    common: &Path,
    onto: &Oid,
    state: LandingState,
    squash: &Squash,
) -> Result<ControlFlow<BatchLandAnswer, Oid>, LandError> {
    let zone = squash_zone(git, squash)?;
    if let Some(member) = zone_in_batch(&state, &zone) {
        return Ok(ControlFlow::Break(BatchLandAnswer::ZoneInBatch { zone, change_id: member.change_id.clone() }));
    }
    let earlier = state.members().cloned().collect();
    join(git, common, onto, state.origin_at_start, earlier, squash)
}

/// Builds the target of landing the squash onto `onto` (landing_target_for). Conflicted: Break(Conflicted), and no
/// state is written. A commit: the landing state, `origin_at_start` and `earlier` then the squash as the newest
/// member at the target, is written under `common`, and Continue(target) is returned for the op.
fn join(
    git: &Git,
    common: &Path,
    onto: &Oid,
    origin_at_start: Oid,
    earlier: Vec<LandingMember>,
    squash: &Squash,
) -> Result<ControlFlow<BatchLandAnswer, Oid>, LandError> {
    let (target, zone) = match landing_target_for(git, onto, squash)? {
        (LandingTarget::Commit(target), zone) => (target, zone),
        (LandingTarget::Conflicted(files), _) => return Ok(ControlFlow::Break(BatchLandAnswer::Conflicted { files })),
    };
    let newest = LandingMember {
        change_id: squash.change_id.clone(),
        zone,
        squash: squash.commit.clone(),
        integration: target.clone(),
    };
    let state = LandingState { origin_at_start, earlier, newest, pending: None };
    write_landing_state(common, &state).map_err(LandError::State)?;
    Ok(ControlFlow::Continue(target))
}
