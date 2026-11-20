//! One bisect probe in the cli's words.

use land::{Probe, ProbeOutcome};

/// The commit probed and how the test command ended there: passed, failed with its exit code, failed by a signal,
/// or timed out after its limit in whole seconds, which counts as a fail.
pub fn probe_line(probe: &Probe) -> String {
    let commit = &probe.commit;
    match &probe.outcome {
        ProbeOutcome::Passed => format!("{commit}: passed"),
        ProbeOutcome::Failed { code: Some(code) } => format!("{commit}: failed with exit {code}"),
        ProbeOutcome::Failed { code: None } => format!("{commit}: failed, killed by a signal"),
        ProbeOutcome::TimedOut { limit } => {
            format!("{commit}: timed out after {} s, counted as a fail", limit.as_secs())
        }
    }
}
