//! The report of a join that join::join refused.

use ::join::{JoinError, JoinRunError};

use crate::report::{failure_report, ExitCode, Report};

/// A refusal the caller can act on is exit 1 "refused": the error's text, who owns the fix and the next step. A
/// missing zone is the zone coordinator's; a missing or unrelated request, a conflicted path that is not UTF-8,
/// and a request commit replay cannot take (a merge, a root, no author date) are this request's. Anything else
/// (git, the op log, a state file) is failure_report.
pub fn join_refusal_report(error: &JoinRunError) -> Report {
    const REQUEST: &str = "this request";
    let (owner, next) = match error {
        JoinRunError::ZoneMissing(_) => ("the zone's coordinator", "have the zone made, then mortar join"),
        JoinRunError::RequestMissing(_) => (REQUEST, "make the request folder again, then mortar join"),
        JoinRunError::Unrelated { .. } => (REQUEST, "cut the request again from the zone"),
        JoinRunError::StatePath(_) => (REQUEST, "rename that file to a UTF-8 name, then mortar join"),
        JoinRunError::Replay(JoinError::MergeCommit(_) | JoinError::RootCommit(_) | JoinError::EmptyAuthorDate(_)) => (
            REQUEST,
            "rewrite the request's commits as one-parent commits with author dates, then mortar join",
        ),
        _ => return failure_report("join", &error.to_string()),
    };
    Report {
        verb: "join".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![error.to_string()],
        owner: Some(owner.to_string()),
        next: next.to_string(),
        fields: Vec::new(),
    }
}
