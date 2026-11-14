//! push_landing: publish's step 1, the leased push of the landed commit to the remote's origin branch, inside its
//! own op's intent so MORTAR_OP rides on it and the guard's pre-push check finds the move listed (S-60).
//! No unit test: it writes the op log, pushes and reads the remote; publish_push_op builds the intent it
//! writes. Covered by the publish_zit cases 1 (publish moves remote and local: the done publish-push record) and
//! 3 (kill between the steps: the push runs once), 5 (a push refused with the remote still at expected) and 6 (a
//! remote moved while the push was refused).

use crate::landing::{publish_push_op, remote_origin, LandingState, PublishError, PushResult};
use crate::squash::LandRefs;
use gitio::{Git, PushLease, PushOutcome, PushRef, PushRefusal};
use oplog::{append_op_record, close_record, intent_record, new_op_id, op_git, unix_now, OpPhase, OpRecord, OpResult};
use std::path::Path;

/// Appends the intent publish_push_op builds from the state (expected = origin_at_start, new = integration), then
/// pushes new onto the remote's `refs/heads/<origin_branch>` leased at expected, with op_git's git. Pushed or up to
/// date: the intent is closed done (moved), and the answer is Pushed. Any refusal, or a git error (a guard's
/// pre-push refusal arrives as one): the remote is read again, and what it holds decides. At new, the push landed:
/// closed done (moved), Pushed. Otherwise the intent is closed failed (not moved, naming what the remote holds):
/// still at expected is the error (PushRefused for a refusal, Git for a git error); anything else, absent included,
/// answers Moved. An error appending, re-reading or closing is returned with the intent left as it stands.
pub(crate) fn push_landing(
    git: &Git,
    common: &Path,
    refs: &LandRefs,
    state: &LandingState,
    actor: &str,
) -> Result<PushResult, PublishError> {
    let expected = &state.origin_at_start;
    let refname = format!("refs/heads/{}", refs.origin_branch);
    let op = publish_push_op(refs, expected, &state.integration, &state.change_id, new_op_id(), actor, unix_now());
    let intent = intent_record(&op);
    append_op_record(common, &intent).map_err(PublishError::Oplog)?;
    let push_ref = PushRef {
        refname: refname.clone(),
        new: Some(state.integration.clone()),
        lease: PushLease::At(expected.clone()),
    };
    let failure = match op_git(git, &op.op_id).push(&refs.remote, &[push_ref]) {
        Ok(outcomes) => match outcomes.into_iter().find_map(refusal) {
            None => return pushed(common, &intent),
            Some(refusal) => PublishError::PushRefused { refname: refname.clone(), refusal },
        },
        Err(error) => PublishError::Git(error),
    };
    let remote = remote_origin(git, refs)?;
    if remote.as_ref() == Some(&state.integration) {
        return pushed(common, &intent);
    }
    let not_moved = OpResult::NotMoved { refs: vec![(refname, remote.clone())] };
    append_op_record(common, &close_record(&intent, OpPhase::Failed, not_moved, unix_now()))
        .map_err(PublishError::Oplog)?;
    if remote.as_ref() == Some(expected) {
        return Err(failure);
    }
    Ok(PushResult::Moved)
}

/// Closes the intent done (moved): the remote is at the landed commit.
fn pushed(common: &Path, intent: &OpRecord) -> Result<PushResult, PublishError> {
    let done = close_record(intent, OpPhase::Done, OpResult::Moved, unix_now());
    append_op_record(common, &done).map_err(PublishError::Oplog)?;
    Ok(PushResult::Pushed)
}

/// The refusal in one ref's push outcome; None when the ref was pushed or already up to date.
fn refusal(outcome: PushOutcome) -> Option<PushRefusal> {
    match outcome {
        PushOutcome::Refused(refusal) => Some(refusal),
        PushOutcome::Pushed | PushOutcome::UpToDate => None,
    }
}
