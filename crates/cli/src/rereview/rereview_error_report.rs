//! The report of an `mr rereview` that land's rereview stopped, in the cli's own words: land's, gitio's and
//! oplog's Display texts are never printed; commit ids and change ids are.
//! The Git and Trailers cases are repo integration tests to write (not written here): git itself must
//! fail to read a commit or produce the range-diff, or a commit's Mortar trailers must be unreadable, and the RIT
//! checks the cli's words are printed, not gitio's or oplog's.

use land::RereviewError;
use oplog::ChangeId;

use crate::report::{failure_report, ExitCode, OutValue, Report};

const VERB: &str = "rereview";
const NEXT: &str = "name two squash commits of one change: mr rereview <reviewed commit> <new commit> --actor <name>";

/// - Git: failure_report, "git could not read a commit or produce the range-diff".
/// - Trailers: failure_report, "a commit's Mortar trailers could not be read".
/// - NotSameChange: exit 1 Refused, outcome "not-same-change", line "<reviewed> carries <change X | no
///   Mortar-Change-Id> and <new> carries <change Y | no Mortar-Change-Id>; they are not one change", owner "the two
///   squash commits", next NEXT. Fields: reviewed and new, each the commit id.
/// - NoParent: exit 1 Refused, outcome "refused", line "<commit> is a root commit, so it has no parent to diff from",
///   owner "the commit <commit>", next NEXT. Fields: none.
pub fn rereview_error_report(error: &RereviewError) -> Report {
    match error {
        RereviewError::Git(_) => failure_report(VERB, "git could not read a commit or produce the range-diff"),
        RereviewError::Trailers(_) => failure_report(VERB, "a commit's Mortar trailers could not be read"),
        RereviewError::NotSameChange { reviewed, reviewed_id, new, new_id } => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Refused,
            outcome: "not-same-change".to_string(),
            lines: vec![format!(
                "{} carries {} and {} carries {}; they are not one change",
                reviewed.as_str(),
                carried(reviewed_id),
                new.as_str(),
                carried(new_id)
            )],
            owner: Some("the two squash commits".to_string()),
            next: NEXT.to_string(),
            fields: vec![
                ("reviewed".to_string(), OutValue::Text(reviewed.as_str().to_string())),
                ("new".to_string(), OutValue::Text(new.as_str().to_string())),
            ],
        },
        RereviewError::NoParent { commit } => Report {
            verb: VERB.to_string(),
            exit: ExitCode::Refused,
            outcome: "refused".to_string(),
            lines: vec![format!("{} is a root commit, so it has no parent to diff from", commit.as_str())],
            owner: Some(format!("the commit {}", commit.as_str())),
            next: NEXT.to_string(),
            fields: Vec::new(),
        },
    }
}

/// "change <id>", or "no Mortar-Change-Id" for a commit without one.
fn carried(id: &Option<ChangeId>) -> String {
    id.as_ref().map_or_else(|| "no Mortar-Change-Id".to_string(), |id| format!("change {}", id.as_str()))
}
