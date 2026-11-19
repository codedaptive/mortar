//! read_status: everything `mr status` reports about a request or attempt folder, read without changing anything.
//! An attempt folder's progress is read from its own try branch and snapshot by read_progress; its
//! overlaps stay the request's.
//! Shell API: run_status calls it.
//! No unit test: it runs git and reads the folder's files; status_next_tests, status_outcome_tests,
//! status_lines_tests, status_fields_tests, is_joined_tests and move_stalled_tests cover the decisions made from
//! what it reads, and status_report_tests the report, in both rendered forms.
//! The repo integration tests are to cover, with the mortar binary as a child process in a request folder made by
//! the landing verbs in a throwaway repository (tests running real git set GIT_CONFIG_GLOBAL=/dev/null and
//! GIT_CONFIG_NOSYSTEM=1):
//! - an unsaved edit gives next mr save;
//! - a real paused join gives its paused commit and files, next mr resolve;
//! - an over-cap new file is listed with the cap;
//! - a needed path outside the work order is listed as a gap;
//! - work snapshotted after a join but never saved to the request branch is neither saved nor joined, next mr
//!   save (the validation test crates/cli/tests/validation/status_joined_tests.rs);
//! - status writes no file and moves no ref: refs, the folder's index and the op log are unchanged after it.

use folders::gaps;
use gitio::Git;

use super::{read_overlaps, read_progress, StatusState};
use crate::config::SnapshotSettings;
use crate::context::FolderContext;
use crate::report::{failure_report, Report};

/// The state of `folder` for `actor`: read_progress's (the folder's branch and zone tips, saved_state with the config's
/// excludes and cap as save snapshots it (`settings`), saved when the folder's tree equals the branch tip's,
/// zone_moved, joined when the branch tip is in the zone, the branch has commits of its own and the folder is
/// saved, no_work_yet, the paused join and the stalled folder move); then the changed, untracked and conflicted paths; the overlaps (read_overlaps, against
/// each request's fork point from the zone tip read_progress read); the record's gaps. A save error is
/// save_error_report relabelled for `verb`; every other failure is failure_report. Reads only: no lock, no
/// recover, no ref move.
pub fn read_status(
    folder: &FolderContext,
    settings: &SnapshotSettings,
    actor: &str,
    verb: &str,
) -> Result<StatusState, Report> {
    let (progress, zone_tip) = read_progress(folder, settings, actor, verb)?;
    let git = Git::new(&folder.toplevel);
    let record = &folder.record;
    Ok(StatusState {
        paths: git.status_paths(&folder.toplevel).map_err(|error| failure_report(verb, &error.to_string()))?,
        overlaps: read_overlaps(&git, &record.zone, &progress.request, &zone_tip, verb)?,
        gaps: gaps(record),
        ..progress
    })
}
