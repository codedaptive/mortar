//! How one probe's command ended, as bisect reads it. A bare type; probe_outcome_of_tests exercises it.

use std::time::Duration;

/// The verdict of the caller's command at one commit: exit 0 passes, anything else fails.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProbeOutcome {
    /// The command exited 0.
    Passed,
    /// The command exited with a code other than 0; `code` is None when a signal killed it.
    Failed { code: Option<i32> },
    /// The command was still running at `limit` and was killed; it counts as a fail, reported "timed out".
    TimedOut { limit: Duration },
}
