//! push_landing: publish's step 1, the leased push of the landed commit to the remote's origin branch, inside its
//! own op's intent so MORTAR_OP rides on it and the guard's pre-push check finds the move listed.
//! No sibling unit test: it writes the op log, pushes and reads the remote. Its decisions are push_refusal,
//! decide_refused_push and push_intent_close (push_refusal_tests, decide_refused_push_tests,
//! push_intent_close_tests); push_landed says whether the push landed; publish_push_op builds the intent it opens.
//! Covered by tests/publish_zit: publishing.rs publish_moves_the_remote_then_the_local_origin_and_cleans_up,
//! a_push_refused_with_the_remote_still_at_expected_is_push_refused_and_keeps_the_landing,
//! a_remote_moved_while_the_push_was_refused_answers_origin_moved and
//! an_open_push_intent_before_the_push_is_closed_failed_and_pushed_again (the second intent, closed done); kills.rs
//! killed_between_the_steps_recovers_and_the_repeat_publishes_once.

use crate::landing::{
    decide_refused_push, publish_push_op, push_intent_close, push_landed, push_refusal, remote_origin, LandingState,
    PublishError, PushResult,
};
use crate::squash::LandRefs;
use gitio::{Git, Oid, PushLease, PushRef};
use oplog::{close_intent, new_op_id, op_git, open_remote_intent, unix_now, RemoteIntentOutcome};
use std::path::Path;

/// Opens the intent publish_push_op builds from the state (expected = origin_at_start, new = integration) with
/// oplog's open_remote_intent, marked with `refs.remote`, then pushes new onto the remote's
/// `refs/heads/<origin_branch>` leased at expected, with op_git's git. An intent for this move already open or done
/// (publish settles open ones first, and a done push is followed by step 2, so neither is expected) is PushIntent
/// naming that record, and nothing is pushed. Pushed or up to date: the answer is Pushed. Any refusal (push_refusal
/// picks the first), or a git error (a guard's pre-push refusal arrives as one): the remote is read again, and
/// decide_refused_push decides from what it holds, a remote at or past new (push_landed) counting as at new: at new,
/// Pushed; at expected, the error (PushRefused for a refusal, Git for a git error); anything else, absent included,
/// Moved. The intent is closed once, through close_intent with push_intent_close: done exactly when the push landed
/// (pushed, or the remote at or past new). An error opening the intent is returned with nothing written; an error
/// re-reading the remote, fetching it or closing the intent is returned with the intent left open: in flight
/// and the next publish's settle_push_intent closes it from the remote.
pub(crate) fn push_landing(
    git: &Git,
    common: &Path,
    refs: &LandRefs,
    state: &LandingState,
    actor: &str,
) -> Result<PushResult, PublishError> {
    let expected = &state.origin_at_start;
    let new = &state.integration;
    let refname = format!("refs/heads/{}", refs.origin_branch);
    let op = publish_push_op(refs, expected, new, &state.change_id, new_op_id(), actor, unix_now());
    match open_remote_intent(git, common, &op, &refs.remote).map_err(PublishError::Oplog)? {
        RemoteIntentOutcome::Opened(_) => {}
        RemoteIntentOutcome::AlreadyOpen(found) | RemoteIntentOutcome::Repeated(found) => {
            return Err(PublishError::PushIntent { op_id: found.record.op_id, phase: found.record.phase })
        }
    }
    let push_ref = PushRef { refname: refname.clone(), new: Some(new.clone()), lease: PushLease::At(expected.clone()) };
    let (remote, landed, decided) = match op_git(git, &op.op_id).push(&refs.remote, &[push_ref]) {
        Ok(outcomes) => match push_refusal(outcomes) {
            None => (Some(new.clone()), true, Ok(PushResult::Pushed)),
            Some(refusal) => {
                refused(git, refs, state, PublishError::PushRefused { refname: refname.clone(), refusal })?
            }
        },
        Err(error) => refused(git, refs, state, PublishError::Git(error))?,
    };
    close_intent(common, &op.op_id, push_intent_close(remote.as_ref(), landed, &refname)).map_err(PublishError::Oplog)?;
    decided
}

type Refused = (Option<Oid>, bool, Result<PushResult, PublishError>);

/// The push did not go through as asked: the remote as read again, whether the push landed (push_landed), and what
/// decide_refused_push makes of it, given new for a remote at or past new.
fn refused(
    git: &Git,
    refs: &LandRefs,
    state: &LandingState,
    failure: PublishError,
) -> Result<Refused, PublishError> {
    let (expected, new) = (&state.origin_at_start, &state.integration);
    let remote = remote_origin(git, refs)?;
    let landed = push_landed(git, refs, remote.as_ref(), expected, new)?;
    let seen = if landed { Some(new) } else { remote.as_ref() };
    let decided = decide_refused_push(seen, expected, new, failure);
    Ok((remote, landed, decided))
}
