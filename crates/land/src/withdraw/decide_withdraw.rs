//! decide_withdraw: whether a zone's squash may be withdrawn, from what withdraw read. decide_withdraw_tests covers
//! every branch; the withdraw_zit tests the_squash_in_the_landing_state_is_refused_and_moves_nothing and
//! an_earlier_batch_member_cannot_be_withdrawn cover it through withdraw.

use crate::landing::LandingState;
use crate::withdraw::WithdrawError;
use gitio::Oid;

/// `commit` in origin (an ancestor of the origin tip `origin`) is Published, checked first; else a landing state with
/// a member, newest or earlier, whose squash is `commit` is Landing with that member's change ID; else Ok. A state
/// whose members all name other commits does not block: the caller holds lock `land`, so that state belongs to
/// other squashes.
pub(crate) fn decide_withdraw(
    zone: &str,
    commit: &Oid,
    origin: &Oid,
    in_origin: bool,
    state: Option<&LandingState>,
) -> Result<(), WithdrawError> {
    if in_origin {
        return Err(WithdrawError::Published { zone: zone.to_string(), commit: commit.clone(), origin: origin.clone() });
    }
    match state.and_then(|state| state.members().find(|member| member.squash == *commit)) {
        Some(member) => Err(WithdrawError::Landing {
            zone: zone.to_string(),
            commit: commit.clone(),
            change_id: member.change_id.clone(),
        }),
        None => Ok(()),
    }
}
