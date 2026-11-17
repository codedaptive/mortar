//! The report line a not-run test expects. Test-only: flow/not_run_tests.rs, kill/kill_not_run_tests.rs and
//! agent/agent_not_run_tests.rs build their expected lines with it.

use crate::report::Line;

/// A check line that did not hold, with `did` and `saw` as given.
pub fn failed_line(did: &str, saw: &str) -> Line {
    Line::Check { ok: false, did: did.to_string(), saw: saw.to_string() }
}
