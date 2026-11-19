//! ZoneRequests: one zone and its open requests, as after_publish's caller hands them in. A declaration only;
//! tests/after_publish_zit (zone_moves.rs) exercises it in every case.

use join::OpenRequest;

/// The zone's name (its branch is `refs/heads/zone/<zone>`) and the open requests the caller knows of in it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZoneRequests {
    pub zone: String,
    pub requests: Vec<OpenRequest>,
}
