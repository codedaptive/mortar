//! Drops every attempt of a request: removes every attempt folder, then one folder-drop-attempts op deletes every
//! attempt branch. An attempt folder whose branch is gone (an orphan) is OrphanAttempt before anything is
//! touched; next step: remake_orphans, which the caller runs first.
//! The request branch does not move; the attempts' trysnap refs stay until sweep_refs takes them when the request
//! closes.
//! No unit test: it runs git and appends to the op log. request_branch_tests, unsaved_attempts_tests,
//! drop_op_tests, ref_moved_from_tests, attempt_refs_left_tests, drop_plan_tests, attempts_of_tests and
//! orphan_attempts_tests cover its decisions. A kill between the removals and the op leaves the remaining
//! folders and every branch; a repeat given the same saved list finds them and finishes.
//! tests/folders/attempts_drop.rs covers, in a throwaway repository: a drop of 3 saved attempts removes all three
//! folders and branches in one folder-drop-attempts op, with the request branch unmoved and the trysnap refs left; an
//! unsaved attempt (with a folder, or a folderless branch) is AttemptsUnsaved, with nothing changed; an untracked file
//! in an attempt folder is git's refusal, with the op not run, and a repeat after cleaning it finishes; a repeat
//! after a completed drop returns empty and logs nothing; a try branch moved between the read and the op is
//! AttemptRefsLeft, with the branches kept and the folders already removed, and a repeat given the same saved list
//! finishes; an attempt folder whose try branch was deleted is OrphanAttempt naming its folder, with nothing touched,
//! and after remake_orphans a drop with every attempt saved finishes and remove of the request folder goes ahead.
//! tests/folders/attempts_remove.rs covers remove of the request folder going ahead after drop_attempts.

use gitio::Git;
use oplog::{apply, new_op_id, unix_now, ApplyOutcome};

use super::attempt_refs_left::attempt_refs_left;
use super::{attempts_of, drop_op, drop_plan, orphan_attempts, Dropped};
use crate::make::{ref_moved_from, FolderError};
use crate::plan::request_branch;

/// Drops every attempt of `request` in `zone` and returns the attempt numbers dropped, in k order. `saved` is the
/// caller's answer per attempt, as (k, saved): for an attempt with a folder, snapshot's saved_state(..).saved with
/// no file refused. Before anything is touched: the names must make a request
/// branch (Plan), no attempt folder may have lost its branch (OrphanAttempt naming them; next step: remake_orphans,
/// which the caller runs), and every attempt, with a folder or not, must be saved (AttemptsUnsaved). No attempts and no
/// orphan folders: Ok(empty), nothing removed and no op logged, which is also what a repeat after a completed drop
/// finds. Then git removes each folder without force, so git itself refuses modified or untracked files (Git, the op
/// not run). Then, when there is a branch to delete, the folder-drop-attempts op runs (see apply_drop); with none, no
/// op is logged. A try branch moved between the read and the op is AttemptRefsLeft, naming the folders already
/// removed; every attempt branch is kept. The caller holds the lock and runs oplog::recover first;
/// drop_attempts takes no lock itself.
pub fn drop_attempts(
    git: &Git,
    zone: &str,
    request: &str,
    saved: &[(u32, bool)],
    actor: &str,
) -> Result<Vec<u32>, FolderError> {
    let rb = request_branch(zone, request).map_err(FolderError::Plan)?;
    let refs = git
        .list_refs(&format!("refs/heads/try/{zone}/{request}/"))
        .map_err(FolderError::Git)?;
    let worktrees = git.worktree_list().map_err(FolderError::Git)?;
    let attempts = attempts_of(zone, request, &refs, &worktrees);
    let orphans = orphan_attempts(zone, request, &refs, &worktrees);
    if attempts.is_empty() && orphans.is_empty() {
        return Ok(Vec::new());
    }
    let dropped = drop_plan(&attempts, &orphans, saved)?;
    for folder in &dropped.remove {
        git.worktree_remove(folder, false).map_err(FolderError::Git)?;
    }
    if let Some((first, _)) = dropped.delete.first() {
        apply_drop(git, &format!("refs/heads/{rb}"), first, &dropped, actor)
            .map_err(|e| attempt_refs_left(e, &dropped.remove))?;
    }
    let mut ks: Vec<u32> = attempts.iter().map(|a| a.k).collect();
    ks.sort_unstable();
    Ok(ks)
}

/// Applies drop_op through oplog::apply on the common dir. Done is kept. A Failed op is the RefMoved its record
/// names (ref_moved_from); a Failed record with no stale ref, or Repeated (which the op's unique key rules out), is
/// RefMoved naming `first`, the first branch to delete, and where it is now.
fn apply_drop(git: &Git, request_ref: &str, first: &str, dropped: &Dropped, actor: &str) -> Result<(), FolderError> {
    let common = git.common_dir().map_err(FolderError::Git)?;
    let op = drop_op(new_op_id(), actor.to_string(), unix_now(), request_ref, dropped).map_err(FolderError::Git)?;
    let failed = match apply(git, &common, &op).map_err(FolderError::Oplog)? {
        ApplyOutcome::Done(_) => return Ok(()),
        ApplyOutcome::Failed(record) => Some(record),
        ApplyOutcome::Repeated(_) => None,
    };
    if let Some(err) = failed.as_ref().and_then(ref_moved_from) {
        return Err(err);
    }
    Err(FolderError::RefMoved {
        refname: first.to_string(),
        at: git.read_ref(first).map_err(FolderError::Git)?,
    })
}
