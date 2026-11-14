//! read_status: everything `mortar status` reports about a request folder, read without changing anything (S-85).
//! Shell API: run_status and run_need call it.
//! No unit test: it runs git and reads the folder's files (S-5, S-104); status_next_tests, status_outcome_tests,
//! status_lines_tests, status_fields_tests, is_joined_tests and move_stalled_tests cover the decisions made from
//! what it reads, and crates/cli/tests/status_report_tests.rs the report. Nagatha's integration group is to cover,
//! with the mortar binary as a child process in a request folder made by the landing verbs in a throwaway
//! repository (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - an unsaved edit gives next mortar save;
//! - a real paused join (Orc #1312) gives its paused commit and files, next mortar resolve;
//! - an over-cap new file is listed with the cap;
//! - a needed path outside the work order is listed as a gap;
//! - status writes no file and moves no ref: refs, the folder's index and the op log are unchanged after it.

use ::join::paused_join;
use folders::gaps;
use gitio::Git;
use snapshot::saved_state;

use super::{is_joined, read_branch_tips, read_overlaps, read_stalled_move, StatusState};
use crate::config::SnapshotSettings;
use crate::context::FolderContext;
use crate::report::{failure_report, Report};
use crate::save::{save_error_report, snapshot_target_for};

/// The state of `folder` for `actor`: the request and zone tips (read_branch_tips: a missing branch is its
/// refusal); the changed, untracked and conflicted paths; saved_state with the config's excludes and cap, as save
/// snapshots (`settings`); zone_moved when the zone tip is not in the request (the zone has commits the request has
/// not taken in); joined when the request tip is in the zone and the request has commits of its own (is_joined);
/// the paused join; the stalled folder move (read_stalled_move); the overlaps (read_overlaps, against each request's
/// fork point from the zone, S-108); the record's gaps. A save error is save_error_report relabelled for `verb`;
/// every other failure is failure_report. Reads only: no lock, no recover, no ref move (S-85).
pub fn read_status(
    folder: &FolderContext,
    settings: &SnapshotSettings,
    actor: &str,
    verb: &str,
) -> Result<StatusState, Report> {
    let fail = |what: String| failure_report(verb, &what);
    let record = &folder.record;
    let (Some(request), Some(target)) = (record.request.as_deref(), snapshot_target_for(record, actor)) else {
        return Err(fail("this folder's record names no request".to_string()));
    };
    let git = Git::new(&folder.toplevel);
    let (request_ref, request_tip, zone_tip) = read_branch_tips(&git, &record.zone, request, verb)?;
    let saved = saved_state(&git, &folder.toplevel, &target, &settings.excludes, settings.cap_bytes)
        .map_err(|error| Report { verb: verb.to_string(), ..save_error_report(&error) })?;
    let ancestor = |a, b| git.is_ancestor(a, b).map_err(|error| fail(error.to_string()));
    Ok(StatusState {
        zone: record.zone.clone(),
        request: request.to_string(),
        paths: git.status_paths(&folder.toplevel).map_err(|error| fail(error.to_string()))?,
        saved: saved.saved,
        refused: saved.refused,
        zone_moved: !ancestor(&zone_tip, &request_tip)?,
        joined: is_joined(&record.base, &request_tip, ancestor(&request_tip, &zone_tip)?),
        paused: paused_join(&folder.git_dir).map_err(|error| fail(error.to_string()))?,
        stalled: read_stalled_move(&git, folder, &request_ref, verb)?,
        overlaps: read_overlaps(&git, &record.zone, request, &zone_tip, verb)?,
        gaps: gaps(record),
    })
}
