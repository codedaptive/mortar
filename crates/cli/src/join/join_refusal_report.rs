//! The report of a join that join_on_base refused.

use ::join::{JoinError, JoinRunError};
use oplog::OplogError;

use crate::report::{failure_report, with_moved_first, ExitCode, Report};

/// A refusal the caller can act on is exit 3 "refused": the error's text, who owns the fix and the next step. A
/// missing zone, and a zone moved sideways during the join (the op log's Protected), are the zone coordinator's; a
/// missing or unrelated request, a conflicted path that is not UTF-8, and a request commit replay cannot take (a
/// merge, a root, no author date) are this request's. Anything else (git, the op log, a state file) is
/// failure_report. `moved` (moved_before's line), when given, is the first line of either report: it names the refs
/// the save or snapshot before the join moved, ahead of join's own text, whose "join moved no ref" speaks for the join
/// step alone.
pub fn join_refusal_report(error: &JoinRunError, moved: Option<&str>) -> Report {
    let report = refusal(error);
    Report { lines: with_moved_first(moved, report.lines), ..report }
}

/// The report for `error` alone.
fn refusal(error: &JoinRunError) -> Report {
    const REQUEST: &str = "this request";
    const COORDINATOR: &str = "the zone's coordinator";
    let (owner, next) = match error {
        JoinRunError::ZoneMissing(_) => (COORDINATOR, "have the zone made, then mr join".to_string()),
        JoinRunError::Oplog(OplogError::Protected { refname, .. }) => {
            (COORDINATOR, format!("ask the zone's coordinator why {refname} moved, then mr join"))
        }
        JoinRunError::RequestMissing(_) => (REQUEST, "make the request folder again, then mr join".to_string()),
        JoinRunError::Unrelated { .. } => (REQUEST, "cut the request again from the zone".to_string()),
        JoinRunError::StatePath(_) => (REQUEST, "rename that file to a UTF-8 name, then mr join".to_string()),
        JoinRunError::Replay(JoinError::MergeCommit(_) | JoinError::RootCommit(_) | JoinError::EmptyAuthorDate(_)) => (
            REQUEST,
            "rewrite the request's commits as one-parent commits with author dates, then mr join".to_string(),
        ),
        _ => return failure_report("join", &error.to_string()),
    };
    Report {
        verb: "join".to_string(),
        exit: ExitCode::Refused,
        outcome: "refused".to_string(),
        lines: vec![error.to_string()],
        owner: Some(owner.to_string()),
        next,
        fields: Vec::new(),
    }
}
