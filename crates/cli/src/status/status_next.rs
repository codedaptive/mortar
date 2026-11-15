//! status_next: the one next step `mortar status` gives.

use super::StatusState;

/// The first that holds, in the spec's order: a stalled folder move is finished by `mortar join`; a paused join by
/// `mortar resolve`; unsaved work by `mortar save "<sentence>"`; saved work not yet joined by `mortar join`; a
/// joined request waits for the zone's tests.
pub fn status_next(state: &StatusState) -> String {
    match (state.stalled.is_some(), state.paused.is_some(), state.saved, state.joined) {
        (true, ..) => "mortar join",
        (_, true, ..) => "mortar resolve",
        (_, _, false, _) => "mortar save \"<sentence>\"",
        (_, _, _, false) => "mortar join",
        _ => "wait for the zone's tests",
    }
    .to_string()
}
