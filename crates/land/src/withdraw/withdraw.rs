//! withdraw: deletes a zone's squash ref before the squash lands. It takes no lock: the caller holds `land` and has
//! run recover.
//! No sibling unit test: it runs git, reads the landing state and writes the op log; decide_withdraw_tests cover
//! its decisions. Covered by the withdraw_zit tests
//! a_withdraw_deletes_the_squash_ref_and_answers_its_commit_and_change_id,
//! a_repeated_withdraw_is_absent_and_writes_nothing, a_published_squash_is_refused_and_moves_nothing,
//! the_squash_in_the_landing_state_is_refused_and_moves_nothing, an_earlier_batch_member_cannot_be_withdrawn and
//! squash_withdraw_squash_draws_a_new_change_id.

use crate::landing::{read_landing_state, resolve_landing_state};
use crate::squash::{read_prior_squash, read_tip, squash_ref, LandRefs};
use crate::withdraw::{decide_withdraw, withdraw_op, WithdrawAnswer, WithdrawError};
use gitio::Git;
use oplog::{apply, new_op_id, unix_now, ApplyOutcome, OplogError};

/// Reads the origin tip, then `refs/mortar/squash/<zone>` and its commit's Mortar-Change-Id: no squash ref answers
/// Absent and writes nothing. Then the landing state is read and, when there is one, resolved against integration's
/// tip (resolve_landing_state): a squash commit already in origin is refused as Published, and one that is a member
/// of the batch in the landing state, newest or earlier, as Landing, each with nothing moved and no op. Otherwise one
/// `land-withdraw` op deletes the squash ref, expecting it at the commit read, and moves no other ref: Withdrawn
/// with that commit and its change ID. The op finding the ref moved is OpFailed; a done op with its key is
/// OpRepeated.
pub fn withdraw(git: &Git, refs: &LandRefs, zone: &str, actor: &str) -> Result<WithdrawAnswer, WithdrawError> {
    let origin = read_tip(git, &format!("refs/heads/{}", refs.origin_branch)).map_err(WithdrawError::Squash)?;
    let refname = squash_ref(zone);
    let Some(prior) = read_prior_squash(git, &refname, &origin).map_err(WithdrawError::Squash)? else {
        return Ok(WithdrawAnswer::Absent);
    };
    let common = git.common_dir().map_err(WithdrawError::Git)?;
    let state = match read_landing_state(&common).map_err(WithdrawError::State)? {
        Some(state) => {
            let integration = read_tip(git, &format!("refs/heads/{}", refs.integration_branch))
                .map_err(WithdrawError::Squash)?;
            Some(resolve_landing_state(state, &integration))
        }
        None => None,
    };
    decide_withdraw(zone, &prior.commit, &origin, prior.in_origin, state.as_ref())?;
    let op = withdraw_op(&refname, &prior.commit, new_op_id(), actor, unix_now()).map_err(WithdrawError::Git)?;
    withdraw_applied(apply(git, &common, &op))?;
    Ok(WithdrawAnswer::Withdrawn { commit: prior.commit, change_id: prior.change_id })
}

/// Done is success; Repeated is OpRepeated and Failed is OpFailed, each with its record; an apply error is Oplog.
fn withdraw_applied(outcome: Result<ApplyOutcome, OplogError>) -> Result<(), WithdrawError> {
    match outcome.map_err(WithdrawError::Oplog)? {
        ApplyOutcome::Done(_) => Ok(()),
        ApplyOutcome::Repeated(record) => Err(WithdrawError::OpRepeated(record)),
        ApplyOutcome::Failed(record) => Err(WithdrawError::OpFailed(record)),
    }
}
