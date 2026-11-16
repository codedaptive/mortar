//! status_next: the one next step `mortar status` gives in a request folder; an attempt folder's is attempt_next.

use super::{StatusPhase, StatusState};

/// The step for StatusPhase's phase of the state: a stalled folder move is finished by `mortar join`; a paused join
/// by `mortar resolve`; unsaved work by `mortar save "<sentence>"`; saved work not yet joined by `mortar join`; a
/// joined request waits for the zone's tests.
pub fn status_next(state: &StatusState) -> String {
    match StatusPhase::from(state) {
        StatusPhase::Stalled => "mortar join",
        StatusPhase::Paused => "mortar resolve",
        StatusPhase::Unsaved => "mortar save \"<sentence>\"",
        StatusPhase::Saved => "mortar join",
        StatusPhase::Joined => "wait for the zone's tests",
    }
    .to_string()
}
