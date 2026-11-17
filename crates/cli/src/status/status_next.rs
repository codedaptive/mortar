//! status_next: the one next step `mortar status` gives in a request folder; an attempt folder's is attempt_next.

use super::{StatusPhase, StatusState};
use crate::join::MoveCause;

/// The step for StatusPhase's phase of the state: a stalled folder move is finished by `mortar undo` when an undo
/// left it and by `mortar join` when a rebase did (S-143); a paused join by `mortar resolve`; unsaved work by
/// `mortar save "<sentence>"`; saved work not yet joined by `mortar join`; a joined request waits for the zone's
/// tests.
pub fn status_next(state: &StatusState) -> String {
    match StatusPhase::from(state) {
        StatusPhase::Stalled => match state.stalled.as_ref().map(|stalled| stalled.cause) {
            Some(MoveCause::Undo) => "mortar undo",
            Some(MoveCause::Rebase) | None => "mortar join",
        },
        StatusPhase::Paused => "mortar resolve",
        StatusPhase::Unsaved => "mortar save \"<sentence>\"",
        StatusPhase::Saved => "mortar join",
        StatusPhase::Joined => "wait for the zone's tests",
    }
    .to_string()
}
