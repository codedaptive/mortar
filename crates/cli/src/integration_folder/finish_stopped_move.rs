//! finish_stopped_move: the integration folder move a stopped land, back-out or publish left undone, finished by the
//! next of them under the land lock before its own land call, so a plain repeat after a kill is safe. A
//! refused move is finished by a repeat when the folder was cleared as the refusal says (paths moved out or put back
//! to the index copy, the index left as it is); a clear that resets the index to HEAD (the folder's HEAD is already
//! the branch's new commit) leaves no stall for the repeat to see, and the folder's files are then their owner's to
//! bring to HEAD, unreported.
//! No sibling test: it reads the op log and runs git and the folder move; last_integration_old_tests and
//! integration_move_stopped_tests cover its decisions, note_finished_move_tests the report line.
//! Repo integration tests to write (cross-zone, not written here), with the mortar binary as a child
//! process in a throwaway repository (GIT_CONFIG_GLOBAL=/dev/null, GIT_CONFIG_NOSYSTEM=1):
//! - a kill after land's op and before the folder move, then a plain repeat of land, back-out (land answering
//!   NothingToBackOut) or publish (land answering NothingToPublish): the folder moves from the op's old value to the
//!   tip and the report's first line says so;
//! - a staged edit in the integration folder (its index at another tree): no move, the folder untouched, the verb
//!   goes on;
//! - no integration folder (no worktree on the integration branch): no move, the verb goes on;
//! - integration never moved by an op (no Done record moving it): no move, the verb goes on;
//! - an unmerged index in the integration folder: no move, the verb goes on;
//! - files with local changes in the way in the integration folder at a stopped move: exit 1 "folder-not-moved"
//!   listing the paths, integration not moved by that run, the folder unchanged; move them out (or put them back to
//!   the index copy), repeat: the move finishes and the verb goes on;
//! - the same after a verb's own move (land, back-out, publish OriginMoved) was refused: move them out, repeat: the
//!   finish moves the folder and the verb goes on.

use std::path::Path;

use gitio::{Git, IndexTreeError};
use oplog::read_op_log;

use super::{integration_folder, integration_move_stopped, last_integration_old, move_integration_folder, FinishedMove};
use crate::report::{failure_report, Report};

/// The caller holds the land lock after recover. The last Done op that moved refs/heads/<integration_branch>
/// (last_integration_old) gives the old value; the branch missing, no such op, or no integration folder is
/// Ok(None). When the folder's index tree is still old's tree and the branch is past old
/// (integration_move_stopped), the folder is moved from old to the tip by move_integration_folder as `actor`, and the
/// move is the answer; any other index is Ok(None) with the folder untouched. A read that fails is failure_report
/// for `verb` in the cli's words. A refused move is move_integration_folder's report, whose first line says `verb`
/// stopped before its land call and moved nothing, and whose next ends with `repeat`: the caller returns it, so the
/// verb stops before its land call, integration is not moved by it and nothing changed.
pub fn finish_stopped_move(
    git: &Git,
    common: &Path,
    verb: &str,
    integration_branch: &str,
    actor: &str,
    repeat: &str,
) -> Result<Option<FinishedMove>, Report> {
    let log = read_op_log(common).map_err(|_| failure_report(verb, "the op log could not be read"))?;
    let integration_ref = format!("refs/heads/{integration_branch}");
    let Some(tip) = git
        .read_ref(&integration_ref)
        .map_err(|_| failure_report(verb, &format!("git could not read {integration_ref}")))?
    else {
        return Ok(None);
    };
    let Some(old) = last_integration_old(&log.records, &integration_ref) else {
        return Ok(None);
    };
    let Some(folder) = integration_folder(git, integration_branch).map_err(|reason| failure_report(verb, &reason))?
    else {
        return Ok(None);
    };
    let index_tree = match git.index_tree(&folder) {
        Ok(tree) => Some(tree),
        Err(IndexTreeError::Unmerged { .. }) => None,
        Err(IndexTreeError::Git(_)) => {
            return Err(failure_report(verb, "git could not read the integration folder's index"));
        }
    };
    let old_tree = git
        .commit_info(&old)
        .map_err(|_| failure_report(verb, &format!("git could not read commit {old}")))?
        .tree;
    if !integration_move_stopped(index_tree.as_ref(), &old, &old_tree, &tip) {
        return Ok(None);
    }
    let done = format!(
        "{verb} stopped before its land call and moved nothing: integration is at {tip}, where an earlier run moved \
         it, and its folder is still at {old}"
    );
    let moved = move_integration_folder(git, verb, integration_branch, &old, &tip, actor, &done, repeat)?;
    Ok(moved.map(|folder| FinishedMove { folder, old, tip }))
}
