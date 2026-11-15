//! publish: puts a landing on the remote and then on the local origin branch (SPEC-land 04-publish.md, S-16, S-29,
//! S-59, S-60). It takes no lock: the caller holds `land` and has run recover (S-16).
//! No unit test (S-5): it reads the landing state, runs git against the remote and writes the op log;
//! decide_publish_tests covers its decisions. Covered by the publish_zit tests
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up,
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again,
//! killed_between_the_steps_recovers_and_the_repeat_publishes_once,
//! with_no_landing_state_there_is_nothing_to_publish_and_the_remote_is_untouched,
//! publishing::an_open_push_intent_after_the_push_is_closed_done_on_the_repeat,
//! publishing::an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again and
//! publishing::an_open_push_intent_with_the_remote_past_new_is_closed_done (a push intent left open is settled
//! before the step is decided).

use crate::landing::{
    advance_origin, back_out_onto, decide_publish, fetch_origin, finish_publish, push_landing, read_landing_state,
    push_landed, remote_origin, settle_push_intent, LandingState, PublishAnswer, PublishError, PublishStep, PushResult,
};
use crate::squash::{read_tip, LandRefs};
use gitio::{Git, Oid};
use std::path::Path;

/// With no landing state: NothingToPublish, the remote not contacted. Otherwise, with expected = the state's
/// origin_at_start and new = its integration, the local origin tip decides (decide_publish), the remote read only
/// when the local origin is at expected. A remote at or past new (push_landed: new is its ancestor, so our push
/// landed and someone pushed on top) counts as at new. Once read, a push intent for this move left open by an
/// earlier run is closed from it first (settle_push_intent, S-103): done when the remote is at or past new, else
/// failed. Cleanup and BackOut read no remote: the run that moved the local origin had closed its push intent
/// already. Then: push new leased at expected, move the local origin to new, delete the zone's squash ref and remove
/// the state: Published { origin: new }. A remote already at or past new skips the push; a local origin already at
/// new does only the clean-up. A remote someone else moved (seen before the push or after
/// it was refused) is fetched, the local origin fast-forwarded to it and the landing backed out onto it:
/// OriginMoved { found }. A local origin at neither expected nor new (a publish killed after that fast-forward)
/// finishes the back-out onto it. A remote with no origin branch is RemoteMissing. The origin is never moved
/// backward: every local origin move is protected, the push leased.
pub fn publish(git: &Git, refs: &LandRefs, actor: &str) -> Result<PublishAnswer, PublishError> {
    let common = git.common_dir().map_err(PublishError::Git)?;
    let Some(state) = read_landing_state(&common).map_err(PublishError::State)? else {
        return Ok(PublishAnswer::NothingToPublish);
    };
    let local = read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(PublishError::Squash)?;
    let remote = if local == state.origin_at_start { read_remote(git, &common, refs, &state)? } else { None };
    match decide_publish(remote.as_ref(), &local, Some(&state)) {
        PublishStep::Nothing => Ok(PublishAnswer::NothingToPublish),
        PublishStep::Push => match push_landing(git, &common, refs, &state, actor)? {
            PushResult::Pushed => advanced(git, &common, refs, &state, actor),
            PushResult::Moved => origin_moved(git, &common, refs, &state, &local, actor),
        },
        PublishStep::Advance => advanced(git, &common, refs, &state, actor),
        PublishStep::Cleanup => {
            finish_publish(git, &common, &state, actor)?;
            Ok(PublishAnswer::Published { origin: state.integration })
        }
        PublishStep::FetchOrigin => origin_moved(git, &common, refs, &state, &local, actor),
        PublishStep::BackOut { onto } => backed_out_onto(git, refs, actor, onto),
        PublishStep::RemoteMissing => Err(PublishError::RemoteMissing {
            remote: refs.remote.clone(),
            refname: format!("refs/heads/{}", refs.origin_branch),
        }),
    }
}

/// The remote's origin as decide_publish takes it, after settling a push intent left open (settle_push_intent):
/// the integration commit when the push landed (push_landed), else the remote as read.
fn read_remote(git: &Git, common: &Path, refs: &LandRefs, state: &LandingState) -> Result<Option<Oid>, PublishError> {
    let remote = remote_origin(git, refs)?;
    let landed = push_landed(git, refs, remote.as_ref(), &state.origin_at_start, &state.integration)?;
    settle_push_intent(common, refs, state, remote.as_ref(), landed)?;
    Ok(if landed { Some(state.integration.clone()) } else { remote })
}

/// The remote is at or past new: step 2 moves the local origin there, then the clean-up: Published { origin: new }.
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
