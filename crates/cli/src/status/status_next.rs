//! status_next: the one next step `mr status` gives in a request folder; an attempt folder's is attempt_next.

use super::{StatusPhase, StatusState};
use crate::join::MoveCause;

/// The step for StatusPhase's phase of the state: a stalled folder move is finished by `mr undo` when an undo
/// left it and by `mr join` when a rebase did (S-143); a paused join by `mr resolve`; unsaved work by
/// `mr save "<sentence>"`; a saved folder with no work yet (its branch still at its base) by making its change,
/// then `mr save` (S-145); other saved work not yet joined by `mr join`; a joined request waits for the zone's
/// tests.
pub fn status_next(state: &StatusState) -> String {
    match StatusPhase::from(state) {
        StatusPhase::Stalled => match state.stalled.as_ref().map(|stalled| stalled.cause) {
            Some(MoveCause::Undo) => "mr undo",
            Some(MoveCause::Rebase) | None => "mr join",
        },
        StatusPhase::Paused => "mr resolve",
        StatusPhase::Unsaved => "mr save \"<sentence>\"",
        StatusPhase::Saved if state.no_work_yet => "make your change, then mr save",
        StatusPhase::Saved => "mr join",
        StatusPhase::Joined => "wait for the zone's tests",
    }
    .to_string()
}
