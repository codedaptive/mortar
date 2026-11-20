//! A run step's verdict once its world is removed.

use crate::report::Verdict;

/// The body's own verdict when the removal succeeded; Fail when it did not. A successful removal never turns a
/// failed body into a pass, and a failed removal always fails the step.
pub fn step_verdict(body_verdict: Verdict, cleanup_ok: bool) -> Verdict {
    if cleanup_ok {
        body_verdict
    } else {
        Verdict::Fail
    }
}
