//! back_out_onto: moves integration back to an origin, by default where the landing found it (SPEC-land 03-land.md,
//! 04-publish.md, S-56, S-59). It takes no lock: the caller holds `land` and has run recover (S-16). The squash
//! commit and its ref are untouched, so the squash goes back to its zone with the same change ID.
//! No unit test (S-5): it runs git, reads and removes the landing state and writes the op log; decide_back_out_tests
//! and landing_applied_tests cover its decisions. Covered by the landing_zit test
//! back_out_restores_integration_then_has_nothing_to_back_out and the publish_zit test
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again (integration put on the fetched origin).

use crate::landing::{
    decide_back_out, integration_op, landing_applied, read_landing_state, remove_landing_state, BackOutAnswer,
    BackOutStep, LandError,
};
use crate::squash::{read_tip, LandRefs};
use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now};

/// The origin is `onto` when given (publish passes the local origin tip it fetched or found), else the local origin
/// tip read here. Reads the integration tip and the landing state, then: integration at the origin removes any
/// landing state (finishing a back-out killed after its op) and answers NothingToBackOut; integration off the origin
/// with no state is refused as NoLandingState; integration at neither the origin nor the state's commit as
/// IntegrationElsewhere. Otherwise one op moves integration from the state's commit to `onto`, or without it to the
/// state's origin_at_start; the state is removed, and the answer is BackedOut with the landing's change ID.
pub(crate) fn back_out_onto(
    git: &Git,
    refs: &LandRefs,
    actor: &str,
    onto: Option<&Oid>,
) -> Result<BackOutAnswer, LandError> {
    let origin = match onto {
        Some(onto) => onto.clone(),
        None => read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(LandError::Squash)?,
    };
    let integration_ref = format!("refs/heads/{}", refs.integration_branch);
    let integration = read_tip(git, &integration_ref).map_err(LandError::Squash)?;
    let common = git.common_dir().map_err(LandError::Git)?;
    let state = read_landing_state(&common).map_err(LandError::State)?;
    let landed = match decide_back_out(&origin, &integration, state.as_ref()) {
        BackOutStep::Nothing => {
            remove_landing_state(&common).map_err(LandError::State)?;
            return Ok(BackOutAnswer::NothingToBackOut);
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
        &landed.integration,
        onto.unwrap_or(&landed.origin_at_start),
        &landed.change_id,
        new_op_id(),
        actor,
        unix_now(),
    );
    landing_applied(apply(git, &common, &op))?;
    remove_landing_state(&common).map_err(LandError::State)?;
    Ok(BackOutAnswer::BackedOut { change_id: landed.change_id })
}
