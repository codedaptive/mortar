//! Keeps one attempt: removes every attempt folder, then one folder-keep op moves the request branch to the kept
//! attempt's tip and deletes every attempt branch (S-69).
//! No unit test: it runs git and appends to the op log (S-5). request_branch_tests, attempt_branch_tests,
//! attempts_of_tests, keep_plan_tests, request_held_tests, keep_op_tests and ref_moved_from_tests cover its decisions.
//! A kill between the removals and the op leaves the remaining folders and every branch; a repeat given the same saved
//! list finds them and finishes. Nagatha's integration group is to cover, in a throwaway repository: keep(2) of 3 with
//! every dropped attempt saved leaves the request branch at try-2's tip, all three try folders and branches gone, and
//! one folder-keep op logged; an unsaved dropped attempt is AttemptsUnsaved, with nothing changed; a request branch
//! moved on (a commit on it after the attempts were cut) is RequestMovedOn, with nothing changed; an untracked file in
//! an attempt folder is git's refusal, with the op not run, and a repeat after cleaning it finishes; a repeat after a
//! completed keep is AttemptMissing; the attempts' trysnap refs are left in place.

use gitio::{Git, Oid};
use oplog::{apply, new_op_id, unix_now, ApplyOutcome, Op};

use super::{attempts_of, keep_op, keep_plan, request_held, Kept};
use crate::make::{ref_moved_from, FolderError};
use crate::plan::{attempt_branch, request_branch};

/// Keeps attempt `k` of `request` in `zone` and returns (the request branch's tip before, the kept tip), so the caller
/// can move the request folder with move_folder(old, new) (S-49). `saved` is the caller's is_saved per attempt to be
/// dropped, as (k, saved) (S-32). Before anything is touched: the request branch must exist (RequestBranchMissing),
/// attempt k must exist (AttemptMissing), every dropped attempt, with a folder or not, must be saved (AttemptsUnsaved),
/// and the kept tip must descend from the request tip (RequestMovedOn). Then git removes each attempt folder without
/// force, the kept attempt's last, so git itself refuses modified or untracked files (Git, the op not run). Then the
/// folder-keep op runs; see apply_keep. The caller holds the lock and runs oplog::recover first (S-16, S-25); keep
/// takes no lock itself. After a completed keep no attempt branch is left, so a repeat is AttemptMissing.
pub fn keep(
    git: &Git,
    zone: &str,
    request: &str,
    k: u32,
    saved: &[(u32, bool)],
    actor: &str,
) -> Result<(Oid, Oid), FolderError> {
    let rb = request_branch(zone, request).map_err(FolderError::Plan)?;
    let ab = attempt_branch(zone, request, k).map_err(FolderError::Plan)?;
    let request_ref = format!("refs/heads/{rb}");
    let request_tip = git
        .read_ref(&request_ref)
        .map_err(FolderError::Git)?
        .ok_or_else(|| FolderError::RequestBranchMissing { branch: rb.clone() })?;
    let refs = git
        .list_refs(&format!("refs/heads/try/{zone}/{request}/"))
        .map_err(FolderError::Git)?;
    let worktrees = git.worktree_list().map_err(FolderError::Git)?;
    let kept = keep_plan(&ab, k, &attempts_of(zone, request, &refs, &worktrees), saved)?;
    let merge_base = git.merge_base(&request_tip, &kept.tip).map_err(FolderError::Git)?;
    if !request_held(&request_tip, merge_base.as_ref()) {
        return Err(FolderError::RequestMovedOn { branch: rb, attempt: ab });
    }
    for folder in &kept.remove {
        git.worktree_remove(folder, false).map_err(FolderError::Git)?;
    }
    apply_keep(git, &request_ref, &request_tip, &kept, actor)?;
    Ok((request_tip, kept.tip))
}

/// Applies keep_op through oplog::apply on the common dir. Done is kept. Repeated means that exact keep was logged
/// before, yet keep_plan found the attempt branches again, so it applies once more with an op whose key ends in its
/// own op id, as make_branch and delete_folder_refs do. A Failed op is the RefMoved its record names (ref_moved_from);
/// a Failed record with no stale ref, or a second op answered Repeated, is RefMoved naming the request ref and
/// where it is now.
fn apply_keep(git: &Git, request_ref: &str, request_tip: &Oid, kept: &Kept, actor: &str) -> Result<(), FolderError> {
    let common = git.common_dir().map_err(FolderError::Git)?;
    let op = || -> Result<Op, FolderError> {
        keep_op(new_op_id(), actor.to_string(), unix_now(), request_ref, request_tip, kept).map_err(FolderError::Git)
    };
    let failed = match apply(git, &common, &op()?).map_err(FolderError::Oplog)? {
        ApplyOutcome::Done(_) => return Ok(()),
        ApplyOutcome::Failed(record) => Some(record),
        ApplyOutcome::Repeated(_) => {
            let mut again = op()?;
            again.key = format!("{}:{}", again.key, again.op_id);
            match apply(git, &common, &again).map_err(FolderError::Oplog)? {
                ApplyOutcome::Done(_) => return Ok(()),
                ApplyOutcome::Failed(record) => Some(record),
                ApplyOutcome::Repeated(_) => None,
            }
        }
    };
    if let Some(err) = failed.as_ref().and_then(ref_moved_from) {
        return Err(err);
    }
    Err(FolderError::RefMoved {
        refname: request_ref.to_string(),
        at: git.read_ref(request_ref).map_err(FolderError::Git)?,
    })
}
