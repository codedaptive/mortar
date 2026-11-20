//! missing_branch_report: the refusal of a verb whose zone or request branch does not exist.

use crate::report::{ExitCode, Report};

/// Exit 1 "refused", naming `refname`. A missing zone branch (`zone`) is the zone coordinator's to make; a missing
/// request branch is this request's, whose folder is made again. join_refusal_report words the same two cases
/// the same way.
pub fn missing_branch_report(verb: &str, refname: &str, zone: bool) -> Report {
    let (owner, next) = if zone {
        ("the zone's coordinator", format!("have the zone made, then mr {verb}"))
    } else {
        ("this request", format!("make the request folder again, then mr {verb}"))
    };
    Report {
        verb: verb.to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![format!("the branch {refname} does not exist")],
        owner: Some(owner.to_string()),
        next,
        fields: Vec::new(),
    }
}
