//! decide_formatter_wait: whether a formatter run is done, may keep running, or must be killed at its limit.

use crate::formatter_wait::FormatterWait;
use std::time::Duration;

/// `exited` means the run is over (formatter reaped and its stderr at EOF). That is Done even past the limit: the
/// run ended by itself, so its real result is kept. A run not over at `elapsed` >= `limit` is Kill: stop waiting (a
/// zero limit stops any run not already over); otherwise Wait.
pub(crate) fn decide_formatter_wait(elapsed: Duration, limit: Duration, exited: bool) -> FormatterWait {
    if exited {
        return FormatterWait::Done;
    }
    if elapsed >= limit {
        return FormatterWait::Kill;
    }
    FormatterWait::Wait
}
