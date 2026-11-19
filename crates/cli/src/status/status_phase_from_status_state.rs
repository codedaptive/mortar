//! From<&StatusState> for StatusPhase: the first that holds wins, in this order: a stalled folder move first,
//! then a paused join, unsaved work, saved work not yet joined, and a joined request.

use super::{StatusPhase, StatusState};

impl From<&StatusState> for StatusPhase {
    fn from(state: &StatusState) -> Self {
        if state.stalled.is_some() {
            StatusPhase::Stalled
        } else if state.paused.is_some() {
            StatusPhase::Paused
        } else if !state.saved {
            StatusPhase::Unsaved
        } else if !state.joined {
            StatusPhase::Saved
        } else {
            StatusPhase::Joined
        }
    }
}
