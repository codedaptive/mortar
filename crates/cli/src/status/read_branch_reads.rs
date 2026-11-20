//! read_branch_reads: the git reads a request or attempt folder's joined answer is made from, shared by `mr status`
//! (read_progress) and `mr disk` (read_folder_holds).
//! No unit test: it runs git and builds the folder's tree; joined_from_reads_tests, is_saved_tests and
//! missing_branch_report_tests cover the decisions made from what it reads, and the integration tests
//! crates/cli/tests/status_need_runs/status_joined_tests.rs and
//! crates/cli/tests/across_kits/snapshot_overlap_disk_sweep/disk.rs reach it through status and disk.

use std::path::Path;

use folders::FolderRecord;
use gitio::Git;
use snapshot::saved_state;

use super::{progress_branch, read_tips, saved_state_report, BranchReads};
use crate::config::SnapshotSettings;
use crate::report::{failure_report, Report};
use crate::save::snapshot_target_for;

/// The reads for the folder at `toplevel` with `record`, for `actor`: the tips of the folder's own branch (the
/// request branch, or attempt k's try branch: progress_branch) and of the zone (read_tips: a missing branch is its
/// refusal); saved_state with the config's excludes and cap, as save snapshots (`settings`); the branch tip's tree;
/// and whether the zone holds the branch tip. Every run goes through `git`, so its env reaches each one. A record
/// naming no request is refused. Every failure is in the cli's words, never another crate's text: a request the plan
/// cannot name a branch for is "request <r> in zone <z> is not a name a branch can carry"; a commit git cannot read
/// is "git could not read the commit <oid>"; an ancestry git cannot answer is "git could not tell whether <branch
/// tip> is in the zone at <zone tip>"; saved_state's errors are saved_state_report's. Each is failure_report for
/// `verb` but saved_state's refusals. Moves no ref and takes no lock; saved_state writes the folder's tree objects.
pub fn read_branch_reads(
    git: &Git,
    toplevel: &Path,
    record: &FolderRecord,
    settings: &SnapshotSettings,
    actor: &str,
    verb: &str,
) -> Result<BranchReads, Report> {
    let fail = |what: String| failure_report(verb, &what);
    let zone = &record.zone;
    let (Some(request), Some(target)) = (record.request.as_deref(), snapshot_target_for(record, actor)) else {
        return Err(fail("this folder's record names no request".to_string()));
    };
    let (branch_ref, branch_tip, zone_tip) = read_tips(
        git,
        &progress_branch(zone, request, record.kind)
            .map_err(|_| fail(format!("request {request} in zone {zone} is not a name a branch can carry")))?,
        zone,
        verb,
    )?;
    let saved = saved_state(git, toplevel, &target, &settings.excludes, settings.cap_bytes)
        .map_err(|error| saved_state_report(&error, request, zone, verb))?;
    let tip_tree =
        git.commit_info(&branch_tip).map_err(|_| fail(format!("git could not read the commit {branch_tip}")))?.tree;
    let request_in_zone = git.is_ancestor(&branch_tip, &zone_tip).map_err(|_| {
        fail(format!("git could not tell whether {branch_tip} is in the zone at {zone_tip}"))
    })?;
    Ok(BranchReads { branch_ref, branch_tip, zone_tip, request_in_zone, saved, tip_tree })
}
