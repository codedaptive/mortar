//! LandingState: the landing batch in flight, as `<common>/mortar/landing.json` holds it. A bare type;
//! tests/landing_zit exercises it through land and back_out, tests/publish_zit through publish.

use crate::landing::LandingMember;
use gitio::Oid;

/// The origin tip when the batch started and its members in landing order: `earlier` the members landed before the
/// newest (empty for a batch of one), so a state always has a member. Written before the op that moves integration;
/// read by land (a repeat after a kill finds it), back_out, publish and commit_files, each through
/// resolve_landing_state (publish through read_resolved_landing_state), and by read_landing_status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LandingState {
    pub origin_at_start: Oid,
    pub earlier: Vec<LandingMember>,
    pub newest: LandingMember,
    /// A commit commit_files wrote on top of the newest member's `integration`, written here before its op moves
    /// integration to it, so the op may not have run: resolve_landing_state takes it as the newest member's
    /// integration only when integration is at it.
    pub pending: Option<Oid>,
}
