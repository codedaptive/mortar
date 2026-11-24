//! batch_members: the landing batch's members as BatchMembers, in landing order. batch_members_tests covers it.

use crate::landing::{BatchMember, LandingState};

/// Every member's change ID and zone, `earlier` then `newest`: the order they landed in.
pub(crate) fn batch_members(state: &LandingState) -> Vec<BatchMember> {
    state
        .members()
        .map(|member| BatchMember { change_id: member.change_id.clone(), zone: member.zone.clone() })
        .collect()
}
