//! How a probe reads: one line naming the commit and the outcome, with "timed out" for a timeout (S-78).
//! No sibling unit test (S-104): every test of it runs gitio's Oid Display (S-101). Covered by tests/bisect_zit
//! (a_timed_out_probe_counts_as_a_fail, through the report's text); its unit cases are Nagatha's RITs (owed).

use std::fmt;
use std::time::Duration;

use super::{Probe, ProbeOutcome};

impl fmt::Display for Probe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let commit = &self.commit;
        match &self.outcome {
            ProbeOutcome::Passed => write!(f, "{commit}: passed"),
            ProbeOutcome::Failed { code: Some(code) } => write!(f, "{commit}: failed with exit code {code}"),
            ProbeOutcome::Failed { code: None } => write!(f, "{commit}: failed, killed by a signal"),
            ProbeOutcome::TimedOut { limit } => write!(f, "{commit}: failed, timed out after {}", limit_text(limit)),
        }
    }
}

/// A limit in whole seconds ("60s") when it is one, else in whole milliseconds ("1500ms"), so the text is
/// exact for the limits callers give and does not depend on Duration's Debug form.
fn limit_text(limit: &Duration) -> String {
    match limit.subsec_nanos() {
        0 => format!("{}s", limit.as_secs()),
        _ => format!("{}ms", limit.as_millis()),
    }
}
