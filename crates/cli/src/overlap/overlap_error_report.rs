//! overlap_error_report: the report of an `mr overlap` whose read of the requests' changed files failed, one arm
//! per JoinError, in the cli's own words: gitio's Display is never printed (S-128 (4)).

use ::join::JoinError;

use crate::report::{failure_report, Report};

const VERB: &str = "overlap";

/// Every arm is a failure (exit 5): overlap reads only, so nothing half-done is left and a repeat is safe. join's
/// overlap returns only Git; the other arms are its replay's refusals, each worded naming its commit or ref.
pub fn overlap_error_report(error: &JoinError) -> Report {
    let what = match error {
        JoinError::Git(_) => "git failed while reading the requests' changed files".to_string(),
        JoinError::OntoMissing(commit) => format!("commit {commit} does not exist here"),
        JoinError::TipMissing(refname) => format!("{refname} does not exist"),
        JoinError::MergeCommit(commit) => format!("commit {commit} is a merge"),
        JoinError::RootCommit(commit) => format!("commit {commit} has no parent"),
        JoinError::EmptyAuthorDate(commit) => format!("commit {commit} has an empty author date"),
        JoinError::TipNotReplayed(refname) => format!("git gave no update for {refname}"),
    };
    failure_report(VERB, &what)
}
