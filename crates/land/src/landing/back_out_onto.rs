//! back_out_onto: moves integration back to an origin, by default where the landing batch found it, dropping every
//! member at once. It takes no lock: the caller holds `land` and has run recover. The squash commits and their refs
//! are untouched, so each squash goes back to its zone with the same change ID.
//! No unit test: it runs git, reads and removes the landing state and writes the op log; decide_back_out_tests
//! and landing_applied_tests cover its decisions. Covered by the landing_zit tests
//! backing_out::back_out_restores_integration_then_has_nothing_to_back_out,
//! batch::back_out_restores_origin_for_every_member and kills::killed_second_member_recovers_then_back_out_discards_both
//! (integration at the point before the newest member); the publish_zit test
//! publishing::a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again (integration put on the
//! fetched origin); the commit_files_zit test committing::back_out_drops_the_landing_and_named_commit backs out a
//! commit_files commit.

use crate::landing::{
    decide_back_out, integration_op, landing_applied, read_landing_state, remove_landing_state,
    resolve_landing_state, BackOutStep, BatchBackOutAnswer, LandError,
};
use crate::squash::{read_tip, LandRefs};
use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now};

/// The origin is `onto` when given (publish passes the local origin tip it fetched or found), else the local origin
/// tip read here. Reads the integration tip and the landing state, resolved against that tip (resolve_landing_state,
/// so the newest member's integration may be a commit_files commit integration reached), then: integration at the
/// origin removes any landing state (finishing a back-out killed after its op) and answers NothingToBackOut;
/// integration off the origin with no state is refused as NoLandingState; integration at neither the batch's tip
/// nor the point before its newest member as IntegrationElsewhere. Otherwise one `land-back-out` op, with the
/// newest member's change ID, moves integration from where it was read (the tip, or the point before the newest
/// member when that member's land was killed between its state write and its op) to `onto`, or without it to the
/// state's origin_at_start; the state is removed, and the answer is BackedOut with the newest member's change ID
/// and every member's in landing order. The whole batch goes: there is no partial back-out.
pub(crate) fn back_out_onto(
    git: &Git,
    refs: &LandRefs,
    actor: &str,
    onto: Option<&Oid>,
) -> Result<BatchBackOutAnswer, LandError> {
    let origin = match onto {
        Some(onto) => onto.clone(),
        None => read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(LandError::Squash)?,
    };
    let integration_ref = format!("refs/heads/{}", refs.integration_branch);
    let integration = read_tip(git, &integration_ref).map_err(LandError::Squash)?;
    let common = git.common_dir().map_err(LandError::Git)?;
    let state = read_landing_state(&common)
        .map_err(LandError::State)?
        .map(|state| resolve_landing_state(state, &integration));
    let landed = match decide_back_out(&origin, &integration, state.as_ref()) {
        BackOutStep::Nothing => {
            remove_landing_state(&common).map_err(LandError::State)?;
            return Ok(BatchBackOutAnswer::NothingToBackOut);
        }
        BackOutStep::NoState => return Err(LandError::NoLandingState { origin, integration }),
        BackOutStep::Elsewhere { expected } => {
            return Err(LandError::IntegrationElsewhere { expected, actual: integration })
        }
        BackOutStep::Revert(landed) => landed,
    };
    let op = integration_op(
        "land-back-out",
        &integration_ref,
        &integration,
        onto.unwrap_or(&landed.origin_at_start),
        &landed.newest.change_id,
        new_op_id(),
        actor,
        unix_now(),
    );
    landing_applied(apply(git, &common, &op))?;
    remove_landing_state(&common).map_err(LandError::State)?;
    let change_ids = landed.members().map(|member| member.change_id.clone()).collect();
    Ok(BatchBackOutAnswer::BackedOut { change_id: landed.newest.change_id, change_ids })
}
