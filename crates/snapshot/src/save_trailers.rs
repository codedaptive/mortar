//! save_trailers: the trailers a save commit carries, built by oplog's mortar_trailer_pairs.

use crate::SnapshotTarget;
use oplog::{mortar_trailer_pairs, ChangeId, MortarTrailers};

/// The Mortar-Change-Id, Mortar-Request and Mortar-Zone pairs for `change_id` and `target`, in that order.
pub(crate) fn save_trailers(change_id: &ChangeId, target: &SnapshotTarget) -> Vec<(String, String)> {
    mortar_trailer_pairs(&MortarTrailers {
        change_id: Some(change_id.clone()),
        request: Some(target.request.clone()),
        zone: Some(target.zone.clone()),
    })
}
