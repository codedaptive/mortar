//! Reading a probe's outcome from how run_in_group ended: exit 0 passes, other ends fail, a spawn failure stops.

use snapshot::GroupRunEnd;

use super::ProbeOutcome;

/// The probe outcome of one run_in_group end.
///
/// Exited with code 0 is Passed; any other code, or none (a signal), is Failed with that code. TimedOut is
/// TimedOut with its limit, which counts as a fail (S-78). SpawnFailed is neither a pass nor a fail: the suite's
/// verdict at that commit is unknown, so its error text is returned for bisect to stop on (S-78, #1146). The
/// command's stderr is dropped.
pub fn probe_outcome_of(end: GroupRunEnd) -> Result<ProbeOutcome, String> {
    match end {
        GroupRunEnd::Exited { code: Some(0), .. } => Ok(ProbeOutcome::Passed),
        GroupRunEnd::Exited { code, .. } => Ok(ProbeOutcome::Failed { code }),
        GroupRunEnd::TimedOut { limit, .. } => Ok(ProbeOutcome::TimedOut { limit }),
        GroupRunEnd::SpawnFailed { error } => Err(error),
    }
}
