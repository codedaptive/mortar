//! hooked_squash_error_report: the report of mr squash when land's squash_with_hooks refused, in the cli's words.

use land::HookedSquashError;

use super::{landing_refusal, squash_error_report};
use crate::report::{hook_refusal_report, Report};

const NOTHING_MOVED: &str = "no squash commit was written and no ref moved";

/// - Squash: squash_error_report's report of the error.
/// - Refused: hook_refusal_report, with the line that no squash commit was written and no ref moved.
/// - Unreadable: refused (exit 3, no fields), naming the part of the hooked message that does not read back; the
///   hooked message itself is not printed, since a hook can make it any length.
pub(crate) fn hooked_squash_error_report(verb: &str, error: &HookedSquashError) -> Report {
    match error {
        HookedSquashError::Squash(error) => squash_error_report(verb, error),
        HookedSquashError::Refused(refusal) => hook_refusal_report(verb, refusal, NOTHING_MOVED),
        HookedSquashError::Unreadable { part, message: _ } => landing_refusal(
            verb,
            vec![
                format!("the commit hooks left a squash message whose {part} does not read back"),
                NOTHING_MOVED.to_string(),
            ],
            "whoever maintains the commit-msg hook",
            "repeat mr squash; if it fails again, the commit message part named is unreadable".to_string(),
        ),
    }
}
