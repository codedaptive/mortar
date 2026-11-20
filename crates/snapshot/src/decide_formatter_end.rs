//! decide_formatter_end: whether a formatter run ends this poll, and how.

use crate::formatter_end::FormatterEnd;
use crate::formatter_wait::FormatterWait;

/// None while `step` is Wait. Done is Exited (Done is given only once the leader is reaped and its stderr at EOF).
/// Kill with the leader reaped is ExitedPipeHeld: a reaped leader cannot be killed, so its own exit is the result.
/// Kill with the leader not reaped is TimedOut.
pub(crate) fn decide_formatter_end(step: FormatterWait, leader_reaped: bool) -> Option<FormatterEnd> {
    match (step, leader_reaped) {
        (FormatterWait::Wait, _) => None,
        (FormatterWait::Done, _) => Some(FormatterEnd::Exited),
        (FormatterWait::Kill, true) => Some(FormatterEnd::ExitedPipeHeld),
        (FormatterWait::Kill, false) => Some(FormatterEnd::TimedOut),
    }
}
