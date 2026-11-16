//! read_branch_tips: where a request's branch and its zone's branch stand, for `mortar need` (status reads them with
//! read_tips).
//! No unit test: it runs git (S-5, S-104); missing_branch_report_tests covers the refusal it gives. Nagatha's
//! integration group is to cover, with the mortar binary as a child process in a request folder made by the landing
//! verbs in a throwaway repository (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and
//! GIT_CONFIG_NOSYSTEM=1):
//! - status in a clean, saved and joined folder reads both tips and gives next "wait for the zone's tests";
//! - saved and not joined, status gives next mortar join;
//! - with the zone moved on, status says the zone has commits this request has not taken in and gives next mortar join;
//! - a missing request branch or a missing zone branch: status (and need) refuse with missing_branch_report's wording,
//!   exit 1, nothing changed.

use folders::plan::request_branch;
use gitio::{Git, Oid};

use super::read_tips;
use crate::report::{failure_report, Report};

/// The request branch's full name, its tip and the zone branch's tip (read_tips of request_branch). A missing
/// branch is missing_branch_report for `verb`; a name the plan refuses or a git failure is failure_report. Reads
/// only: it moves no ref.
pub fn read_branch_tips(git: &Git, zone: &str, request: &str, verb: &str) -> Result<(String, Oid, Oid), Report> {
    read_tips(git, &request_branch(zone, request).map_err(|error| failure_report(verb, &error.reason))?, zone, verb)
}
