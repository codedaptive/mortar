//! The checks both make functions run, under the lock, before the folder's branch, worktree or record is written.
//! No unit test: it reads the worktree list, the file system and free disk (S-5); repeat_decision_tests,
//! branch_holder_tests and floor_check_tests cover its decisions. Nagatha's integration group is to cover,
//! through make_zone_folder and make_request_folder: a repeat finds the record already there; a different
//! folder at the path refuses with DifferentFolder; a branch checked out elsewhere refuses with
//! BranchCheckedOut; a disk under the floor refuses with BelowFloor and leaves no branch and no worktree.

use std::path::Path;

use super::{branch_holder, existing_folder, repeat_decision, FolderError, FolderSetup, Repeat};
use crate::plan::floor_check;
use crate::record::FolderRecord;

/// Some(the record already there) when the folder at setup.path is `wanted` but for its base and needed paths:
/// a repeat, which the caller refills with refill_folder before answering it (a folder whose index has unmerged
/// entries is returned untouched). None when the folder may be made. Refuses a different folder at the path,
/// `branch` (a short name) checked out in another worktree, and free disk under setup.floor_bytes.
pub(crate) fn check_before_make(
    setup: &FolderSetup,
    wanted: &FolderRecord,
    branch: &str,
) -> Result<Option<FolderRecord>, FolderError> {
    let git = setup.git;
    if let Some(existing) = existing_folder(git, &setup.path)? {
        return match repeat_decision(wanted, &existing) {
            Repeat::Same => Ok(Some(existing)),
            Repeat::Different => Err(FolderError::DifferentFolder {
                path: setup.path.clone(),
                existing,
            }),
        };
    }
    let worktrees = git.worktree_list().map_err(FolderError::Git)?;
    if let Some(path) = branch_holder(&worktrees, &format!("refs/heads/{branch}"), &setup.path) {
        return Err(FolderError::BranchCheckedOut {
            branch: branch.to_string(),
            path,
        });
    }
    floor_check(free_disk_at(setup)?, setup.floor_bytes, &setup.sweep).map_err(FolderError::BelowFloor)?;
    Ok(None)
}

/// Free disk on the volume that will hold setup.path, asked of setup.path itself when it exists (an empty
/// directory), else of its nearest existing ancestor. A relative path none of whose parts exist is asked at ".".
fn free_disk_at(setup: &FolderSetup) -> Result<u64, FolderError> {
    let probe = setup
        .path
        .ancestors()
        .find(|p| p.exists())
        .unwrap_or(Path::new("."));
    setup.disk.free_bytes(probe).map_err(|reason| FolderError::DiskProbe {
        path: probe.to_path_buf(),
        reason,
    })
}
