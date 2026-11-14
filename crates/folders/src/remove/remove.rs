//! Removes a request folder whose work has joined: its worktree by gitio, then its branch and snapshot ref by one
//! op-log op. No unit test for the git and op-log path: it runs git and appends to the op log (S-5). remove_tests
//! covers the unsaved refusal, unsaved_refusal_tests its four cases, remove_step_for_tests the folder's kind, HEAD
//! and containment, step_refusal_tests each refusal, repeat_step_for_tests a repeated op, and remove_op_tests the
//! op as data. A kill between worktree_remove and the op leaves the branch, contained in its zone, with no folder;
//! no work is lost, and sweep_refs deletes it and its snapshot ref (S-52 (1)). Nagatha's integration group is to cover,
//! in a throwaway repository: a joined, saved request folder is removed with its branch and snapshot ref deleted by
//! one folder-remove op in mortar/oplog.jsonl; a repeat after that op was logged and the branch was made again at
//! the same tip deletes it again; a branch not joined is NotJoined, with nothing changed; a zone folder is
//! ZoneFolder; a detached HEAD, an unborn branch and a worktree with no record each refuse with NotAFolder; a
//! missing zone branch is ZoneBranchMissing; an untracked file in the folder is git's refusal, with nothing
//! deleted; a snapshot ref moved between its read and the op, or a repeat with the branch elsewhere, is
//! RemovedRefsLeft, with the branch kept; a git or op-log error in the op passes through as Git or Oplog.

use std::path::{Path, PathBuf};

use gitio::Git;

use super::delete_folder_refs::delete_folder_refs;
use super::remove_step_for::remove_step_for;
use super::unsaved_refusal::unsaved_refusal;
use super::step_refusal::step_refusal;
use crate::make::{read_folder_record, FolderError};
use crate::plan::{request_branch, snap_ref, zone_branch};
use crate::record::FolderKind;

/// Removes the request folder at `folder`. `saved` is the caller's is_saved with no refused file (S-32); false, or
/// any file in `refused` (those the snapshot size cap of `cap` bytes refused), is NotRemovedUnsaved before any git
/// run. A HEAD with no branch or no commit, a worktree with no folder record, a request record with no request, or
/// HEAD on another branch than the record's request branch is NotAFolder; a zone folder is ZoneFolder; a missing
/// zone branch is ZoneBranchMissing; a branch not contained in its zone branch is NotJoined. Then git removes the
/// worktree without force, so git itself refuses modified or untracked files (Git, nothing deleted), and one
/// folder-remove op deletes the branch and the snapshot ref, if there is one; a ref not where it was read keeps both
/// refs and is RemovedRefsLeft, since the worktree is already gone. The caller holds the lock and runs
/// oplog::recover first (S-16, S-25); remove takes no lock itself.
pub fn remove(
    git: &Git,
    folder: &Path,
    saved: bool,
    refused: &[PathBuf],
    cap: Option<u64>,
    actor: &str,
) -> Result<(), FolderError> {
    if let Some(err) = unsaved_refusal(folder, saved, refused, cap) {
        return Err(err);
    }
    let not_a_folder = |reason: String| FolderError::NotAFolder {
        path: folder.to_path_buf(),
        reason,
    };
    let (head, tip) = git.head(folder).map_err(FolderError::Git)?;
    let Some(head) = head else {
        return Err(not_a_folder("its HEAD names no branch".to_string()));
    };
    let Some(tip) = tip else {
        return Err(not_a_folder("its branch has no commit".to_string()));
    };
    let git_dir = git.git_dir(folder).map_err(FolderError::Git)?;
    let Some(record) = read_folder_record(&git_dir)? else {
        return Err(not_a_folder("a worktree with no Mortar folder record".to_string()));
    };
    let request = match (record.kind, &record.request) {
        (FolderKind::Zone, _) => String::new(),
        (FolderKind::Request, Some(request)) => request.clone(),
        (FolderKind::Request, None) => return Err(not_a_folder("its record names no request".to_string())),
    };
    let zone_short = zone_branch(&record.zone).map_err(FolderError::Plan)?;
    let zone_ref = format!("refs/heads/{zone_short}");
    let (expected, merge_base) = match record.kind {
        // remove_step_for answers ZoneFolder for a zone record before it reads either.
        FolderKind::Zone => (String::new(), None),
        FolderKind::Request => {
            let rb = request_branch(&record.zone, &request).map_err(FolderError::Plan)?;
            let zone_tip = git
                .read_ref(&zone_ref)
                .map_err(FolderError::Git)?
                .ok_or(FolderError::ZoneBranchMissing { branch: zone_short })?;
            (format!("refs/heads/{rb}"), git.merge_base(&tip, &zone_tip).map_err(FolderError::Git)?)
        }
    };
    let step = remove_step_for(record.kind, &expected, &head, &tip, merge_base.as_ref());
    if let Some(err) = step_refusal(step, folder, &record.zone, &head, &expected, &zone_ref) {
        return Err(err);
    }
    let snap_name = snap_ref(&record.zone, &request).map_err(FolderError::Plan)?;
    let snap_tip = git.read_ref(&snap_name).map_err(FolderError::Git)?;
    git.worktree_remove(folder, false).map_err(FolderError::Git)?;
    delete_folder_refs(git, folder, &head, &tip, snap_tip.as_ref().map(|oid| (snap_name.as_str(), oid)), actor)
}
