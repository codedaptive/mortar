//! attempt_next: the one next step `mortar status` gives in an attempt folder (S-127(1)).

use super::StatusState;

/// Unsaved work is saved to the attempt's own branch with `mortar save "<sentence>"`, whatever else holds (a
/// stalled move or a paused join included), so the harness never weighs an attempt whose work is not saved. A saved
/// attempt with no work yet (its try branch still at its base) makes its change, then `mortar save` (S-145). Other
/// saved work waits for the harness, which keeps or drops the attempts. An attempt never joins or resolves: that is the
/// request's, after the harness keeps one attempt.
pub fn attempt_next(state: &StatusState) -> String {
    match (state.saved, state.no_work_yet) {
        (false, _) => "mortar save \"<sentence>\"",
        (true, true) => "make your change, then mortar save",
        (true, false) => "wait for the harness to keep or drop the attempts",
    }
    .to_string()
}
