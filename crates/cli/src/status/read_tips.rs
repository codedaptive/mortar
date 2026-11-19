//! read_tips: where a folder's branch and its zone's branch stand, for `mr status` and `mr need`.
//! No unit test: it runs git; missing_branch_report_tests covers the refusal it gives, and the RIT
//! crates/cli/tests/cli_rit/status_attempt_folder_tests.rs reads an attempt's try branch with it.

use folders::plan::zone_branch;
use gitio::{Git, Oid};

use super::missing_branch_report;
use crate::report::{failure_report, Report};

/// The full ref refs/heads/<branch>, its tip and the zone branch's tip. A missing branch is missing_branch_report
/// for `verb`; a zone name the plan refuses or a git failure is failure_report. Reads only: it moves no ref.
pub fn read_tips(git: &Git, branch: &str, zone: &str, verb: &str) -> Result<(String, Oid, Oid), Report> {
    let branch_ref = format!("refs/heads/{branch}");
    let zone_ref = format!("refs/heads/{}", zone_branch(zone).map_err(|error| failure_report(verb, &error.reason))?);
    let branch_tip = tip(git, &branch_ref, false, verb)?;
    let zone_tip = tip(git, &zone_ref, true, verb)?;
    Ok((branch_ref, branch_tip, zone_tip))
}

/// The tip of the full ref `refname`, or the refusal naming it when it is absent (`zone` says whose branch it is).
fn tip(git: &Git, refname: &str, zone: bool, verb: &str) -> Result<Oid, Report> {
    git.read_ref(refname)
        .map_err(|error| failure_report(verb, &error.to_string()))?
        .ok_or_else(|| missing_branch_report(verb, refname, zone))
}
