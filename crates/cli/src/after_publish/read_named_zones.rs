//! read_named_zones: each named zone with its zone folder and its open requests, read from the worktrees' folder
//! records. Two callers: run_after_publish, for the zones on its line, before land's call; and
//! run_overlap, for its one zone, which uses only the open requests (relabelling a failure report to its own verb).
//! No sibling test: it runs git and reads the folders' records; status's open_requests_for_tests covers the
//! choice of open requests.
//! Repo integration tests to write (cross-zone, not written here):
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
use crate::status::open_requests_for;

const VERB: &str = "after-publish";

/// In `zones` order, each zone's NamedZone. Every worktree but bare and prunable ones is read for its folder record
/// (one with none is not a Mortar folder). A zone's folder is the first Zone record of it in git's worktree order;
/// its open requests are open_requests_for over its Request records, each with its request branch's tip and its
/// snapshot ref's commit, based at the merge base of its tip and refs/heads/zone/<zone>'s tip, which is read only
/// when the zone has a request to base. Every failure is failure_report in the cli's words. Reads only.
pub fn read_named_zones(git: &Git, zones: &[String]) -> Result<Vec<NamedZone>, Report> {
    let records = read_records(git)?;
    zones.iter().map(|zone| named_zone(git, zone, &records)).collect()
}

/// Each worktree's path and folder record, in git's order, skipping bare and prunable worktrees and those with none.
fn read_records(git: &Git) -> Result<Vec<(PathBuf, FolderRecord)>, Report> {
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
        records.extend(record.map(|record| (worktree.path, record)));
    }
    Ok(records)
}

/// `zone`'s folder and open requests from `records`.
fn named_zone(git: &Git, zone: &str, records: &[(PathBuf, FolderRecord)]) -> Result<NamedZone, Report> {
    let fail = |what: String| failure_report(VERB, &what);
    let mut folders = Vec::new();
    for (path, record) in records.iter().filter(|(_, record)| record.kind == FolderKind::Request && record.zone == zone)
    {
        let (branch_tip, snapshot) = request_tips(git, record, zone, path).map_err(fail)?;
        folders.push((record.clone(), branch_tip, snapshot));
    }
    let mut zone_tip = None;
    let requests = open_requests_for(zone, &folders, |name, tip| fork_point(git, zone, &mut zone_tip, name, tip))
        .map_err(fail)?;
    Ok(NamedZone {
        zone: zone.to_string(),
        folder: records
            .iter()
            .find(|(_, record)| record.kind == FolderKind::Zone && record.zone == zone)
            .map(|(path, _)| path.clone()),
        requests,
    })
}

/// The merge base of the request `name`'s `tip` and the zone's tip, which is read into `zone_tip` on the first call.
fn fork_point(git: &Git, zone: &str, zone_tip: &mut Option<Oid>, name: &str, tip: &Oid) -> Result<Oid, String> {
    let zone_ref = format!("refs/heads/zone/{zone}");
    let zone_tip = match zone_tip.clone() {
        Some(known) => known,
        None => match git.read_ref(&zone_ref) {
            Ok(Some(read)) => zone_tip.insert(read).clone(),
            Ok(None) => return Err(format!("{zone_ref} does not exist")),
            Err(_) => return Err(format!("git could not read {zone_ref}")),
        },
    };
    match git.merge_base(tip, &zone_tip) {
        Ok(Some(base)) => Ok(base),
        Ok(None) => Err(format!("the request {name} shares no commit with zone {zone}")),
        Err(_) => Err(format!("git could not find where the request {name} forked from zone {zone}")),
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
