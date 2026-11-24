//! LandingState::members: the batch's members in landing order. landing_state_members_tests covers it.

use crate::landing::{LandingMember, LandingState};
use std::iter::once;

impl LandingState {
    /// Every member, `earlier` then `newest`: the order they landed in.
    pub(crate) fn members(&self) -> impl Iterator<Item = &LandingMember> {
        self.earlier.iter().chain(once(&self.newest))
    }
}
