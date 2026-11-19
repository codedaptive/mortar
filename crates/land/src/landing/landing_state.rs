//! LandingState: the landing in flight, as `<common>/mortar/landing.json` holds it. A bare type;
//! tests/landing_zit exercises it through land and back_out, tests/publish_zit through publish.

use gitio::Oid;
use oplog::ChangeId;

/// The squash being landed (its change ID, zone and commit), the origin tip when the landing started, and the
/// commit the landing moves integration to (the squash itself for a fast-forward, else the merge commit). Written
/// before the op that moves integration; read by land (a repeat after a kill finds it), back_out and publish.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LandingState {
    pub change_id: ChangeId,
    pub zone: String,
    pub squash: Oid,
    pub origin_at_start: Oid,
    pub integration: Oid,
}
