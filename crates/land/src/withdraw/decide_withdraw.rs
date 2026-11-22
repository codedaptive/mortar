//! decide_withdraw: whether a zone's squash may be withdrawn, from what withdraw read.

use crate::landing::LandingState;
use crate::withdraw::WithdrawError;
use gitio::Oid;

/// `commit` in origin (an ancestor of the origin tip `origin`) is Published, checked first; else a landing state
/// whose squash is `commit` is Landing with the state's change ID; else Ok. A state naming another commit does not
/// block: the caller holds lock `land`, so that state belongs to another squash.
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
    match state {
        Some(state) if state.squash == *commit => Err(WithdrawError::Landing {
            zone: zone.to_string(),
            commit: commit.clone(),
            change_id: state.change_id.clone(),
        }),
        _ => Ok(()),
    }
}
