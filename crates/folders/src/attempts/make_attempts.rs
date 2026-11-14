//! Makes a request's attempt folders: attempt k on try/<zone>/<request>/<k>, cut from the request branch's tip, in a
//! sparse worktree at <dir>/try-<k> (S-69).
//! No unit test: it takes a lock, appends to the op log, runs git and writes files (S-5). request_branch_tests,
//! attempt_branch_tests, attempt_path_tests, sparse_paths_tests, plan_attempts_tests, branch_op_tests,
//! branch_step_after_tests, repeat_decision_tests, skip_refill_tests, refill_cone_tests, branch_holder_tests and
//! attach_head_request_tests cover its decisions. tests/folders/attempts_make.rs covers, in a throwaway repository:
//! the request branch and attempt branches try/<zone>/<request>/1..n all exist together, each folder at
//! <dir>/try-<k> on its branch at the request tip with an Attempt(k) record, and n folder-branch ops are logged; a
//! repeat makes nothing new and returns the same records; n = 0 is Plan; a missing request branch is
//! RequestBranchMissing; a folders lock held by a live pid is Lock.

use super::plan_attempts;
use crate::make::{make_or_refill, with_folders_lock, FolderError, FolderSetup};
use crate::plan::{attempt_path, request_branch, sparse_paths};
use crate::record::FolderRecord;

/// Makes `n` attempt folders for `request` in `zone`, each holding `paths` (the work order's paths), inside
/// setup.path, and returns their records in k order. The names and the cone are checked first. Under the folders
/// lock, keyed `folders:attempts:req/<zone>/<request>`: reads the request branch's tip (absent:
/// RequestBranchMissing), plans the attempts with plan_attempts (n == 0 is Plan), then makes or refills each in k
/// order through make_or_refill at attempt_path(setup.path, k), with setup's floor, sweep, disk, pids and actor. A
/// refusal at attempt j leaves attempts 1..j-1 made; a repeat finds those, refills them, and makes the rest.
///
/// The caller runs oplog::recover first, as apply requires.
pub fn make_attempts(
    setup: &FolderSetup,
    zone: &str,
    request: &str,
    paths: &[String],
    n: u32,
) -> Result<Vec<FolderRecord>, FolderError> {
    let rb = request_branch(zone, request).map_err(FolderError::Plan)?;
    let cone = sparse_paths(paths, &[]).map_err(FolderError::Plan)?;
    let git = setup.git;
    let common = git.common_dir().map_err(FolderError::Git)?;
    with_folders_lock(git, &common, &format!("folders:attempts:{rb}"), setup.pids, || {
        let tip = git
            .read_ref(&format!("refs/heads/{rb}"))
            .map_err(FolderError::Git)?
            .ok_or_else(|| FolderError::RequestBranchMissing { branch: rb.clone() })?;
        plan_attempts(zone, request, &tip, &cone, n)
            .map_err(FolderError::Plan)?
            .into_iter()
            .map(|planned| {
                let attempt = FolderSetup {
                    git,
                    path: attempt_path(&setup.path, planned.k),
                    floor_bytes: setup.floor_bytes,
                    sweep: setup.sweep.clone(),
                    disk: setup.disk,
                    pids: setup.pids,
                    actor: setup.actor.clone(),
                };
                make_or_refill(&attempt, &common, planned.record, &planned.branch, &tip, &cone)
            })
            .collect()
    })
}
