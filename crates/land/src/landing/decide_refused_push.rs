//! decide_refused_push: what a refused or failed push came to, from what the remote's origin branch holds after it.
//! decide_refused_push_tests covers every branch.

use crate::landing::{PublishError, PushResult};
use gitio::Oid;

/// In order: `remote` at `new`, the push landed: Pushed. At `expected`, the push's own `failure` stands and is
/// returned. Anything else, absent included: Moved.
pub(crate) fn decide_refused_push(
    remote: Option<&Oid>,
    expected: &Oid,
    new: &Oid,
    failure: PublishError,
) -> Result<PushResult, PublishError> {
    if remote == Some(new) {
        return Ok(PushResult::Pushed);
    }
    if remote == Some(expected) {
        return Err(failure);
    }
    Ok(PushResult::Moved)
}
