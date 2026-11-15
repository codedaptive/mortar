//! status_outcome: the one word naming where a request folder stands.

use super::StatusState;

/// The first that holds, in status_next's order: "stalled" (a folder move a kill left undone), "paused" (a join
/// stopped on a conflict), "unsaved", "saved" (not yet joined), else "joined".
pub fn status_outcome(state: &StatusState) -> &'static str {
    match (state.stalled.is_some(), state.paused.is_some(), state.saved, state.joined) {
        (true, ..) => "stalled",
        (_, true, ..) => "paused",
        (_, _, false, _) => "unsaved",
        (_, _, _, false) => "saved",
        _ => "joined",
    }
}
