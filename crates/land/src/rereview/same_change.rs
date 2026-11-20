//! Whether two commits are versions of one change: both carry the same Mortar-Change-Id.
//! same_change_tests exercises it.

use gitio::Oid;
use oplog::{ChangeId, MortarTrailers};

use super::RereviewError;

/// The change ID `reviewed` and `new` share. NotSameChange, carrying both commits and both ids as found, when
/// the ids differ or either commit has none: two commits without an id are not known to be one change.
pub(crate) fn same_change(
    reviewed: &Oid,
    reviewed_trailers: &MortarTrailers,
    new: &Oid,
    new_trailers: &MortarTrailers,
) -> Result<ChangeId, RereviewError> {
    match (&reviewed_trailers.change_id, &new_trailers.change_id) {
        (Some(a), Some(b)) if a == b => Ok(a.clone()),
        (reviewed_id, new_id) => Err(RereviewError::NotSameChange {
            reviewed: reviewed.clone(),
            reviewed_id: reviewed_id.clone(),
            new: new.clone(),
            new_id: new_id.clone(),
        }),
    }
}
