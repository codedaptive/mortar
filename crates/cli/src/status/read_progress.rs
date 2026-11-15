//! read_progress: the part of a request folder's state that says where its work stands, read without changing
//! anything (S-85).
//! Shell API: read_status and run_need call it.
//! No unit test: it runs git and reads the folder's files (S-5, S-104); is_joined_tests, move_stalled_tests and
//! status_next_tests cover the decisions made from what it reads, and the ZIT crates/cli/tests/status_joined_tests.rs
//! covers joined with the mortar binary in a throwaway repository: work snapshotted after a join but never saved
//! is not joined; a join with nothing new since, and a formatted save then a join, stay joined.

use ::join::paused_join;
use gitio::{Git, Oid, StatusPaths};
use snapshot::saved_state;

use super::{is_joined, read_branch_tips, read_stalled_move, StatusState};
use crate::config::SnapshotSettings;
use crate::context::FolderContext;
use crate::report::{failure_report, Report};
use crate::save::{save_error_report, snapshot_target_for};

/// The progress of `folder` for `actor`, with the zone tip it read: the request and zone tips (read_branch_tips: a
/// missing branch is its refusal); saved_state with the config's excludes and cap, as save snapshots (`settings`);
/// zone_moved when the zone tip is not in the request (the zone has commits the request has not taken in); joined
/// when the request tip is in the zone, the request has commits of its own and the folder's tree (saved_state's)
/// equals the tip's tree (is_joined); the paused join; the stalled folder move (read_stalled_move). It leaves
/// `paths` default and `overlaps` and `gaps` empty: status_next reads only the fields above, so it runs no path,
/// overlap or gap read; read_status fills them, reading overlaps against the returned zone tip. A save error is
/// save_error_report relabelled for `verb`; every other failure is failure_report. Reads only: no lock, no ref
/// move, no file written (S-85).
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
    let (request_ref, request_tip, zone_tip) = read_branch_tips(&git, &record.zone, request, verb)?;
    let saved = saved_state(&git, &folder.toplevel, &target, &settings.excludes, settings.cap_bytes)
        .map_err(|error| Report { verb: verb.to_string(), ..save_error_report(&error) })?;
    let ancestor = |a, b| git.is_ancestor(a, b).map_err(|error| fail(error.to_string()));
    let tip_tree = git.commit_info(&request_tip).map_err(|error| fail(error.to_string()))?.tree;
    let state = StatusState {
        zone: record.zone.clone(),
        request: request.to_string(),
        paths: StatusPaths::default(),
        saved: saved.saved,
        refused: saved.refused,
        zone_moved: !ancestor(&zone_tip, &request_tip)?,
        joined: is_joined(&record.base, &request_tip, ancestor(&request_tip, &zone_tip)?, &saved.tree, &tip_tree),
        paused: paused_join(&folder.git_dir).map_err(|error| fail(error.to_string()))?,
        stalled: read_stalled_move(&git, folder, &request_ref, verb)?,
        overlaps: Vec::new(),
        gaps: Vec::new(),
    };
    Ok((state, zone_tip))
}
