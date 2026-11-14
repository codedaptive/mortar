//! publish: puts a landing on the remote and then on the local origin branch (SPEC-land 04-publish.md, S-16, S-29,
//! S-59, S-60). It takes no lock: the caller holds `land` and has run recover (S-16).
//! No unit test (S-5): it reads the landing state, runs git against the remote and writes the op log;
//! decide_publish_tests covers its decisions. Covered by the publish_zit tests
//! publish_moves_the_remote_then_the_local_origin_and_cleans_up,
//! a_remote_moved_by_someone_else_is_fetched_backed_out_onto_and_lands_again,
//! killed_between_the_steps_recovers_and_the_repeat_publishes_once and
//! with_no_landing_state_there_is_nothing_to_publish_and_the_remote_is_untouched.

use crate::landing::{
    advance_origin, back_out_onto, decide_publish, fetch_origin, finish_publish, push_landing, read_landing_state,
    remote_origin, LandingState, PublishAnswer, PublishError, PublishStep, PushResult,
};
use crate::squash::{read_tip, LandRefs};
use gitio::{Git, Oid};
use std::path::Path;

/// With no landing state: NothingToPublish, the remote not contacted. Otherwise, with expected = the state's
/// origin_at_start and new = its integration, the local origin tip decides (decide_publish), the remote read only
/// when the local origin is at expected: push new leased at expected, then move the local origin to new, delete the
/// zone's squash ref and remove the state: Published { origin: new }. A remote already at new skips the push; a
/// local origin already at new does only the clean-up. A remote someone else moved (seen before the push or after
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
    let remote = if local == state.origin_at_start { remote_origin(git, refs)? } else { None };
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

/// The remote is at new: step 2 moves the local origin there, then the clean-up: Published { origin: new }.
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
