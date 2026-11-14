//! save_change_id: whether a save reuses HEAD's change ID (S-34).

use crate::{SnapshotError, SnapshotTarget};
use gitio::Oid;
use oplog::{ChangeId, MortarTrailers};

/// When `head`'s Mortar-Request and Mortar-Zone are both `target`'s, HEAD is this request's own commit: its
/// Mortar-Change-Id is reused (Some), and a missing one gives ChangeIdMissing rather than a second ID for the
/// request. Otherwise (the zone's tip, another request's commit, or no trailers) None: the caller draws a new ID.
pub(crate) fn save_change_id(
    head: &Oid,
    head_trailers: &MortarTrailers,
    target: &SnapshotTarget,
) -> Result<Option<ChangeId>, SnapshotError> {
    let ours = head_trailers.request.as_deref() == Some(target.request.as_str())
        && head_trailers.zone.as_deref() == Some(target.zone.as_str());
    match (&head_trailers.change_id, ours) {
        (_, false) => Ok(None),
        (Some(id), true) => Ok(Some(id.clone())),
        (None, true) => Err(SnapshotError::ChangeIdMissing { head: head.clone() }),
    }
}
