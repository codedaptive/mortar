//! publish_with_push: puts a landing on the remote (push on) and then on the local origin branch. It takes no lock:
//! the caller holds `land` and has run recover.
//! No unit test: it runs git, reads the landing state and writes the op log; decide_publish_tests covers its
//! decisions. Covered by the publish_zit tests (through publish, push on)
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up,
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again,
//! killed_between_the_steps_recovers_and_the_repeat_publishes_once,
//! with_no_landing_state_there_is_nothing_to_publish_and_the_remote_is_untouched,
//! publishing::a_remote_already_past_new_with_no_open_intent_is_published,
//! publishing::an_open_push_intent_after_the_push_is_closed_done_on_the_repeat,
//! publishing::an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again and
//! publishing::an_open_push_intent_with_the_remote_past_new_is_closed_done (a push intent left open is settled
//! before the step is decided); and directly by no_push::push_off_publishes_locally_without_reading_the_remote,
//! no_push::push_off_repeat_after_origin_advanced_only_cleans_up,
//! no_push::push_off_moved_local_origin_backs_out_without_fetching, no_push::push_on_published_reports_pushed and
//! no_push::push_on_moved_remote_reports_not_pushed, repeats::open_push_intent_refuses_push_off_then_push_on_finishes
//! and repeats::push_off_killed_after_origin_move_finishes_push_on_without_push; a commit_files commit by the
//! commit_files_zit tests committing::named_commit_is_published_without_the_unnamed_edit and
//! kills::killed_after_update_ref_recovers_done_repeats_same_and_publishes (a pending commit integration reached).

use crate::landing::{
    advance_origin, back_out_onto, decide_publish, fetch_origin, finish_publish, push_landing, read_landing_state,
    push_landed, read_push_intent, remote_origin, resolve_landing_state, settle_push_intent, LandingState,
    PublishAnswer, PublishError, PublishOutcome, PublishStep, PushIntentState, PushResult,
};
use crate::squash::{read_tip, LandRefs};
use gitio::{Git, Oid};
use oplog::OpPhase;
use std::path::Path;

/// With no landing state: NothingToPublish, the remote not contacted. Otherwise, with expected = the state's
/// origin_at_start and new = its integration (resolve_landing_state against integration's tip, read only when the
/// state has a pending commit), the local origin tip decides (decide_publish).
///
/// Push on: the remote is read only when the local origin is at expected. A remote at or past new (push_landed: new
/// is its ancestor, so our push landed and someone pushed on top) counts as at new. Once read, a push intent for
/// this move left open by an earlier run is closed from it first (settle_push_intent): done when the remote is at or
/// past new, else failed. Cleanup and BackOut read no remote: a push-on run that moved the local origin had closed
/// its push intent already, and a push-off run's repeat has none, so its clean-up finishes without a push. Then:
/// push new leased at expected, move the local origin to new, delete the zone's squash ref and remove the state:
/// Published { origin: new }. A remote already at or past new skips the push; a local origin already at new does
/// only the clean-up. A remote someone else moved (seen before the push or after it was refused) is fetched, the
/// local origin fast-forwarded to it and the landing backed out onto it: OriginMoved { found }. A local origin at
/// neither expected nor new (a publish killed after that fast-forward) finishes the back-out onto it. A remote with
/// no origin branch is RemoteMissing.
///
/// Push off (SPEC 04 "publish with push off"): no push, no remote intent, the remote is never read and nothing is
/// fetched; only the op log is read for this publish's push intent. While that intent is open (a push-on run killed
/// between its steps): PushIntent { phase: Intent }, nothing changed. Otherwise a local origin at expected moves to
/// new and the clean-up runs: Published { origin: new }; one already at new does only the clean-up: Published; one
/// anywhere else has the landing backed out onto it: OriginMoved { found: the local origin }.
///
/// `pushed` is true only when the answer is Published and this publish's push intent, read after the step, closed
/// done, on either setting. The origin never moves backward: every local origin move is protected, the push leased.
pub fn publish_with_push(git: &Git, refs: &LandRefs, actor: &str, push: bool) -> Result<PublishOutcome, PublishError> {
    let common = git.common_dir().map_err(PublishError::Git)?;
    let Some(state) = read_landing_state(&common).map_err(PublishError::State)? else {
        return Ok(PublishOutcome { answer: PublishAnswer::NothingToPublish, pushed: false });
    };
    let state = if state.pending.is_some() {
        let integration_ref = format!("refs/heads/{}", refs.integration_branch);
        resolve_landing_state(state, &read_tip(git, &integration_ref).map_err(PublishError::Squash)?)
    } else {
        state
    };
    let local = read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(PublishError::Squash)?;
    let intent = read_push_intent(&common, refs, &state)?;
    let remote = if push && local == state.origin_at_start { read_remote(git, &common, refs, &state)? } else { None };
    let answer = match decide_publish(push, &intent, remote.as_ref(), &local, Some(&state)) {
        PublishStep::Nothing => PublishAnswer::NothingToPublish,
        PublishStep::Push => match push_landing(git, &common, refs, &state, actor)? {
            PushResult::Pushed => advanced(git, &common, refs, &state, actor)?,
            PushResult::Moved => origin_moved(git, &common, refs, &state, &local, actor)?,
        },
        PublishStep::Advance => advanced(git, &common, refs, &state, actor)?,
        PublishStep::Cleanup => {
            finish_publish(git, &common, &state, actor)?;
            PublishAnswer::Published { origin: state.integration.clone() }
        }
        PublishStep::FetchOrigin => origin_moved(git, &common, refs, &state, &local, actor)?,
        PublishStep::BackOut { onto } => backed_out_onto(git, refs, actor, onto)?,
        PublishStep::RemoteMissing => {
            return Err(PublishError::RemoteMissing {
                remote: refs.remote.clone(),
                refname: format!("refs/heads/{}", refs.origin_branch),
            })
        }
        PublishStep::PushOpen { op_id } => return Err(PublishError::PushIntent { op_id, phase: OpPhase::Intent }),
    };
    let pushed = matches!(answer, PublishAnswer::Published { .. })
        && read_push_intent(&common, refs, &state)? == PushIntentState::Done;
    Ok(PublishOutcome { answer, pushed })
}

/// The remote's origin as decide_publish takes it, after settling a push intent left open (settle_push_intent):
/// the integration commit when the push landed (push_landed), else the remote as read.
fn read_remote(git: &Git, common: &Path, refs: &LandRefs, state: &LandingState) -> Result<Option<Oid>, PublishError> {
    let remote = remote_origin(git, refs)?;
    let landed = push_landed(git, refs, remote.as_ref(), &state.origin_at_start, &state.integration)?;
    settle_push_intent(common, refs, state, remote.as_ref(), landed)?;
    Ok(if landed { Some(state.integration.clone()) } else { remote })
}

/// Step 2 moves the local origin from expected to new, then the clean-up: Published { origin: new }. Push on, the
/// remote is at or past new; push off, the remote is not consulted.
fn advanced(
    git: &Git,
    common: &Path,
    refs: &LandRefs,
    state: &LandingState,
    actor: &str,
) -> Result<PublishAnswer, PublishError> {
    advance_origin(git, common, refs, state, actor)?;
    finish_publish(git, common, state, actor)?;
    Ok(PublishAnswer::Published { origin: state.integration.clone() })
}

/// The remote moved: fetch it, fast-forward the local origin from `local` onto it, and back the landing out there.
fn origin_moved(
    git: &Git,
    common: &Path,
    refs: &LandRefs,
    state: &LandingState,
    local: &Oid,
    actor: &str,
) -> Result<PublishAnswer, PublishError> {
    let found = fetch_origin(git, common, refs, local, &state.change_id, actor)?;
    backed_out_onto(git, refs, actor, found)
}

/// Integration put on the local origin tip `found` and the landing state removed: OriginMoved { found }.
fn backed_out_onto(git: &Git, refs: &LandRefs, actor: &str, found: Oid) -> Result<PublishAnswer, PublishError> {
    back_out_onto(git, refs, actor, Some(&found)).map_err(PublishError::Land)?;
    Ok(PublishAnswer::OriginMoved { found })
}
