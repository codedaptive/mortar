//! Makes one folder on its branch at a tip, or refills it when a repeat finds it already there: the steps
//! make_request_folder and make_attempts run under the folders lock once they know the record they want.
//! No unit test: it appends to the op log, runs git and writes files (S-5). repeat_decision_tests,
//! branch_holder_tests, floor_check_tests, branch_op_tests, branch_step_after_tests, skip_refill_tests,
//! refill_cone_tests and attach_head_request_tests cover its decisions. Nagatha's integration group is to cover it
//! through make_request_folder's and make_attempts' cases.

use std::path::Path;

use gitio::Oid;

use super::attach_head::attach_head;
use super::check_before_make::check_before_make;
use super::fill_folder::fill_folder;
use super::make_branch::make_branch;
use super::refill_folder::refill_folder;
use super::{FolderError, FolderSetup};
use crate::record::FolderRecord;

/// Makes the folder `wanted` at setup.path on `branch` (a short name) at `tip`, holding `cone`, and returns its
/// record. When check_before_make finds the folder there, differing only in its base and needed paths, refills it
/// through refill_folder (untouched when its index has unmerged entries) and returns that record. Otherwise, with
/// check_before_make's refusals (a different folder, the branch checked out elsewhere, free disk under the floor)
/// all before the branch is made: makes the branch through make_branch under `common`, adds the worktree detached
/// at `tip`, points its HEAD at the branch, then writes the record and fills the folder. The caller holds the
/// folders lock and has run oplog::recover.
pub(crate) fn make_or_refill(
    setup: &FolderSetup,
    common: &Path,
    wanted: FolderRecord,
    branch: &str,
    tip: &Oid,
    cone: &[String],
) -> Result<FolderRecord, FolderError> {
    let git = setup.git;
    if let Some(existing) = check_before_make(setup, &wanted, branch)? {
        return refill_folder(git, &setup.path, existing);
    }
    make_branch(git, &setup.actor, common, branch, tip)?;
    git.worktree_add(&setup.path, tip, None).map_err(FolderError::Git)?;
    attach_head(git, &setup.path, branch)?;
    fill_folder(git, &setup.path, &wanted, cone)?;
    Ok(wanted)
}
