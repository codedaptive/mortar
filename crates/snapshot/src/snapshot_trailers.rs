//! snapshot_trailers: the trailers a snapshot commit carries, built by oplog's mortar_trailer_pairs.

use oplog::{mortar_trailer_pairs, MortarTrailers};

use crate::SnapshotTarget;

/// The Mortar-Request then Mortar-Zone pairs for `target`, in that order; a snapshot commit carries no
/// Mortar-Change-Id, so change_id is None and mortar_trailer_pairs skips it.
pub(crate) fn snapshot_trailers(target: &SnapshotTarget) -> Vec<(String, String)> {
    mortar_trailer_pairs(&MortarTrailers {
        change_id: None,
        request: Some(target.request.clone()),
        zone: Some(target.zone.clone()),
    })
}
