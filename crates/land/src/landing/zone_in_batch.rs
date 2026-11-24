//! zone_in_batch: the member of a landing batch holding a zone. zone_in_batch_tests covers it.

use crate::landing::{LandingMember, LandingState};

/// The first member, in landing order, whose zone is `zone`; None when no member holds it. A zone has one squash out,
/// so land_batch answers ZoneInBatch with this member rather than append a second squash of the same zone.
pub(crate) fn zone_in_batch<'a>(state: &'a LandingState, zone: &str) -> Option<&'a LandingMember> {
    state.members().find(|member| member.zone == zone)
}
