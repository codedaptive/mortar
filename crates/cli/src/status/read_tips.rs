//! read_tips: where a folder's branch and its zone's branch stand, for `mr status` and `mr need`.
//! No unit test: it runs git; missing_branch_report_tests covers the refusal it gives, and the integration test
//! crates/cli/tests/status_need_runs/status_attempt_folder_tests.rs reads an attempt's try branch with it.

use folders::plan::zone_branch;
use gitio::{Git, Oid};

use super::missing_branch_report;
use crate::report::{failure_report, Report};

/// The full ref refs/heads/<branch>, its tip and the zone branch's tip. A missing branch is missing_branch_report
/// for `verb`. A zone name the plan refuses is failure_report "zone <zone> is not a name a branch can carry"; a ref
/// git cannot read is failure_report "git could not read <ref>". Both are the cli's words, never the plan's or git's
/// text. Reads only: it moves no ref.
pub fn read_tips(git: &Git, branch: &str, zone: &str, verb: &str) -> Result<(String, Oid, Oid), Report> {
    let branch_ref = format!("refs/heads/{branch}");
    let zone_ref = format!(
        "refs/heads/{}",
        zone_branch(zone)
            .map_err(|_| failure_report(verb, &format!("zone {zone} is not a name a branch can carry")))?
    );
    let branch_tip = tip(git, &branch_ref, false, verb)?;
    let zone_tip = tip(git, &zone_ref, true, verb)?;
    Ok((branch_ref, branch_tip, zone_tip))
}

/// The tip of the full ref `refname`, or the refusal naming it when it is absent (`zone` says whose branch it is).
fn tip(git: &Git, refname: &str, zone: bool, verb: &str) -> Result<Oid, Report> {
    git.read_ref(refname)
        .map_err(|_| failure_report(verb, &format!("git could not read {refname}")))?
        .ok_or_else(|| missing_branch_report(verb, refname, zone))
}
