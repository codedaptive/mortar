//! settle_push_intent: on a repeat, publish closes a push intent an earlier run left open, from what the remote holds
//! now: recover leaves such an intent open, and only publish reads the remote.
//! No unit test: it reads and appends the op log. Its decision is push_intent_close (push_intent_close_tests).
//! Covered by the publish_zit tests publishing::an_open_push_intent_after_the_push_is_closed_done_on_the_repeat,
//! publishing::an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again,
//! publishing::an_open_push_intent_with_the_remote_past_new_is_closed_done,
//! repeats::open_push_intent_refuses_push_off_then_push_on_finishes and
//! kills.rs killed_between_the_steps_recovers_and_the_repeat_publishes_once (nothing left open, nothing written).

use crate::landing::{push_intent_close, read_push_intent, LandingState, PublishError, PushIntentState};
use crate::squash::LandRefs;
use gitio::Oid;
use oplog::close_intent;
use std::path::Path;

/// The push intent read_push_intent finds open for the state's move is closed once through close_intent with
/// push_intent_close(`remote`, `landed`, the origin ref): Done when the push landed (the remote at or past new, as
/// push_landed answers), else Failed naming what the remote holds. None open: nothing is written. Reading or
/// appending the log failing is Oplog.
pub(crate) fn settle_push_intent(
    common: &Path,
    refs: &LandRefs,
    state: &LandingState,
    remote: Option<&Oid>,
    landed: bool,
) -> Result<(), PublishError> {
    let PushIntentState::Open { op_id } = read_push_intent(common, refs, state)? else {
        return Ok(());
    };
    let refname = format!("refs/heads/{}", refs.origin_branch);
    close_intent(common, &op_id, push_intent_close(remote, landed, &refname)).map_err(PublishError::Oplog)?;
    Ok(())
}
