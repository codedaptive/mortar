//! read_named_zones: each named zone with its zone folder and its open requests, read from the worktrees' folder
//! records, each request forked at its recorded base when it was cut with partners and that base is an ancestor of
//! its tip, else at the merge base with the zone (request_base's fork_point). Two callers: run_after_publish, for
//! the zones on its line, before land's call; and run_overlap, for its one zone, which uses only the open requests
//! (relabelling a failure report to its own verb).
//! No sibling test: it runs git and reads the folders' records; status's open_requests_for_tests covers the
//! choice of open requests, and crates/cli/tests/status_need_runs/status_partners_tests.rs
//! (partner_overlap_verb_forks_at_recorded_base) the overlap verb on a request cut with a partner, whose partner's
//! changes are not counted as its own.
//! The integration tests crates/cli/tests/across_kits/after_publish/requests.rs (after-publish) and
//! crates/cli/tests/across_kits/snapshot_overlap_disk_sweep/overlap.rs (overlap) cover, with the mortar binary:
//! - the open requests of a moved zone, request folders made by the landing verbs, are listed to rebase at their
//!   next join, each with its snapshot commit when it has one;
//! - a zone with no zone folder is reported as having none, and its requests are still listed;
//! - a request sharing no commit with its zone, and a request whose zone branch is missing, stop after-publish
//!   before land's call, and overlap before join's overlap, exit 5, naming the request or the ref;
//! - a worktree whose folder record cannot be read stops after-publish before land's call, and overlap before
//!   join's overlap, naming that folder.

use std::path::{Path, PathBuf};

use folders::make::read_folder_record;
use folders::plan::{request_branch, snap_ref, PlanError};
use folders::{FolderKind, FolderRecord};
use gitio::{Git, Oid};

use super::NamedZone;
use crate::report::{failure_report, Report};
use crate::request_base::fork_point;
use crate::status::open_requests_for;

const VERB: &str = "after-publish";

/// In `zones` order, each zone's NamedZone. Every worktree but bare and prunable ones is read for its folder record
/// (one with none is not a Mortar folder). A zone's folder is the first Zone record of it in git's worktree order;
/// its open requests are open_requests_for over its Request records, each with its request branch's tip and its
/// snapshot ref's commit, based at request_base's fork_point: the recorded base of a request cut with partners, else
/// the merge base of its tip and refs/heads/zone/<zone>'s tip, which is read only when a request needs it.
/// After-publish's answer is the same either way: land lists the requests whose base is not the zone's new tip
/// (requests_to_rebase), and a recorded merge of a zone and its partners is never a zone tip. Every failure is
/// failure_report in the cli's words. Reads only.
pub fn read_named_zones(git: &Git, zones: &[String]) -> Result<Vec<NamedZone>, Report> {
    let records = read_records(git)?;
    zones.iter().map(|zone| named_zone(git, zone, &records)).collect()
}

/// Each worktree's path, git dir and folder record, in git's order, skipping bare and prunable worktrees and those
/// with none.
fn read_records(git: &Git) -> Result<Vec<(PathBuf, PathBuf, FolderRecord)>, Report> {
    let fail = |what: String| failure_report(VERB, &what);
    let worktrees =
        git.worktree_list().map_err(|_| fail("git could not list the repository's worktrees".to_string()))?;
    let mut records = Vec::new();
    for worktree in worktrees.into_iter().filter(|worktree| !worktree.bare && !worktree.prunable) {
        let folder = worktree.path.display().to_string();
        let git_dir =
            git.git_dir(&worktree.path).map_err(|_| fail(format!("git could not find the git dir of {folder}")))?;
        let record =
            read_folder_record(&git_dir).map_err(|_| fail(format!("the folder record of {folder} cannot be read")))?;
        records.extend(record.map(|record| (worktree.path, git_dir, record)));
    }
    Ok(records)
}

/// `zone`'s folder and open requests from `records`.
fn named_zone(git: &Git, zone: &str, records: &[(PathBuf, PathBuf, FolderRecord)]) -> Result<NamedZone, Report> {
    let fail = |what: String| failure_report(VERB, &what);
    let request_folders: Vec<_> =
        records.iter().filter(|(.., record)| record.kind == FolderKind::Request && record.zone == zone).collect();
    let mut folders = Vec::new();
    for (path, _, record) in &request_folders {
        let (branch_tip, snapshot) = request_tips(git, record, zone, path).map_err(fail)?;
        folders.push((record.clone(), branch_tip, snapshot));
    }
    let zone_ref = format!("refs/heads/zone/{zone}");
    let mut zone_tip = None;
    let requests = open_requests_for(zone, &folders, |name, tip| {
        let folder = request_folders.iter().find(|(.., record)| record.request.as_deref() == Some(name));
        let folder = folder.map(|(_, git_dir, record)| (record, git_dir.as_path()));
        fork_point(git, folder, zone, tip, || read_zone_tip(git, &zone_ref, &mut zone_tip), |error| match error {
            None => format!("the request {name} shares no commit with zone {zone}"),
            Some(_) => format!("git could not find where the request {name} forked from zone {zone}"),
        })
    })
    .map_err(fail)?;
    Ok(NamedZone {
        zone: zone.to_string(),
        folder: records
            .iter()
            .find(|(.., record)| record.kind == FolderKind::Zone && record.zone == zone)
            .map(|(path, ..)| path.clone()),
        requests,
    })
}

/// The tip of `zone_ref`, read into `zone_tip` on the first call that needs it and taken from there after.
fn read_zone_tip(git: &Git, zone_ref: &str, zone_tip: &mut Option<Oid>) -> Result<Oid, String> {
    if let Some(known) = zone_tip {
        return Ok(known.clone());
    }
    match git.read_ref(zone_ref) {
        Ok(Some(read)) => Ok(zone_tip.insert(read).clone()),
        Ok(None) => Err(format!("{zone_ref} does not exist")),
        Err(_) => Err(format!("git could not read {zone_ref}")),
    }
}

/// The request branch's tip and the snapshot ref's commit of a Request record of `zone` at `path`; None for both
/// when the record names no request, which open_requests_for skips.
fn request_tips(
    git: &Git,
    record: &FolderRecord,
    zone: &str,
    path: &Path,
) -> Result<(Option<Oid>, Option<Oid>), String> {
    let Some(request) = record.request.as_deref() else {
        return Ok((None, None));
    };
    let unusable =
        |_: PlanError| format!("the folder record of {} names a request Mortar has no branch for", path.display());
    let branch = format!("refs/heads/{}", request_branch(zone, request).map_err(unusable)?);
    let snapshot = snap_ref(zone, request).map_err(unusable)?;
    let read = |refname: &str| git.read_ref(refname).map_err(|_| format!("git could not read {refname}"));
    Ok((read(&branch)?, read(&snapshot)?))
}
