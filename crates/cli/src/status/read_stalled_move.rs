//! read_stalled_move: the folder move a kill left undone, if any, read without finishing it (S-85, S-97, S-116).
//! No unit test: it reads the op log and runs git (S-5, S-104); find_pending_move_tests and move_stalled_tests cover
//! its decisions. Nagatha's integration group is to cover, with the mortar binary as a child process in a request
//! folder made by the landing verbs in a throwaway repository (tests running real git set
//! GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - a kill after join's rebase op and before the folder move: status reports the stall (the request ref, the new
//!   tip and the folder's old tip) and gives next mortar join; after the move status reports no stall (S-97);
//! - a kill after join's rebase op and after the folder move's set_index(folder_tree), before the move itself: status
//!   reports the stall (S-87, S-88, S-116);
//! - a finished move followed by a staged edit is not reported as a stall (next mortar save) (S-116);
//! - an index with unmerged entries (a paused join) is not reported as a stall;
//! - status moves no ref and writes no file: refs, the folder's index and the op log are unchanged after it.

use gitio::{Git, IndexTreeError};

use super::move_stalled;
use crate::context::FolderContext;
use crate::join::{read_pending_move, PendingMove};
use crate::report::{failure_report, Report};

/// The pending move of `request_ref` (read_pending_move), kept only when it is stalled: the folder's index tree is
/// still where the move started, the old tip's tree or the op's folder_tree, and not the new tip's tree
/// (move_stalled). find_pending_move keeps answering after a finished move, so the index decides (S-97, S-116). An
/// index with unmerged entries has no tree and is not a stall; any other failure is failure_report for `verb`.
/// index_tree reads a copy of the index, so the folder's index and its lock are untouched (S-97 amended); nothing
/// here finishes the move (S-85).
pub fn read_stalled_move(
    git: &Git,
    folder: &FolderContext,
    request_ref: &str,
    verb: &str,
) -> Result<Option<PendingMove>, Report> {
    let Some(pending) = read_pending_move(git, &folder.common, request_ref, verb)? else {
        return Ok(None);
    };
    let old_tree = git.commit_info(&pending.old).map_err(|error| failure_report(verb, &error.to_string()))?.tree;
    let new_tree = git.commit_info(&pending.new).map_err(|error| failure_report(verb, &error.to_string()))?.tree;
    let index_tree = match git.index_tree(&folder.toplevel) {
        Ok(tree) => Some(tree),
        Err(IndexTreeError::Unmerged { .. }) => None,
        Err(IndexTreeError::Git(error)) => return Err(failure_report(verb, &error.to_string())),
    };
    Ok(move_stalled(index_tree.as_ref(), &old_tree, pending.folder_tree.as_ref(), &new_tree).then_some(pending))
}
