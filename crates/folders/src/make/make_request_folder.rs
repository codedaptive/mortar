//! Makes a request's folder: its branch, cut from the zone branch's tip, and a sparse worktree on it.
//! No unit test: it takes a lock, appends to the op log, runs git and writes files. zone_branch_tests,
//! request_branch_tests, sparse_paths_tests, branch_op_tests, branch_step_after_tests, repeat_decision_tests,
//! skip_refill_tests, refill_cone_tests, branch_holder_tests and attach_head_request_tests cover its decisions.
//! The integration group is to cover, in a throwaway repository: a request folder is made on req/<zone>/<request>
//! at zone/<zone>'s tip, holds the cone and its record, and its branch op is in the op log; a second call
//! refills the folder (finishing a fill that was interrupted), answers the same record and logs no new op; a
//! repeat on a folder with unmerged entries returns the record and leaves the folder untouched; a
//! missing zone branch refuses with ZoneBranchMissing; a request branch already at another commit refuses with
//! BranchExists; a branch deleted after it was made, then make again at the same zone tip: the branch is made
//! again; a folders lock held by a live process refuses with FolderError::Lock and makes nothing.

use super::make_or_refill::make_or_refill;
use super::with_folders_lock::with_folders_lock;
use super::{FolderError, FolderSetup};
use crate::plan::{request_branch, sparse_paths, zone_branch};
use crate::record::{FolderKind, FolderRecord};

/// Makes the folder for `request` in `zone` at setup.path, holding `paths` (the work order's paths), and
/// returns its record. Under the folders lock, keyed `folders:make:req/<zone>/<request>`: reads zone/<zone>'s tip; when
/// the folder exists and differs only in its base and needed paths, refills it (untouched when its index has unmerged
/// entries) and answers its record; refuses a different folder, the request branch checked out elsewhere, and free disk
/// under the floor, all before the branch is made, so a refused folder leaves no branch; makes the branch through
/// make_branch; adds the worktree detached at the tip and points its HEAD at the branch; then writes the record and
/// fills the folder. The steps after reading the tip are make_or_refill's.
///
/// The caller runs oplog::recover first, as apply requires.
pub fn make_request_folder(
    setup: &FolderSetup,
    zone: &str,
    request: &str,
    paths: &[String],
) -> Result<FolderRecord, FolderError> {
    let zb = zone_branch(zone).map_err(FolderError::Plan)?;
    let rb = request_branch(zone, request).map_err(FolderError::Plan)?;
    let cone = sparse_paths(paths, &[]).map_err(FolderError::Plan)?;
    let git = setup.git;
    let common = git.common_dir().map_err(FolderError::Git)?;
    with_folders_lock(git, &common, &format!("folders:make:{rb}"), setup.pids, || {
        let tip = git
            .read_ref(&format!("refs/heads/{zb}"))
            .map_err(FolderError::Git)?
            .ok_or_else(|| FolderError::ZoneBranchMissing { branch: zb.clone() })?;
        let wanted = FolderRecord {
            kind: FolderKind::Request,
            zone: zone.to_string(),
            request: Some(request.to_string()),
            base: tip.clone(),
            work_order_paths: cone.clone(),
            needed_paths: Vec::new(),
            parent: Some(zb.clone()),
        };
        make_or_refill(setup, &common, wanted, &[], &rb, &tip, &cone)
    })
}
