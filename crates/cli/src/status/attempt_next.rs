//! attempt_next: the one next step `mortar status` gives in an attempt folder (S-127(1)).

use super::StatusState;

/// Unsaved work is saved to the attempt's own branch with `mortar save "<sentence>"`, whatever else holds (a
/// stalled move or a paused join included), so the harness never weighs an attempt whose work is not saved. Saved
/// work waits for the harness, which keeps or drops the attempts. An attempt never joins or resolves: that is the
/// request's, after the harness keeps one attempt.
pub fn attempt_next(state: &StatusState) -> String {
    if state.saved { "wait for the harness to keep or drop the attempts" } else { "mortar save \"<sentence>\"" }
        .to_string()
}
