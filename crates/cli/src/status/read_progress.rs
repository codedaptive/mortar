//! read_progress: the part of a request or attempt folder's state that says where its work stands, read without
//! changing anything. Saved means the folder's tree equals its branch tip's tree, not the snapshot ref's: kept
//! as a commit.
//! Shell API: read_status and run_need call it.
//! No unit test: it runs git and reads the folder's files; is_saved_tests, no_work_yet_tests, is_joined_tests,
//! joined_from_reads_tests, move_stalled_tests and status_next_tests cover the decisions made from what it reads, and
//! the integration test crates/cli/tests/status_need_runs/status_joined_tests.rs covers joined with the mortar binary in
//! a throwaway repository: work snapshotted after a join but never saved is not joined; a join with nothing new since, and a
//! formatted save then a join, stay joined.

use ::join::paused_join;
use gitio::{Git, Oid, StatusPaths};

use super::{is_saved, joined_from_reads, no_work_yet, read_branch_reads, read_stalled_move, StatusState};
use crate::config::SnapshotSettings;
use crate::context::FolderContext;
use crate::report::{failure_report, Report};

/// The progress of `folder` for `actor`, with the zone tip it read. read_branch_reads gives the tips of the folder's
/// own branch and of the zone, saved_state's answer with the config's excludes and cap (`settings`: the folder's
/// tree and the files the cap `refused`), the branch tip's tree and whether the zone holds the branch tip. From
/// those: saved when the folder's tree equals the branch tip's tree (is_saved); zone_moved when the zone tip is not
/// in the branch (the zone has commits the folder has not taken in); joined by joined_from_reads (is_joined:
/// the branch tip is in the zone, the branch has commits of its own and the folder is saved); no_work_yet when the
/// branch tip is still the record's base and the folder is saved; the paused join; the stalled folder move of the
/// branch (read_stalled_move). It leaves `paths` default and `overlaps` and `gaps` empty: status_next reads only
/// the fields above, so it runs no path, overlap or gap read; read_status fills them, reading overlaps against the
/// returned zone tip. A record naming no request is refused and a save error is saved_state_report's, both
/// for `verb` from read_branch_reads; every other failure is failure_report. Moves no ref, takes no lock
/// and writes no file in the folder.
pub fn read_progress(
    folder: &FolderContext,
    settings: &SnapshotSettings,
    actor: &str,
    verb: &str,
) -> Result<(StatusState, Oid), Report> {
    let fail = |what: String| failure_report(verb, &what);
    let record = &folder.record;
    let git = Git::new(&folder.toplevel);
    let reads = read_branch_reads(&git, &folder.toplevel, record, settings, actor, verb)?;
    let zone_moved = !git.is_ancestor(&reads.zone_tip, &reads.branch_tip).map_err(|error| fail(error.to_string()))?;
    let state = StatusState {
        zone: record.zone.clone(),
        request: record.request.clone().unwrap_or_default(),
        paths: StatusPaths::default(),
        saved: is_saved(&reads.saved.tree, &reads.tip_tree),
        refused: reads.saved.refused.clone(),
        zone_moved,
        joined: joined_from_reads(&record.base, &reads),
        paused: paused_join(&folder.git_dir).map_err(|error| fail(error.to_string()))?,
        stalled: read_stalled_move(&git, folder, &reads.branch_ref, verb)?,
        overlaps: Vec::new(),
        gaps: Vec::new(),
        no_work_yet: no_work_yet(&record.base, &reads.branch_tip, &reads.saved.tree, &reads.tip_tree),
    };
    Ok((state, reads.zone_tip))
}
