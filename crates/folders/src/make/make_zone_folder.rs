//! Makes a zone's standing folder: a sparse worktree on zone/<zone>, making the branch through the op log when
//! it is new.
//! No unit test: it takes a lock, appends to the op log, runs git and writes files. zone_branch_tests,
//! sparse_paths_tests, zone_start_for_tests, branch_step_after_tests, repeat_decision_tests, skip_refill_tests,
//! refill_cone_tests, branch_holder_tests and attach_head_request_tests cover its decisions.
//! The integration group is to cover, in a throwaway repository: a zone folder on an existing zone/<zone> uses
//! the branch as it is, at its tip with HEAD on the branch, and logs no branch op; with no zone branch and a start
//! commit, the branch is made at the start by an op logged in mortar/oplog.jsonl; with neither, ZoneBranchMissing; a
//! second call refills the folder (finishing a fill that was interrupted) and answers the same record; a repeat on a
//! folder with unmerged entries returns the record and leaves the folder untouched; a folders lock held by a
//! live process refuses with FolderError::Lock and makes nothing.

use gitio::Oid;

use super::attach_head::attach_head;
use super::check_before_make::check_before_make;
use super::fill_folder::fill_folder;
use super::make_branch::make_branch;
use super::refill_folder::refill_folder;
use super::with_folders_lock::with_folders_lock;
use super::zone_start_for::zone_start_for;
use super::{FolderError, FolderSetup, ZoneStart};
use crate::plan::{sparse_paths, zone_branch};
use crate::record::{FolderKind, FolderRecord};

/// Makes `zone`'s folder at setup.path, holding `paths`, and returns its record. `paths` are the zone's paths, its
/// in-repo dependencies and the root build files, given by the caller. Under the folders lock, keyed
/// `folders:make:zone/<zone>`: the base is zone/<zone>'s tip, else `start`, else ZoneBranchMissing; when the folder
/// exists and differs only in its base and needed paths, refills it (untouched when its index has unmerged entries) and
/// answers its record; refuses a different folder, the zone branch checked out elsewhere, and free disk under the
/// floor, all before the branch or worktree is made. A missing zone branch is made at `start` through make_branch,
/// never by git's `worktree add -b`; an existing one is used as it is. The worktree is added detached at the base and
/// its HEAD pointed at the branch; then the record is written and the folder filled.
///
/// The caller runs oplog::recover first, as apply requires.
pub fn make_zone_folder(
    setup: &FolderSetup,
    zone: &str,
    paths: &[String],
    start: Option<&Oid>,
) -> Result<FolderRecord, FolderError> {
    let zb = zone_branch(zone).map_err(FolderError::Plan)?;
    let cone = sparse_paths(paths, &[]).map_err(FolderError::Plan)?;
    let git = setup.git;
    let common = git.common_dir().map_err(FolderError::Git)?;
    with_folders_lock(git, &common, &format!("folders:make:{zb}"), setup.pids, || {
        let tip = git.read_ref(&format!("refs/heads/{zb}")).map_err(FolderError::Git)?;
        let zone_start = zone_start_for(tip.as_ref(), start, &zb)?;
        let base = match &zone_start {
            ZoneStart::Existing(oid) | ZoneStart::Create(oid) => oid.clone(),
        };
        let wanted = FolderRecord {
            kind: FolderKind::Zone,
            zone: zone.to_string(),
            request: None,
            base: base.clone(),
            work_order_paths: cone.clone(),
            needed_paths: Vec::new(),
            parent: None,
        };
        if let Some(existing) = check_before_make(setup, &wanted, &zb)? {
            return refill_folder(git, &setup.path, existing);
        }
        if let ZoneStart::Create(_) = zone_start {
            make_branch(git, &setup.actor, &common, &zb, &base)?;
        }
        git.worktree_add(&setup.path, &base, None).map_err(FolderError::Git)?;
        attach_head(git, &setup.path, &zb)?;
        fill_folder(git, &setup.path, &wanted, &cone)?;
        Ok(wanted)
    })
}
