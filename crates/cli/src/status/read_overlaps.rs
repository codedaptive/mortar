//! read_overlaps: the other open requests of the zone that change a path this request changes since its fork point:
//! request_base's fork_point (the recorded base for a request cut with partners while it is an ancestor of the tip,
//! else the merge base with the zone).
//! No unit test: it runs git and reads folder records; open_requests_for_tests and overlaps_with_tests
//! cover its decisions. crates/cli/tests/status_need_runs/status_partners_tests.rs covers a request cut with a
//! partner, whose partner's changes are not counted as its own. The integration tests
//! crates/cli/tests/status_need_runs/status.rs cover, with the mortar binary
//! as a child process in request folders made by the landing verbs in a throwaway repository (tests running real git set
//! GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - two requests of one zone changing one path: status in either gives the overlap line naming the other and that
//!   path;
//! - a rebased request (joined after the zone moved) shows no overlap on paths only the zone changed;
//! - a worktree whose folder record cannot be read: status fails with failure_report naming that folder, never
//!   'no overlap'.

use std::path::PathBuf;

use ::join::overlap_for;
use folders::make::read_folder_record;
use folders::plan::{request_branch, snap_ref};
use folders::{FolderKind, FolderRecord};
use gitio::{Git, Oid};

use super::{open_requests_for, overlaps_with};
use crate::report::{failure_report, Report};
use crate::request_base::fork_point;

/// Every worktree but bare and prunable ones is read for its folder record (a worktree with none is not a Mortar
/// folder and is skipped); each record of `zone` naming a request gets its request branch's tip and its snapshot
/// ref's commit. open_requests_for picks the zone's open requests, each based at request_base's fork_point: the
/// recorded base for its Request folder cut with partners, else the merge base of its tip and `zone_tip`; join's
/// overlap_for gives the pairs holding `request`, and overlaps_with turns them into (the other request, the shared
/// paths). A request forked at the merge base and sharing no commit with the zone tip is an error, never "no
/// overlap". Any read failing is failure_report for `verb`. Reads only: no lock, no ref move.
pub fn read_overlaps(
    git: &Git,
    zone: &str,
    request: &str,
    zone_tip: &Oid,
    verb: &str,
) -> Result<Vec<(String, Vec<PathBuf>)>, Report> {
    let fail = |what: String| failure_report(verb, &what);
    let mut folders = Vec::new();
    let mut git_dirs = Vec::new();
    for worktree in git.worktree_list().map_err(|error| fail(error.to_string()))? {
        if worktree.bare || worktree.prunable {
            continue;
        }
        let git_dir = git.git_dir(&worktree.path).map_err(|error| fail(error.to_string()))?;
        let record = read_folder_record(&git_dir).map_err(|error| {
            fail(format!("the folder record of {} cannot be read: {error}", worktree.path.display()))
        })?;
        if let Some(record) = record {
            let (branch_tip, snapshot) = tips(git, &record, zone).map_err(fail)?;
            folders.push((record, branch_tip, snapshot));
            git_dirs.push(git_dir);
        }
    }
    let fork_at = |name: &str, tip: &Oid| {
        let folder = folders.iter().zip(&git_dirs).find(|((record, ..), _)| {
            record.kind == FolderKind::Request && record.zone == zone && record.request.as_deref() == Some(name)
        });
        let folder = folder.map(|((record, ..), git_dir)| (record, git_dir.as_path()));
        fork_point(git, folder, zone, tip, || Ok(zone_tip.clone()), |error| match error {
            None => format!("the request {name} shares no commit with the zone {zone}"),
            Some(error) => error.to_string(),
        })
    };
    let open = open_requests_for(zone, &folders, fork_at).map_err(fail)?;
    let pairs = overlap_for(git, &open, request).map_err(|error| fail(error.to_string()))?;
    Ok(overlaps_with(request, &pairs))
}

/// The request branch's tip and the snapshot ref's commit of a record of `zone` naming a request; None for both
/// otherwise, as open_requests_for skips such a record.
fn tips(git: &Git, record: &FolderRecord, zone: &str) -> Result<(Option<Oid>, Option<Oid>), String> {
    let Some(request) = record.request.as_deref().filter(|_| record.zone == zone) else {
        return Ok((None, None));
    };
    let branch = request_branch(zone, request).map_err(|error| error.reason)?;
    let snapshot = snap_ref(zone, request).map_err(|error| error.reason)?;
    let read = |refname: &str| git.read_ref(refname).map_err(|error| error.to_string());
    Ok((read(&format!("refs/heads/{branch}"))?, read(&snapshot)?))
}
