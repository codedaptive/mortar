//! read_stalled_move: the folder move a kill or a refused move left undone, if any, read without finishing it (S-85,
//! S-97, S-116, S-143), from the op log settled in memory as recover would leave it (recovered_records), so a join
//! rebase killed before its Done record is seen; status writes nothing and takes no lock.
//! No unit test: it reads the op log and runs git (S-5, S-104); recovered_records_tests, find_pending_move_tests and
//! move_stalled_tests cover its decisions. Nagatha's integration group is to cover, with the mortar binary as a child
//! process in a request folder made by the landing verbs in a throwaway repository (tests running real git set
//! GIT_CONFIG_GLOBAL=/dev/null and GIT_CONFIG_NOSYSTEM=1):
//! - a kill after join's rebase op and before the folder move: status reports the stall (the request ref, the new
//!   tip and the folder's old tip) and gives next mr join; after the move status reports no stall (S-97);
//! - a join rebase killed after its update-ref and before its Done record (testkit kill shim,
//!   MORTAR_TEST_CRASH=after): status exits 0 with the stall and next mr join, stalled_from/stalled_to the op's
//!   old/new; refs, index and op log unchanged by status; mr join then finishes the move;
//! - an undo of a plain join rebase whose folder move was refused: status reports the undo's stall (the request ref
//!   back at the tip before the rebase, the folder at the rebased tip) and gives next mr undo (S-143);
//! - after mr undo of a resolve's final join step (S-136 (2), S-148): status reports no stall, shows the paused join
//!   and gives next mr resolve; the next save, join or resolve moves no folder files;
//! - a kill after join's rebase op and after the folder move's set_index(folder_tree), before the move itself: status
//!   reports the stall (S-87, S-88, S-116);
//! - a finished move followed by a staged edit is not reported as a stall (next mr save) (S-116);
//! - an index with unmerged entries (a paused join) is not reported as a stall;
//! - status moves no ref and writes no file: refs, the folder's index and the op log are unchanged after it.

use gitio::{Git, IndexTreeError};
use oplog::{
    decide_recovery, read_marked_log, read_ref_value, recovery_close_valued, unix_now, MarkedRecord, OplogError,
    ValuedRecord,
};

use super::{move_stalled, recovered_records};
use crate::context::FolderContext;
use crate::join::{find_pending_move, PendingMove};
use crate::report::{failure_report, Report};

/// The pending move of `request_ref` (find_pending_move over the op log settled in memory by recovered_records, and
/// the ref's value), kept only when it is stalled: the folder's index tree is still where the move started, the old
/// tip's tree or the op's folder_tree, and not the new tip's tree (move_stalled). find_pending_move keeps answering
/// after a finished move, so the index decides (S-97, S-116). An open intent recover would refuse as Mixed stays open
/// here. An index with unmerged entries has no tree and is not a stall; any other failure is failure_report for
/// `verb`. index_tree reads a copy of the index, so the folder's index and its lock are untouched (S-97 amended);
/// nothing here finishes the move or writes the log (S-85).
pub fn read_stalled_move(
    git: &Git,
    folder: &FolderContext,
    request_ref: &str,
    verb: &str,
) -> Result<Option<PendingMove>, Report> {
    let fail = |error: &dyn std::fmt::Display| failure_report(verb, &error.to_string());
    let log = read_marked_log(&folder.common).map_err(|error| fail(&error))?;
    let view = recovered_records(&log.records, |intent| close_in_memory(git, intent)).map_err(|error| fail(&error))?;
    let branch_at = git.read_ref(request_ref).map_err(|error| fail(&error))?;
    let Some(pending) = find_pending_move(&view, request_ref, branch_at.as_ref()) else {
        return Ok(None);
    };
    let old_tree = git.commit_info(&pending.old).map_err(|error| fail(&error))?.tree;
    let new_tree = git.commit_info(&pending.new).map_err(|error| fail(&error))?.tree;
    let index_tree = match git.index_tree(&folder.toplevel) {
        Ok(tree) => Some(tree),
        Err(IndexTreeError::Unmerged { .. }) => None,
        Err(IndexTreeError::Git(error)) => return Err(fail(&error)),
    };
    Ok(move_stalled(index_tree.as_ref(), &old_tree, pending.folder_tree.as_ref(), &new_tree).then_some(pending))
}

/// The record recover would append for `intent`: each move's ref read, decide_recovery, recovery_close_valued with
/// the intent's values. Mixed, which recover refuses and leaves open, is None.
fn close_in_memory(git: &Git, intent: &MarkedRecord) -> Result<Option<ValuedRecord>, OplogError> {
    let refs = intent
        .record
        .moves
        .iter()
        .map(|m| Ok((m.clone(), read_ref_value(git, &m.refname)?)))
        .collect::<Result<Vec<_>, OplogError>>()?;
    let valued = ValuedRecord { record: intent.record.clone(), values: intent.values.clone() };
    match recovery_close_valued(&valued, decide_recovery(&refs), unix_now()) {
        Ok(record) => Ok(Some(record)),
        Err(OplogError::Mixed { .. }) => Ok(None),
        Err(error) => Err(error),
    }
}
