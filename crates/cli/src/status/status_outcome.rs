//! status_outcome: the one word naming where a request folder stands.

use super::{StatusPhase, StatusState};

/// The word naming StatusPhase's phase for the state: "stalled" (a folder move a kill left undone), "paused" (a join
/// stopped on a conflict), "unsaved", "saved" (not yet joined) or "joined".
pub fn status_outcome(state: &StatusState) -> &'static str {
    match StatusPhase::from(state) {
        StatusPhase::Stalled => "stalled",
        StatusPhase::Paused => "paused",
        StatusPhase::Unsaved => "unsaved",
        StatusPhase::Saved => "saved",
        StatusPhase::Joined => "joined",
    }
}
