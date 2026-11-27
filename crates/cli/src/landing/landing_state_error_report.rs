//! The report of a landing verb stopped by land's landing state file, in the cli's own words: the io error's text
//! is never printed, only the path and, for a corrupt file, land's reason.

use land::LandingStateError;

use super::landing_refusal;
use crate::report::{failure_report, Report};

/// Read, Write and Remove: failure_report (exit 5) naming the path that failed. Corrupt: refused (exit 3, no
/// fields), owner "the landing in flight", next to mend or remove the file by hand, then repeat.
pub fn landing_state_error_report(verb: &str, error: &LandingStateError) -> Report {
    match error {
        LandingStateError::Read { path, .. } => failure_report(verb, &format!("reading {} failed", path.display())),
        LandingStateError::Write { path, .. } => failure_report(verb, &format!("writing {} failed", path.display())),
        LandingStateError::Remove { path, .. } => {
            failure_report(verb, &format!("removing {} failed", path.display()))
        }
        LandingStateError::Corrupt { path, reason } => landing_refusal(
            verb,
            vec![format!("{} is not a landing state Mortar wrote: {reason}", path.display())],
            "the landing in flight",
            format!("mend or remove {} by hand, then repeat the same command", path.display()),
        ),
    }
}
