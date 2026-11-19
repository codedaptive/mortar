//! read_progress: the part of a request or attempt folder's state that says where its work stands, read without
//! changing anything. Saved means the folder's tree equals its branch tip's tree, not the snapshot ref's: kept
//! as a commit.
//! Shell API: read_status and run_need call it.
//! No unit test: it runs git and reads the folder's files; is_saved_tests, no_work_yet_tests,
//! is_joined_tests, move_stalled_tests and status_next_tests cover the decisions made from what it reads, and the validation test
//! crates/cli/tests/validation/status_joined_tests.rs covers joined with the mortar binary in a throwaway
//! repository: work snapshotted after a join but never saved is not joined; a join with nothing new since, and a
//! formatted save then a join, stay joined.

use ::join::paused_join;
use gitio::{Git, Oid, StatusPaths};
use snapshot::saved_state;

use super::{is_joined, is_saved, no_work_yet, progress_branch, read_stalled_move, read_tips, StatusState};
use crate::config::SnapshotSettings;
use crate::context::FolderContext;
use crate::report::{failure_report, Report};
use crate::save::{save_error_report, snapshot_target_for};

/// The progress of `folder` for `actor`, with the zone tip it read: the tips of the folder's own branch (the
/// request branch, or attempt k's try branch: progress_branch) and of the zone (read_tips: a missing branch is its
/// refusal); saved_state with the config's excludes and cap, as save snapshots (`settings`), for the folder's tree
/// and the files the cap `refused`; saved when that tree equals the branch tip's tree (is_saved); zone_moved
/// when the zone tip is not in the branch (the zone has commits the folder has not taken in); joined when the
/// branch tip is in the zone, the branch has commits of its own and the folder is saved (is_joined); no_work_yet
/// when the branch tip is still the record's base and the folder is saved; the paused join; the stalled
/// folder move of the branch (read_stalled_move). It leaves `paths`
/// default and `overlaps` and `gaps` empty: status_next reads only the fields above, so it runs no path, overlap
/// or gap read; read_status fills them, reading overlaps against the returned zone tip. A save error is
/// save_error_report relabelled for `verb`; every other failure is failure_report. Reads only: no lock, no ref
/// move, no file written.
pub fn read_progress(
    folder: &FolderContext,
    settings: &SnapshotSettings,
    actor: &str,
    verb: &str,
) -> Result<(StatusState, Oid), Report> {
    let fail = |what: String| failure_report(verb, &what);
    let record = &folder.record;
    let (Some(request), Some(target)) = (record.request.as_deref(), snapshot_target_for(record, actor)) else {
        return Err(fail("this folder's record names no request".to_string()));
    };
    let git = Git::new(&folder.toplevel);
    let (branch_ref, branch_tip, zone_tip) = read_tips(
        &git,
        &progress_branch(&record.zone, request, record.kind).map_err(|error| fail(error.reason))?,
        &record.zone,
        verb,
    )?;
    let saved = saved_state(&git, &folder.toplevel, &target, &settings.excludes, settings.cap_bytes)
        .map_err(|error| Report { verb: verb.to_string(), ..save_error_report(&error) })?;
    let ancestor = |a, b| git.is_ancestor(a, b).map_err(|error| fail(error.to_string()));
    let tip_tree = git.commit_info(&branch_tip).map_err(|error| fail(error.to_string()))?.tree;
    let state = StatusState {
        zone: record.zone.clone(),
        request: request.to_string(),
        paths: StatusPaths::default(),
        saved: is_saved(&saved.tree, &tip_tree),
        refused: saved.refused,
        zone_moved: !ancestor(&zone_tip, &branch_tip)?,
        joined: is_joined(&record.base, &branch_tip, ancestor(&branch_tip, &zone_tip)?, &saved.tree, &tip_tree),
        paused: paused_join(&folder.git_dir).map_err(|error| fail(error.to_string()))?,
        stalled: read_stalled_move(&git, folder, &branch_ref, verb)?,
        overlaps: Vec::new(),
        gaps: Vec::new(),
        no_work_yet: no_work_yet(&record.base, &branch_tip, &saved.tree, &tip_tree),
    };
    Ok((state, zone_tip))
}
