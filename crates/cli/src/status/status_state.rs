//! StatusState: everything `mortar status` reports about one request folder, read before the report is made.
//! A bare type (S-7); status_next_tests, status_outcome_tests, status_lines_tests and status_fields_tests build
//! it, and crates/cli/tests/status_report_tests.rs renders it.

use std::path::PathBuf;

use ::join::PausedJoin;
use gitio::StatusPaths;
use snapshot::RefusedFile;

use crate::join::PendingMove;

/// The folder's `zone` and `request` names; its changed, untracked and conflicted `paths`; whether the folder is
/// `saved` (its tree equals its branch tip's, S-144) and the new files the size cap `refused`; whether the zone has commits the request has not taken in
/// (`zone_moved`) and whether the request is `joined` to the zone; the join `paused` on a conflict, if any; the
/// folder move a kill or a refused move left `stalled` (S-85, S-97, S-143), if any; the other open requests it
/// `overlaps` with, each with the shared paths (S-45); the needed paths outside the work order (`gaps`); and whether
/// the folder has `no_work_yet`: its branch still at its base and its tree the tip's (S-145).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusState {
    pub zone: String,
    pub request: String,
    pub paths: StatusPaths,
    pub saved: bool,
    pub refused: Vec<RefusedFile>,
    pub zone_moved: bool,
    pub joined: bool,
    pub paused: Option<PausedJoin>,
    pub stalled: Option<PendingMove>,
    pub overlaps: Vec<(String, Vec<PathBuf>)>,
    pub gaps: Vec<String>,
    pub no_work_yet: bool,
}
