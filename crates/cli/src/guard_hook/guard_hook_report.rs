//! guard_hook_report: the report of the guard's verdict on one hook run.

use land::GuardVerdict;

use crate::report::{ExitCode, Report};

/// Allow is exit Done, outcome "allowed", with no lines, owner or next: the binary prints nothing for it
/// (prints_nothing), and the zero exit lets git go on. Refuse is exit Refused, outcome "refused", with the
/// verdict's why as the line, its owner and its next; the non-zero exit makes git abort the transaction or the push.
pub fn guard_hook_report(verdict: &GuardVerdict) -> Report {
    let (exit, outcome, lines, owner, next) = match verdict {
        GuardVerdict::Allow => (ExitCode::Done, "allowed", Vec::new(), None, String::new()),
        GuardVerdict::Refuse { why, owner, next } => {
            (ExitCode::Refused, "refused", vec![why.clone()], Some(owner.clone()), next.clone())
        }
    };
    Report { verb: "guard-hook".into(), exit, outcome: outcome.into(), lines, owner, next, fields: Vec::new() }
}
