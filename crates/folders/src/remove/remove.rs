//! Removes a request folder whose work has joined: its worktree by gitio, then its branch and snapshot ref by one
//! op-log op. No unit test for the git and op-log path: it runs git and appends to the op log. remove_tests
//! covers the unsaved refusal, unsaved_refusal_tests its four cases, remove_step_for_tests the folder's kind, HEAD and
//! containment, step_refusal_tests each refusal, repeat_step_for_tests a repeated op, and remove_op_tests the op as
//! data; attempts_refusal's header names the tests of the attempts check. A kill between worktree_remove and the op
//! leaves the branch, contained in its zone, with no folder; no work is lost, and sweep_refs deletes it and its
//! snapshot ref. tests/folders/removal.rs covers, in a throwaway repository: a joined, saved request
//! folder is removed with its branch and snapshot ref deleted by one folder-remove op in mortar/oplog.jsonl; a repeat
//! after that op was logged and the branch was made again at the same tip deletes it again; a branch not joined is
//! NotJoined, with nothing changed; a zone folder is ZoneFolder; a detached HEAD, an unborn branch and a worktree with
//! no record each refuse with NotAFolder; a missing zone branch is ZoneBranchMissing; an untracked file in the folder
//! is git's refusal, with nothing deleted; a snapshot ref moved between its read and the op, or a repeat with the
//! branch elsewhere, is RemovedRefsLeft, with the branch kept; a git or op-log error in the op passes through as Git or
//! Oplog. tests/folders/attempts_remove.rs covers the AttemptsRemain cases attempts_refusal's header names, and an
//! attempt folder being AttemptFolder, with nothing changed.

use std::path::{Path, PathBuf};

use gitio::Git;

use super::attempts_refusal::attempts_refusal;
use super::delete_folder_refs::delete_folder_refs;
use super::remove_step_for::remove_step_for;
use super::unsaved_refusal::unsaved_refusal;
use super::step_refusal::step_refusal;
use crate::make::{read_folder_record, FolderError};
use crate::plan::{request_branch, snap_ref, zone_branch};
use crate::record::FolderKind;

/// Removes the request folder at `folder`. `saved` is snapshot's saved_state(..).saved for it; false, or any
/// file in `refused` (those the snapshot size cap of `cap` bytes refused), is NotRemovedUnsaved before any git run. A
/// HEAD with no branch or no commit, a worktree with no folder record, a request record with no request, or HEAD on
/// another branch than the record's request branch is NotAFolder; an attempt folder is AttemptFolder, as soon as its
/// record is read; a zone folder is ZoneFolder; a missing zone branch is ZoneBranchMissing; a branch not contained in
/// its zone branch is NotJoined; a request whose attempt branches (refs/heads/try/<zone>/<request>/<k>, k an attempt
/// number) or attempt folders remain is AttemptsRemain (attempts_refusal), naming the branches by full name and the
/// folders by path as display text, with nothing changed; the next step is keep or drop_attempts; an attempt
/// folder whose branch is gone is refused by both (OrphanAttempt) until remake_orphans, which the caller runs first,
/// makes it ordinary, and with no snapshot to re-make it from it is copied out and removed by hand. Then git
/// removes the worktree without force, so git itself refuses modified or untracked files (Git, nothing deleted), and
/// one folder-remove op deletes the branch and the snapshot ref, if there is one; a ref not where it was read keeps
/// both refs and is RemovedRefsLeft, since the worktree is already gone. The caller holds the lock and runs
/// oplog::recover first; remove takes no lock itself.
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
        (FolderKind::Attempt(k), _) => {
            return Err(FolderError::AttemptFolder {
                path: folder.to_path_buf(),
                k,
            })
        }
        (FolderKind::Zone, _) => String::new(),
        (FolderKind::Request, Some(request)) => request.clone(),
        (FolderKind::Request, None) => return Err(not_a_folder("its record names no request".to_string())),
    };
    let zone_short = zone_branch(&record.zone).map_err(FolderError::Plan)?;
    let zone_ref = format!("refs/heads/{zone_short}");
    let (expected, merge_base) = match record.kind {
        // remove_step_for answers ZoneFolder for a zone record before it reads either; an attempt record
        // returned AttemptFolder above, so it never reaches here.
        FolderKind::Zone | FolderKind::Attempt(_) => (String::new(), None),
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
    // Only a request record reaches here: step_refusal answered ZoneFolder for a zone record.
    attempts_refusal(git, folder, &record.zone, &request)?;
    let snap_name = snap_ref(&record.zone, &request).map_err(FolderError::Plan)?;
    let snap_tip = git.read_ref(&snap_name).map_err(FolderError::Git)?;
    git.worktree_remove(folder, false).map_err(FolderError::Git)?;
    delete_folder_refs(git, folder, &head, &tip, snap_tip.as_ref().map(|oid| (snap_name.as_str(), oid)), actor)
}
